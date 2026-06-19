// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use std::sync::Arc;
use std::time::Duration;

use crate::data_service::DataService;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Owned strings for [`WebConfig`] so `run()` can be spawned with `'static` futures.
struct OwnedWebConfig {
    bind: String,
    backend: String,
    scenario: Option<String>,
    docroot: Option<String>,
    workers: usize,
    strip_sources: bool,
    cache_ttl_secs: u64,
    spa: bool,
    allowed_origins: Vec<String>,
}

impl OwnedWebConfig {
    fn filesystem(port: u16) -> Self {
        Self {
            bind: format!("127.0.0.1:{port}"),
            backend: "filesystem".to_owned(),
            scenario: None,
            docroot: None,
            workers: 4,
            strip_sources: false,
            cache_ttl_secs: 0,
            spa: false,
            allowed_origins: Vec::new(),
        }
    }

    fn as_web_config(&self) -> WebConfig<'_> {
        WebConfig {
            bind: &self.bind,
            scenario: self.scenario.clone(),
            docroot: self.docroot.clone(),
            backend: &self.backend,
            workers: self.workers,
            strip_sources: self.strip_sources,
            cache_ttl_secs: self.cache_ttl_secs,
            spa: self.spa,
            allowed_origins: self.allowed_origins.clone(),
        }
    }
}

fn reserve_loopback_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("reserve loopback port")
        .local_addr()
        .expect("listener local addr")
        .port()
}

async fn probe_http_status(port: u16, path: &str) -> Option<u16> {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = tokio::net::TcpStream::connect(&addr).await.ok()?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.ok()?;
    let mut buf = [0u8; 512];
    let n = stream.read(&mut buf).await.unwrap_or(0);
    let resp = String::from_utf8_lossy(&buf[..n]);
    resp.lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
}

async fn wait_for_http(port: u16, path: &str, timeout_ms: u64) -> bool {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    while tokio::time::Instant::now() < deadline {
        if probe_http_status(port, path).await == Some(200) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

fn spawn_run(cfg: OwnedWebConfig) -> (tokio::task::JoinHandle<Result<(), AppError>>, u16) {
    let port: u16 = cfg
        .bind
        .rsplit(':')
        .next()
        .expect("bind address")
        .parse()
        .expect("port in bind address");
    let data_service = Arc::new(DataService::new());
    let handle = tokio::spawn(async move {
        let web_cfg = cfg.as_web_config();
        run(web_cfg, data_service).await
    });
    (handle, port)
}

#[tokio::test]
async fn test_run_filesystem_backend_starts_and_serves_routes() {
    let port = reserve_loopback_port();
    let (handle, port) = spawn_run(OwnedWebConfig::filesystem(port));

    for path in [
        "/health",
        "/health/liveness",
        "/health/readiness",
        "/api/status",
    ] {
        assert!(
            wait_for_http(port, path, 1500).await,
            "server on port {port} should respond 200 to {path}"
        );
    }

    assert_eq!(probe_http_status(port, "/").await, Some(200));
    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn test_run_filesystem_with_docroot_serves_static() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("page.html"), "<html>docroot page</html>").expect("write");

    let port = reserve_loopback_port();
    let mut cfg = OwnedWebConfig::filesystem(port);
    cfg.docroot = Some(tmp.path().to_string_lossy().into_owned());

    let (handle, port) = spawn_run(cfg);

    assert!(
        wait_for_http(port, "/health", 1500).await,
        "docroot server should start"
    );
    assert_eq!(probe_http_status(port, "/page.html").await, Some(200));

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn test_run_filesystem_with_spa_docroot() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("index.html"), "<html>spa fallback</html>").expect("write");

    let port = reserve_loopback_port();
    let mut cfg = OwnedWebConfig::filesystem(port);
    cfg.docroot = Some(tmp.path().to_string_lossy().into_owned());
    cfg.spa = true;

    let (handle, port) = spawn_run(cfg);

    assert!(wait_for_http(port, "/health", 1500).await);
    assert_eq!(
        probe_http_status(port, "/missing-spa-route").await,
        Some(200),
        "SPA mode should serve index.html for missing paths"
    );

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn test_run_with_cors_origins_starts() {
    let port = reserve_loopback_port();
    let mut cfg = OwnedWebConfig::filesystem(port);
    cfg.allowed_origins = vec![
        "http://localhost:3000".to_owned(),
        "https://example.com".to_owned(),
    ];

    let (handle, port) = spawn_run(cfg);

    assert!(
        wait_for_http(port, "/health", 1500).await,
        "CORS-enabled server should start"
    );

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn test_run_with_wildcard_cors_starts() {
    let port = reserve_loopback_port();
    let mut cfg = OwnedWebConfig::filesystem(port);
    cfg.allowed_origins = vec!["*".to_owned()];

    let (handle, port) = spawn_run(cfg);

    assert!(
        wait_for_http(port, "/api/primals", 1500).await,
        "wildcard CORS server should start"
    );

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn test_run_content_direct_backend_starts_and_serves_nav() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("docs")).expect("mkdir");
    std::fs::write(
        tmp.path().join("_index.md"),
        "+++\ntitle = \"Home\"\n+++\n# Home",
    )
    .expect("write index");

    let port = reserve_loopback_port();
    let mut cfg = OwnedWebConfig::filesystem(port);
    cfg.backend = "content-direct".to_owned();
    cfg.docroot = Some(tmp.path().to_string_lossy().into_owned());

    let (handle, port) = spawn_run(cfg);

    assert!(wait_for_http(port, "/health", 1500).await);
    assert_eq!(probe_http_status(port, "/").await, Some(200));
    assert_eq!(probe_http_status(port, "/api/nav").await, Some(200));
    assert_eq!(probe_http_status(port, "/api/viz").await, Some(200));

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn test_run_content_direct_default_docroot_missing_rejects() {
    let data_service = Arc::new(DataService::new());
    let mut cfg = test_config("127.0.0.1:0");
    cfg.backend = "content-direct";
    cfg.docroot = None;

    let result = run(cfg, data_service).await;
    assert!(result.is_err(), "missing default content/ dir should fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("docroot") || msg.contains("content"),
        "error should mention docroot: {msg}"
    );
}

#[tokio::test]
async fn test_run_content_provider_backend_starts() {
    use petal_tongue_core::test_fixtures::env_test_helpers;

    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("content-provider-run.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&sock_path).expect("bind unix sock");

    let port = reserve_loopback_port();
    let mut cfg = OwnedWebConfig::filesystem(port);
    cfg.backend = "content-provider".to_owned();

    let sock_str = sock_path.to_string_lossy().to_string();
    env_test_helpers::with_env_var_async("CONTENT_BACKEND_SOCKET", &sock_str, || async {
        let (handle, port) = spawn_run(cfg);

        assert!(
            wait_for_http(port, "/health", 1500).await,
            "content-provider server should start"
        );

        handle.abort();
        let _ = handle.await;
    })
    .await;
}

#[tokio::test]
async fn test_run_bind_address_already_in_use() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let port = listener.local_addr().expect("local addr").port();
    let bind = format!("127.0.0.1:{port}");

    let data_service = Arc::new(DataService::new());
    let result = run(test_config(&bind), data_service).await;

    assert!(result.is_err(), "binding to an in-use port should fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("in use") || msg.contains("Address") || msg.contains("IO error"),
        "bind error should describe address conflict: {msg}"
    );
}

#[tokio::test]
async fn test_run_starts_within_timeout() {
    let port = reserve_loopback_port();
    let cfg = OwnedWebConfig::filesystem(port);
    let data_service = Arc::new(DataService::new());
    let web_cfg = cfg.as_web_config();

    let result = tokio::time::timeout(Duration::from_millis(300), run(web_cfg, data_service)).await;

    match result {
        Err(_elapsed) => {
            // Server started and is waiting on shutdown_signal — expected.
        }
        Ok(Ok(())) => {
            // Graceful exit without signal is also acceptable in CI.
        }
        Ok(Err(e)) => {
            panic!("run() failed unexpectedly: {e}");
        }
    }
}
