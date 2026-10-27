mod http3;

use std::{
    env,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
};

use axum::{
    Json, Router,
    body::Body,
    http::{HeaderValue, Request, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get},
};
use serde_json::{Value, json};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_owned())
        .parse::<IpAddr>()?;
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8443".to_owned())
        .parse::<u16>()?;
    let cert_path = env::var_os("TLS_CERT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/certs/fullchain.pem"));
    let key_path = env::var_os("TLS_KEY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/certs/privkey.pem"));
    let server_name = env::var("TLS_SERVER_NAME").unwrap_or_else(|_| "localhost".to_owned());

    match env::args().nth(1).as_deref() {
        Some("--healthcheck") => {
            let address = healthcheck_address(host, port);
            tokio::time::timeout(
                Duration::from_secs(5),
                http3::healthcheck(address, &cert_path, &server_name),
            )
            .await
            .map_err(|_| "HTTP/3-Healthcheck überschritt fünf Sekunden")??;
            Ok(())
        }
        None => {
            let web_root = configured_root("WEB_ROOT", "/app/web", local_web_root());
            let asset_root = configured_root("ASSET_ROOT", "/app/assets", local_asset_root());
            http3::serve(
                SocketAddr::from((host, port)),
                router(web_root, asset_root),
                &cert_path,
                &key_path,
            )
            .await?;
            Ok(())
        }
        Some(argument) => Err(format!("Unbekanntes Argument: {argument}").into()),
    }
}

fn healthcheck_address(host: IpAddr, port: u16) -> SocketAddr {
    let address = match host {
        IpAddr::V4(address) if address.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(address) if address.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        address => address,
    };
    SocketAddr::from((address, port))
}

pub fn router(web_root: PathBuf, asset_root: PathBuf) -> Router {
    let index = web_root.join("index.html");
    let web = ServeDir::new(web_root)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(index));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/{*path}", any(api_not_found))
        .nest_service("/assets", ServeDir::new(asset_root))
        .fallback_service(web)
        .layer(middleware::from_fn(response_headers))
        .layer(CompressionLayer::new())
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": VERSION }))
}

async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "API-Endpunkt nicht gefunden" })),
    )
}

async fn response_headers(request: Request<Body>, next: Next) -> Response {
    let uri = request.uri().clone();
    let mut response = next.run(request).await;
    let is_success = response.status().is_success();
    let headers = response.headers_mut();

    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self'; media-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    if uri.path().ends_with(".wasm") {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/wasm"),
        );
    }

    if is_success {
        let is_html = headers
            .get(header::CONTENT_TYPE)
            .is_some_and(|value| value.as_bytes().starts_with(b"text/html"));
        let cache_control = if is_html {
            HeaderValue::from_static("no-cache")
        } else {
            cache_policy(&uri)
        };
        headers.insert(header::CACHE_CONTROL, cache_control);
    } else {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

fn cache_policy(uri: &Uri) -> HeaderValue {
    let path = uri.path();
    if path.starts_with("/api/") {
        return HeaderValue::from_static("no-store");
    }
    if path == "/" || path.ends_with('/') || path.ends_with(".html") {
        return HeaderValue::from_static("no-cache");
    }
    if is_versioned(uri) {
        return HeaderValue::from_static("public, max-age=31536000, immutable");
    }
    if path.starts_with("/assets/") {
        return HeaderValue::from_static("public, max-age=3600");
    }
    HeaderValue::from_static("public, max-age=300")
}

fn is_versioned(uri: &Uri) -> bool {
    let versioned_query = uri.query().is_some_and(|query| {
        query.split('&').any(|part| {
            part.split_once('=')
                .is_some_and(|(key, _)| matches!(key, "v" | "ver" | "version"))
        })
    });
    let hashed_name = uri.path().rsplit('/').next().is_some_and(|name| {
        name.split('.').any(|part| {
            (8..=64).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    });
    versioned_query || hashed_name
}

fn configured_root(variable: &str, container: &str, local: PathBuf) -> PathBuf {
    if let Some(value) = env::var_os(variable) {
        return PathBuf::from(value);
    }
    let container = PathBuf::from(container);
    if container.is_dir() { container } else { local }
}

fn local_web_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web")
}

fn local_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Backend liegt im Projektverzeichnis")
        .join("assets")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicU64, Ordering},
    };

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    use super::router;

    static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "totenrunde-web-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn join(&self, path: impl AsRef<Path>) -> PathBuf {
            self.0.join(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    use std::path::PathBuf;

    #[tokio::test]
    async fn health_returns_status_and_version() {
        let app = router(PathBuf::new(), PathBuf::new());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn index_is_served_for_spa_routes() {
        let web = TestDir::new();
        fs::write(
            web.join("index.html"),
            "<!doctype html><canvas id=\"game-canvas\"></canvas>",
        )
        .unwrap();
        let app = router(web.join(""), PathBuf::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/spieler/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("game-canvas"));
    }

    #[tokio::test]
    async fn static_files_and_wasm_have_correct_mime_types() {
        let web = TestDir::new();
        fs::write(web.join("game.js"), "export const ready = true;").unwrap();
        fs::write(web.join("game.wasm"), b"\0asm").unwrap();
        let app = router(web.join(""), PathBuf::new());

        let script = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/game.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(script.status(), StatusCode::OK);
        assert_eq!(script.headers()[header::CONTENT_TYPE], "text/javascript");

        let wasm = app
            .oneshot(
                Request::builder()
                    .uri("/game.wasm")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wasm.status(), StatusCode::OK);
        assert_eq!(wasm.headers()[header::CONTENT_TYPE], "application/wasm");
    }

    #[tokio::test]
    async fn unknown_api_route_returns_json_404() {
        let app = router(PathBuf::new(), PathBuf::new());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/gibt-es-nicht")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["error"], "API-Endpunkt nicht gefunden");
    }
}
