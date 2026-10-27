use std::{
    fs::File,
    future::Future,
    io::BufReader,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::Path,
    sync::Arc,
};

use axum::{Router, body::Body, http::StatusCode, response::IntoResponse};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use h3::{client::SendRequest, server::RequestResolver};
use http_body_util::BodyExt;
use quinn::{
    ClientConfig, Endpoint,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    rustls::{
        self,
        pki_types::{CertificateDer, PrivateKeyDer},
    },
};
use tower::ServiceExt;

const MAX_BODY_SIZE: usize = 1024 * 1024;
type DynError = Box<dyn std::error::Error + Send + Sync>;

pub async fn serve(
    address: SocketAddr,
    app: Router,
    cert_path: &Path,
    key_path: &Path,
) -> Result<(), DynError> {
    let certificates = load_certificates(cert_path)?;
    let key = load_private_key(key_path)?;
    let endpoint = server_endpoint(address, certificates, key)?;
    println!(
        "Totenrunde-Web lauscht ausschließlich per HTTP/3 auf https://{address} (UDP, ALPN h3)"
    );
    run_server(endpoint, app, shutdown_signal()).await
}

fn server_endpoint(
    address: SocketAddr,
    certificates: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<Endpoint, DynError> {
    let mut tls = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, key)?;
    tls.alpn_protocols = vec![b"h3".to_vec()];
    let config = quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls)?));
    Ok(Endpoint::server(config, address)?)
}

async fn run_server(
    endpoint: Endpoint,
    app: Router,
    shutdown: impl Future<Output = ()> + Send,
) -> Result<(), DynError> {
    tokio::select! {
        () = accept_connections(&endpoint, app) => {},
        () = shutdown => {},
    }
    endpoint.close(0u32.into(), b"server shutdown");
    endpoint.wait_idle().await;
    Ok(())
}

async fn accept_connections(endpoint: &Endpoint, app: Router) {
    while let Some(incoming) = endpoint.accept().await {
        let app = app.clone();
        tokio::spawn(async move {
            match incoming.await {
                Ok(connection) => serve_connection(connection, app).await,
                Err(error) => eprintln!("QUIC-Verbindung fehlgeschlagen: {error}"),
            }
        });
    }
}

async fn serve_connection(connection: quinn::Connection, app: Router) {
    let mut connection =
        match h3::server::Connection::new(h3_quinn::Connection::new(connection)).await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("HTTP/3-Verbindung fehlgeschlagen: {error}");
                return;
            }
        };

    loop {
        match connection.accept().await {
            Ok(Some(resolver)) => {
                let app = app.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_request(resolver, app).await {
                        eprintln!("HTTP/3-Anfrage fehlgeschlagen: {error}");
                    }
                });
            }
            Ok(None) => break,
            Err(error) => {
                if !error.is_h3_no_error() {
                    eprintln!("HTTP/3-Verbindung beendet: {error}");
                }
                break;
            }
        }
    }
}

async fn handle_request<C>(resolver: RequestResolver<C, Bytes>, app: Router) -> Result<(), DynError>
where
    C: h3::quic::Connection<Bytes>,
{
    let (request, mut stream) = resolver.resolve_request().await?;
    let (parts, _) = request.into_parts();
    let (body, too_large) = receive_request_body(&mut stream).await?;
    let mut response = if too_large {
        StatusCode::PAYLOAD_TOO_LARGE.into_response()
    } else {
        let request = axum::http::Request::from_parts(parts, Body::from(body));
        app.oneshot(request).await?
    };

    *response.version_mut() = axum::http::Version::HTTP_3;
    let (parts, mut body) = response.into_parts();
    stream
        .send_response(axum::http::Response::from_parts(parts, ()))
        .await?;
    while let Some(frame) = body.frame().await {
        let frame = frame?;
        if let Ok(data) = frame.into_data() {
            stream.send_data(data).await?;
        }
    }
    stream.finish().await?;
    Ok(())
}

async fn receive_request_body<S>(
    stream: &mut h3::server::RequestStream<S, Bytes>,
) -> Result<(Bytes, bool), DynError>
where
    S: h3::quic::RecvStream,
{
    let mut body = BytesMut::new();
    let mut too_large = false;
    while let Some(mut chunk) = stream.recv_data().await? {
        if body.len() + chunk.remaining() > MAX_BODY_SIZE {
            too_large = true;
            continue;
        }
        if !too_large {
            body.put(&mut chunk);
        }
    }
    Ok((body.freeze(), too_large))
}

pub async fn healthcheck(
    address: SocketAddr,
    cert_path: &Path,
    server_name: &str,
) -> Result<(), DynError> {
    let certificates = load_certificates(cert_path)?;
    let (status, _, body) = request_once(
        address,
        server_name,
        certificates,
        &format!("https://{server_name}/api/health"),
    )
    .await?;
    if status != StatusCode::OK {
        return Err(format!("Healthcheck lieferte HTTP {status}").into());
    }
    let health: serde_json::Value = serde_json::from_slice(&body)?;
    if health["status"] != "ok" {
        return Err("Healthcheck meldet keinen fehlerfreien Zustand".into());
    }
    println!("HTTP/3-Healthcheck erfolgreich: {health}");
    Ok(())
}

async fn request_once(
    address: SocketAddr,
    server_name: &str,
    certificates: Vec<CertificateDer<'static>>,
    uri: &str,
) -> Result<(StatusCode, axum::http::HeaderMap, Bytes), DynError> {
    let mut roots = rustls::RootCertStore::empty();
    for certificate in certificates {
        roots.add(certificate)?;
    }
    let mut tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![b"h3".to_vec()];
    let config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(tls)?));

    let bind_address = if address.is_ipv4() {
        SocketAddr::from((IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0))
    } else {
        SocketAddr::from((IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0))
    };
    let mut endpoint = Endpoint::client(bind_address)?;
    endpoint.set_default_client_config(config);
    let connection = endpoint.connect(address, server_name)?.await?;
    let (mut driver, mut sender) = h3::client::new(h3_quinn::Connection::new(connection)).await?;
    let driver_task =
        tokio::spawn(
            async move { std::future::poll_fn(|context| driver.poll_close(context)).await },
        );

    let result = receive_client_response(&mut sender, uri).await;
    drop(sender);
    endpoint.close(0u32.into(), b"request complete");
    let _ = driver_task.await;
    endpoint.wait_idle().await;
    result
}

async fn receive_client_response(
    sender: &mut SendRequest<h3_quinn::OpenStreams, Bytes>,
    uri: &str,
) -> Result<(StatusCode, axum::http::HeaderMap, Bytes), DynError> {
    let request = axum::http::Request::get(uri).body(())?;
    let mut stream = sender.send_request(request).await?;
    stream.finish().await?;
    let response = stream.recv_response().await?;
    let mut body = BytesMut::new();
    while let Some(mut chunk) = stream.recv_data().await? {
        if body.len() + chunk.remaining() > MAX_BODY_SIZE {
            return Err("HTTP/3-Antwort überschreitet das Größenlimit".into());
        }
        body.put(&mut chunk);
    }
    Ok((response.status(), response.headers().clone(), body.freeze()))
}

fn load_certificates(path: &Path) -> Result<Vec<CertificateDer<'static>>, DynError> {
    let file = File::open(path).map_err(|error| {
        format!(
            "TLS-Zertifikat {} konnte nicht geöffnet werden: {error}",
            path.display()
        )
    })?;
    let certificates =
        rustls_pemfile::certs(&mut BufReader::new(file)).collect::<Result<Vec<_>, _>>()?;
    if certificates.is_empty() {
        return Err(format!("TLS-Zertifikat {} ist leer", path.display()).into());
    }
    Ok(certificates)
}

fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, DynError> {
    let file = File::open(path).map_err(|error| {
        format!(
            "TLS-Schlüssel {} konnte nicht geöffnet werden: {error}",
            path.display()
        )
    })?;
    rustls_pemfile::private_key(&mut BufReader::new(file))?
        .ok_or_else(|| format!("TLS-Schlüssel {} ist leer oder ungültig", path.display()).into())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Strg+C-Handler konnte nicht installiert werden");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM-Handler konnte nicht installiert werden")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };

    use axum::http::{StatusCode, header};
    use quinn::rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    use tokio::sync::oneshot;

    use super::{request_once, run_server, server_endpoint};
    use crate::router;

    static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "totenrunde-h3-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn join(&self, path: &str) -> PathBuf {
            self.0.join(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[tokio::test]
    async fn serves_health_and_static_files_over_real_http3() {
        let web = TestDir::new();
        fs::write(
            web.join("index.html"),
            "<canvas id=\"game-canvas\"></canvas>",
        )
        .unwrap();
        fs::write(web.join("game.wasm"), b"\0asm\x01\0\0\0").unwrap();

        let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let endpoint = server_endpoint(
            SocketAddr::from((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)),
            vec![certificate.clone()],
            key,
        )
        .unwrap();
        let address = endpoint.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server = tokio::spawn(run_server(
            endpoint,
            router(web.join(""), PathBuf::new()),
            async {
                let _ = shutdown_rx.await;
            },
        ));

        let (status, _, body) = request_once(
            address,
            "localhost",
            vec![certificate.clone()],
            "https://localhost/api/health",
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"],
            "ok"
        );

        let (status, headers, body) = request_once(
            address,
            "localhost",
            vec![certificate],
            "https://localhost/game.wasm",
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_TYPE], "application/wasm");
        assert_eq!(body.as_ref(), b"\0asm\x01\0\0\0");

        shutdown_tx.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
