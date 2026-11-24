//! Controller tests (DESIGN.md §6). Fake modules are small sh scripts, so no Docker is needed.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mplab_controller::{BODY_LIMIT, Config, LANGUAGES, app};
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

const OK: &str = r#"{"status":"success","stdout":"hi","stderr":"","exit_code":0,"execution_time_ms":3}"#;

struct Env {
    _dir: TempDir,
    root: PathBuf,
    cfg: Config,
}

impl Env {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let root = dir.path().canonicalize().unwrap(); // macOS: /var -> /private/var
        let cfg = Config {
            root: root.clone(),
            max_concurrent: 4,
            slot_wait: Duration::from_secs(1),
            backstop: Duration::from_secs(5),
        };
        Env { _dir: dir, root, cfg }
    }

    /// Writes languages/<lang>/execute as an sh script with the given mode.
    fn module(&self, lang: &str, script: &str, mode: u32) -> &Self {
        let d = self.root.join("languages").join(lang);
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("execute");
        std::fs::write(&p, format!("#!/bin/sh\n{script}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode)).unwrap();
        self
    }

    fn ok_module(&self, lang: &str, script: &str) -> &Self {
        self.module(lang, script, 0o755)
    }

    fn app(&self) -> Router {
        app(self.cfg.clone())
    }
}

async fn send(app: Router, req: Request<Body>) -> (StatusCode, HeaderMap, Bytes) {
    let resp = app.oneshot(req).await.unwrap();
    let (parts, body) = resp.into_parts();
    (parts.status, parts.headers, body.collect().await.unwrap().to_bytes())
}

async fn post(app: Router, body: impl Into<Body>) -> (StatusCode, Value) {
    let req = Request::post("/api/execute").header("content-type", "application/json").body(body.into()).unwrap();
    let (code, _, bytes) = send(app, req).await;
    (code, serde_json::from_slice(&bytes).expect("response is JSON"))
}

fn assert_spec_error(v: &Value, status: &str) {
    assert_eq!(v["status"], status, "{v}");
    assert_eq!(v["stdout"], "");
    assert!(v["stderr"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(v["exit_code"].is_null() && v["execution_time_ms"].is_null());
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

// 1
#[tokio::test]
async fn unsupported_languages_start_no_module() {
    let env = Env::new();
    for lang in LANGUAGES.map(|(v, _)| v) {
        env.ok_module(lang, &format!("touch \"$PWD/started\"; echo '{OK}'"));
    }
    for lang in [json!("Python"), json!("rust"), json!(""), json!(42), Value::Null] {
        let mut req = json!({"source_code": "x"});
        if !lang.is_null() {
            req["language"] = lang;
        }
        let (code, v) = post(env.app(), req.to_string()).await;
        assert_eq!(code, StatusCode::OK);
        assert_spec_error(&v, "unsupported_language");
    }
    // Language is checked before fields.
    let (_, v) = post(env.app(), r#"{"language":"rust"}"#).await;
    assert_eq!(v["status"], "unsupported_language");
    assert!(!env.root.join("started").exists());
}

// 2
#[tokio::test]
async fn module_response_is_returned_byte_for_byte() {
    let env = Env::new();
    let raw = r#"{"stdout":"hi","status":"success","stderr":"","exit_code":0,"execution_time_ms":3,"extra":[1]}"#;
    env.ok_module("ruby", &format!("cat >/dev/null; printf '%s\\n' '{raw}'"));
    let req = Request::post("/api/execute").body(Body::from(r#"{"language":"ruby","source_code":"x"}"#)).unwrap();
    let (code, headers, body) = send(env.app(), req).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(headers["content-type"], "application/json");
    assert_eq!(body, format!("{raw}\n"));
}

// 3
#[tokio::test]
async fn module_gets_exact_request_in_repo_root() {
    let env = Env::new();
    env.ok_module("ruby", &format!("cat > got.json; pwd > cwd.txt; echo '{OK}'"));
    for req in [
        json!({"language": "ruby", "source_code": "x"}),
        json!({"language": "ruby", "source_code": "x", "stdin": null}),
    ] {
        let (_, v) = post(env.app(), req.to_string()).await;
        assert_eq!(v["status"], "success");
        let got: Value = serde_json::from_str(&read(&env.root.join("got.json"))).unwrap();
        assert_eq!(got, json!({"language": "ruby", "source_code": "x", "stdin": ""}));
    }
    let (_, _) = post(env.app(), json!({"language": "ruby", "source_code": "", "stdin": "in"}).to_string()).await;
    let got: Value = serde_json::from_str(&read(&env.root.join("got.json"))).unwrap();
    assert_eq!(got, json!({"language": "ruby", "source_code": "", "stdin": "in"}));
    assert_eq!(read(&env.root.join("cwd.txt")).trim(), env.root.to_str().unwrap());
}

// 4
#[tokio::test]
async fn malformed_requests_get_400() {
    let env = Env::new();
    env.ok_module("ruby", &format!("echo '{OK}'"));
    for body in [
        "not json",
        "[1,2]",
        r#"{"language":"ruby"}"#,
        r#"{"language":"ruby","source_code":5}"#,
        r#"{"language":"ruby","source_code":"x","stdin":7}"#,
    ] {
        let (code, v) = post(env.app(), body).await;
        assert_eq!(code, StatusCode::BAD_REQUEST, "{body}");
        assert_spec_error(&v, "internal_error");
    }
}

// 5
#[tokio::test]
async fn oversized_body_gets_413() {
    let env = Env::new();
    let (code, v) = post(env.app(), vec![b' '; BODY_LIMIT + 1]).await;
    assert_eq!(code, StatusCode::PAYLOAD_TOO_LARGE);
    assert_spec_error(&v, "internal_error");
}

// 6
#[tokio::test]
async fn module_failures_become_internal_error() {
    let cases = [
        (format!("echo '{OK}'; exit 3"), "exited with code 3"),
        ("echo hello".to_string(), "returned invalid JSON"),
        (r#"echo '{"status":"success","stdout":"","stderr":"","exit_code":0}'"#.into(), "missing field execution_time_ms"),
        (r#"echo '{"status":"ok","stdout":"","stderr":"","exit_code":0,"execution_time_ms":1}'"#.into(), "unknown status"),
        ("kill -SEGV $$".into(), "killed by signal 11"),
    ];
    for (script, reason) in cases {
        let env = Env::new();
        env.ok_module("ruby", &script);
        let (code, v) = post(env.app(), r#"{"language":"ruby","source_code":"x"}"#).await;
        assert_eq!(code, StatusCode::OK);
        assert_spec_error(&v, "internal_error");
        assert!(v["stderr"].as_str().unwrap().contains(reason), "{script}: {v}");
    }
}

#[tokio::test]
async fn module_that_cannot_start_is_internal_error() {
    let env = Env::new();
    let p = env.root.join("languages/ruby/execute");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, "#!/nonexistent/interpreter\n").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (_, v) = post(env.app(), r#"{"language":"ruby","source_code":"x"}"#).await;
    assert_spec_error(&v, "internal_error");
    assert!(v["stderr"].as_str().unwrap().contains("could not be started"), "{v}");
}

#[tokio::test]
async fn hung_module_is_killed_with_its_children() {
    let mut env = Env::new();
    env.cfg.backstop = Duration::from_millis(500);
    env.ok_module("ruby", "sleep 60 & echo $! > child.pid; wait");
    let (_, v) = post(env.app(), r#"{"language":"ruby","source_code":"x"}"#).await;
    assert_spec_error(&v, "internal_error");
    assert!(v["stderr"].as_str().unwrap().contains("did not answer"), "{v}");

    let pid: i32 = read(&env.root.join("child.pid")).trim().parse().unwrap();
    let pid = nix::unistd::Pid::from_raw(pid);
    for _ in 0..50 {
        if nix::sys::signal::kill(pid, None).is_err() {
            return; // grandchild is gone
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("the module's child process survived the backstop");
}

// 7
#[tokio::test]
async fn missing_or_non_executable_module() {
    let env = Env::new();
    env.module("ruby", &format!("echo '{OK}'"), 0o644);
    let (code, v) = post(env.app(), r#"{"language":"go","source_code":"x"}"#).await;
    assert_eq!(code, StatusCode::OK);
    assert_spec_error(&v, "internal_error");
    assert!(v["stderr"].as_str().unwrap().contains("Go module is not connected"), "{v}");

    let (_, v) = post(env.app(), r#"{"language":"ruby","source_code":"x"}"#).await;
    assert!(v["stderr"].as_str().unwrap().contains("not executable"), "{v}");

    let langs = get_languages(env.app()).await;
    assert!(langs.iter().all(|l| l["available"] == false));
}

// 8
async fn get_languages(app: Router) -> Vec<Value> {
    let (code, _, body) = send(app, Request::get("/api/languages").body(Body::empty()).unwrap()).await;
    assert_eq!(code, StatusCode::OK);
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn languages_in_spec_order_with_availability() {
    let env = Env::new();
    env.ok_module("ruby", "true").ok_module("cpp", "true");
    let langs = get_languages(env.app()).await;
    let values: Vec<&str> = langs.iter().map(|l| l["value"].as_str().unwrap()).collect();
    assert_eq!(values, ["c", "cpp", "java", "python", "javascript", "go", "ruby", "php", "haskell", "prolog"]);
    assert_eq!(langs[1], json!({"value": "cpp", "name": "C++", "available": true}));
    assert_eq!(langs[6]["available"], true);
    assert_eq!(langs[0]["available"], false);
}

// 9
#[tokio::test]
async fn cors_on_preflight_and_every_response() {
    let env = Env::new();
    let pre = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/execute")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let (code, h, _) = send(env.app(), pre).await;
    assert!(code.is_success());
    assert_eq!(h["access-control-allow-origin"], "*");
    assert!(h.contains_key("access-control-allow-methods"));

    for req in [
        Request::get("/nope").header("origin", "http://x").body(Body::empty()).unwrap(),
        Request::get("/api/languages").header("origin", "http://x").body(Body::empty()).unwrap(),
        Request::post("/api/execute").header("origin", "http://x").body(Body::from("bad")).unwrap(),
    ] {
        let (_, h, _) = send(env.app(), req).await;
        assert_eq!(h["access-control-allow-origin"], "*");
    }

    let (code, _, body) = send(env.app(), Request::get("/nope").body(Body::empty()).unwrap()).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert_spec_error(&serde_json::from_slice(&body).unwrap(), "internal_error");
}

// 10
#[tokio::test]
async fn busy_server_gets_503() {
    let mut env = Env::new();
    env.cfg.max_concurrent = 1;
    env.cfg.slot_wait = Duration::from_millis(200);
    env.ok_module("ruby", &format!("cat >/dev/null; sleep 1; echo '{OK}'"));
    let app = env.app();
    let body = r#"{"language":"ruby","source_code":"x"}"#;

    let first = tokio::spawn(post(app.clone(), body));
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (code, v) = post(app.clone(), body).await;
    assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
    assert_spec_error(&v, "internal_error");

    let (_, v) = first.await.unwrap();
    assert_eq!(v["status"], "success");
    // The slot is released after the first request.
    assert_eq!(post(app, body).await.1["status"], "success");
}
