//! HTTPS über TCP (HTTP/1.1 und HTTP/2). Browser bauen zu einer unbekannten Seite
//! immer zuerst eine TCP-Verbindung auf und wechseln erst nach dem `Alt-Svc`-Header
//! auf HTTP/3; ohne diesen Zugang wäre der QUIC-Server im Browser nicht erreichbar.

use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Router,
    http::{HeaderMap, Request, StatusCode, header},
};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Limited};
use hyper_util::{
    rt::{TokioExecutor, TokioIo, TokioTimer},
    server::{conn::auto::Builder, graceful::GracefulShutdown},
    service::TowerToHyperService,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    TlsAcceptor, TlsConnector,
    rustls::{
        self,
        pki_types::{CertificateDer, ServerName},
    },
};

use crate::{DynError, MAX_BODY_SIZE, tls};

const TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const HEADER_READ_TIMEOUT: Duration = Duration::from_secs(30);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn run_server(
    listener: TcpListener,
    mut tls: rustls::ServerConfig,
    app: Router,
    shutdown: impl Future<Output = ()> + Send,
) -> Result<(), DynError> {
    tls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let acceptor = TlsAcceptor::from(Arc::new(tls));
    let graceful = GracefulShutdown::new();
    tokio::pin!(shutdown);

    loop {
        let stream = tokio::select! {
            () = &mut shutdown => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => stream,
                Err(error) => {
                    // z. B. EMFILE: kurz warten, statt in einer Fehlerschleife zu kreisen
                    eprintln!("TCP-Verbindung konnte nicht angenommen werden: {error}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            },
        };
        let acceptor = acceptor.clone();
        let service = TowerToHyperService::new(app.clone());
        let watcher = graceful.watcher();
        tokio::spawn(async move {
            // Abgebrochene Handshakes (Scanner, Port-Checks) sind normal und werden nicht protokolliert.
            let Ok(Ok(stream)) =
                tokio::time::timeout(TLS_HANDSHAKE_TIMEOUT, acceptor.accept(stream)).await
            else {
                return;
            };
            let mut builder = Builder::new(TokioExecutor::new());
            builder
                .http1()
                .timer(TokioTimer::new())
                .header_read_timeout(HEADER_READ_TIMEOUT);
            builder.http2().timer(TokioTimer::new());
            let connection = builder.serve_connection(TokioIo::new(stream), service);
            if let Err(error) = watcher.watch(connection).await {
                eprintln!("HTTPS-Verbindung fehlgeschlagen: {error}");
            }
        });
    }

    drop(listener);
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    Ok(())
}

pub async fn request_once(
    address: SocketAddr,
    server_name: &str,
    certificates: Vec<CertificateDer<'static>>,
    path: &str,
) -> Result<(StatusCode, HeaderMap, Bytes), DynError> {
    let tls = tls::client_config(certificates, b"http/1.1")?;
    let stream = TcpStream::connect(address).await?;
    let stream = TlsConnector::from(Arc::new(tls))
        .connect(ServerName::try_from(server_name.to_owned())?, stream)
        .await?;
    let (mut sender, connection) =
        hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
    let driver = tokio::spawn(connection);

    let request = Request::get(path)
        .header(header::HOST, server_name)
        .body(Empty::<Bytes>::new())?;
    let response = sender.send_request(request).await?;
    let (parts, body) = response.into_parts();
    let body = Limited::new(body, MAX_BODY_SIZE)
        .collect()
        .await?
        .to_bytes();
    driver.abort();
    Ok((parts.status, parts.headers, body))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        path::PathBuf,
        time::Duration,
    };

    use axum::http::{HeaderValue, StatusCode, header};
    use quinn::rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    use tokio::{net::TcpListener, sync::oneshot};
    use tower_http::set_header::SetResponseHeaderLayer;

    use super::{request_once, run_server};
    use crate::{router, tls};

    #[tokio::test]
    async fn serves_https_over_tcp_and_advertises_http3() {
        let web = std::env::temp_dir().join(format!("totenrunde-tcp-test-{}", std::process::id()));
        fs::create_dir_all(&web).unwrap();
        fs::write(
            web.join("index.html"),
            "<canvas id=\"game-canvas\"></canvas>",
        )
        .unwrap();

        let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let listener = TcpListener::bind(SocketAddr::from((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let app = router(web.clone(), PathBuf::new()).layer(SetResponseHeaderLayer::overriding(
            header::ALT_SVC,
            HeaderValue::from_static("h3=\":443\"; ma=86400"),
        ));
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server = tokio::spawn(run_server(
            listener,
            tls::server_config(vec![certificate.clone()], key).unwrap(),
            app,
            async {
                let _ = shutdown_rx.await;
            },
        ));

        let (status, headers, body) = request_once(
            address,
            "localhost",
            vec![certificate.clone()],
            "/api/health",
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::ALT_SVC], "h3=\":443\"; ma=86400");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"],
            "ok"
        );

        let (status, _, body) = request_once(address, "localhost", vec![certificate], "/")
            .await
            .unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(String::from_utf8_lossy(&body).contains("game-canvas"));

        shutdown_tx.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        fs::remove_dir_all(web).unwrap();
    }
}
