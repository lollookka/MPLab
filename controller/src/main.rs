use std::time::Duration;

use mplab_controller::{Config, app};

fn env_num<T: std::str::FromStr>(name: &str) -> Option<T> {
    let v = std::env::var(name).ok()?;
    Some(v.parse().unwrap_or_else(|_| panic!("{name} must be a number, got {v:?}")))
}

#[tokio::main]
async fn main() {
    let port: u16 = env_num("MPLAB_PORT").unwrap_or(8000);
    let max_concurrent = env_num("MPLAB_MAX_CONCURRENT")
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
    let root = std::env::var("MPLAB_ROOT").unwrap_or_else(|_| ".".into());
    // Absolute, because module paths are resolved against it after the child changes directory.
    let root = std::fs::canonicalize(&root).unwrap_or_else(|e| panic!("MPLAB_ROOT {root:?}: {e}"));

    let cfg = Config { root, max_concurrent, slot_wait: Duration::from_secs(30), backstop: Duration::from_secs(30) };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|e| panic!("cannot listen on 127.0.0.1:{port}: {e}"));
    eprintln!(
        "{}",
        serde_json::json!({"event": "started", "port": port, "root": cfg.root, "max_concurrent": max_concurrent})
    );
    axum::serve(listener, app(cfg)).await.expect("server failed");
}
