//! MPLab execution controller. See DESIGN.md for the why behind each step.

use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tower_http::cors::CorsLayer;

/// API_SPEC section 4, in order. Drives validation and GET /api/languages.
pub const LANGUAGES: [(&str, &str); 10] = [
    ("c", "C"), ("cpp", "C++"), ("java", "Java"), ("python", "Python"),
    ("javascript", "JavaScript"), ("go", "Go"), ("ruby", "Ruby"),
    ("php", "PHP"), ("haskell", "Haskell"), ("prolog", "Prolog"),
];

const STATUSES: [&str; 6] = [
    "success", "compile_error", "runtime_error", "timeout", "unsupported_language", "internal_error",
];
const FIELDS: [&str; 5] = ["status", "stdout", "stderr", "exit_code", "execution_time_ms"];
pub const BODY_LIMIT: usize = 1024 * 1024;
const STDERR_LOG_TAIL: usize = 2048;

#[derive(Clone)]
pub struct Config {
    /// Repo root that holds `languages/`. Must be absolute.
    pub root: PathBuf,
    pub max_concurrent: usize,
    pub slot_wait: Duration,
    pub backstop: Duration,
}

struct AppState {
    cfg: Config,
    slots: Arc<Semaphore>,
}

pub fn app(cfg: Config) -> Router {
    let state = Arc::new(AppState { slots: Arc::new(Semaphore::new(cfg.max_concurrent)), cfg });
    Router::new()
        .route("/api/execute", post(execute))
        .route("/api/languages", get(languages))
        .fallback(|| async { reply(StatusCode::NOT_FOUND, "internal_error", "Not found.") })
        .method_not_allowed_fallback(|| async {
            reply(StatusCode::METHOD_NOT_ALLOWED, "internal_error", "Method not allowed.")
        })
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// A response in the spec shape, written by the controller itself.
fn reply(code: StatusCode, status: &str, msg: &str) -> Response {
    let body = json!({"status": status, "stdout": "", "stderr": msg, "exit_code": null, "execution_time_ms": null});
    (code, Json(body)).into_response()
}

fn module_path(root: &Path, lang: &str) -> PathBuf {
    root.join("languages").join(lang).join("execute")
}

/// None: no file. Some(false): there but not an executable file.
fn executable(path: &Path) -> Option<bool> {
    let m = std::fs::metadata(path).ok()?;
    Some(m.is_file() && m.permissions().mode() & 0o111 != 0)
}

async fn languages(State(st): State<Arc<AppState>>) -> Json<Value> {
    let list: Vec<Value> = LANGUAGES
        .iter()
        .map(|(value, name)| {
            let available = executable(&module_path(&st.cfg.root, value)) == Some(true);
            json!({"value": value, "name": name, "available": available})
        })
        .collect();
    Json(Value::Array(list))
}

/// Fields for the one log line per POST /api/execute.
#[derive(Default)]
struct Log {
    language: Option<&'static str>,
    source_bytes: Option<usize>,
    status: &'static str,
    reason: Option<String>,
    module_stderr: Option<String>,
}

async fn execute(State(st): State<Arc<AppState>>, body: Result<Bytes, BytesRejection>) -> Response {
    let start = Instant::now();
    let mut log = Log::default();
    let resp = handle(&st, body, &mut log).await;
    let time_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let mut line = json!({
        "time_ms": time_ms, "language": log.language, "status": log.status,
        "http": resp.status().as_u16(), "wall_ms": start.elapsed().as_millis(),
        "source_bytes": log.source_bytes,
    });
    if let Some(r) = log.reason { line["reason"] = r.into(); }
    if let Some(e) = log.module_stderr { line["module_stderr"] = e.into(); }
    eprintln!("{line}");
    resp
}

async fn handle(st: &AppState, body: Result<Bytes, BytesRejection>, log: &mut Log) -> Response {
    log.status = "internal_error";
    let body = match body {
        Ok(b) => b,
        Err(r) if r.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return reply(StatusCode::PAYLOAD_TOO_LARGE, "internal_error", "Request body is over 1 MB.");
        }
        Err(_) => return reply(StatusCode::BAD_REQUEST, "internal_error", "Could not read the request body."),
    };
    let req: serde_json::Map<String, Value> = match serde_json::from_slice(&body) {
        Ok(Value::Object(m)) => m,
        _ => return reply(StatusCode::BAD_REQUEST, "internal_error", "Request body must be a JSON object."),
    };

    // Language comes before field checks: {"language":"rust"} is unsupported, not a 400.
    let wanted = req.get("language").and_then(Value::as_str);
    let Some(&(lang, name)) = LANGUAGES.iter().find(|(v, _)| Some(*v) == wanted) else {
        log.status = "unsupported_language";
        let accepted: Vec<&str> = LANGUAGES.iter().map(|(v, _)| *v).collect();
        let msg = format!("Unsupported language. Accepted values: {}", accepted.join(", "));
        return reply(StatusCode::OK, "unsupported_language", &msg);
    };
    log.language = Some(lang);

    let Some(source) = req.get("source_code").and_then(Value::as_str) else {
        return reply(StatusCode::BAD_REQUEST, "internal_error", "source_code must be a string.");
    };
    log.source_bytes = Some(source.len());
    let stdin = match req.get("stdin") {
        None | Some(Value::Null) => "",
        Some(Value::String(s)) => s,
        _ => return reply(StatusCode::BAD_REQUEST, "internal_error", "stdin must be a string."),
    };

    let path = module_path(&st.cfg.root, lang);
    match executable(&path) {
        None => {
            let msg = format!("The {name} module is not connected to MPLab yet.");
            return reply(StatusCode::OK, "internal_error", &msg);
        }
        Some(false) => {
            log.reason = Some("execute file is not executable".into());
            let msg = format!("The {name} module's execute file is not executable.");
            return reply(StatusCode::OK, "internal_error", &msg);
        }
        Some(true) => {}
    }

    let permit = match timeout(st.cfg.slot_wait, st.slots.clone().acquire_owned()).await {
        Ok(Ok(p)) => p,
        _ => return reply(StatusCode::SERVICE_UNAVAILABLE, "internal_error", "MPLab is busy, try again."),
    };

    let input = json!({"language": lang, "source_code": source, "stdin": stdin}).to_string();
    // Spawned so a caller hanging up can't cancel the backstop or leak the permit.
    let task = tokio::spawn(run_module(path, st.cfg.root.clone(), input, st.cfg.backstop, permit));
    let (stdout, stderr, outcome) = match task.await {
        Ok(r) => r,
        Err(e) => (Vec::new(), Vec::new(), Err(format!("crashed the controller task ({e})"))),
    };

    let checked = outcome.and_then(|()| check_response(&stdout));
    match checked {
        Ok(status) => {
            log.status = status;
            ([(header::CONTENT_TYPE, "application/json")], stdout).into_response()
        }
        Err(reason) => {
            let tail = &stderr[stderr.len().saturating_sub(STDERR_LOG_TAIL)..];
            log.module_stderr = Some(String::from_utf8_lossy(tail).into_owned());
            let msg = format!("The {name} module failed: {reason}.");
            log.reason = Some(reason);
            reply(StatusCode::OK, "internal_error", &msg)
        }
    }
}

/// Runs one module to completion or to the backstop. Returns its stdout, stderr and
/// Ok if it exited 0, or the failure reason.
async fn run_module(
    path: PathBuf,
    root: PathBuf,
    input: String,
    backstop: Duration,
    _permit: OwnedSemaphorePermit,
) -> (Vec<u8>, Vec<u8>, Result<(), String>) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let spawned = Command::new(&path)
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0) // own group, so the backstop can kill the module's children too
        .spawn();
    let mut child = match spawned {
        Ok(c) => c,
        Err(e) => return (out, err, Err(format!("could not be started ({e})"))),
    };
    let pgid = child.id().map(|id| Pid::from_raw(id as i32));

    let mut child_in = child.stdin.take().expect("stdin is piped");
    // Write in its own task while we read, so a module that writes before reading can't deadlock.
    tokio::spawn(async move {
        let _ = child_in.write_all(input.as_bytes()).await; // dropping child_in sends EOF
    });
    let mut child_out = child.stdout.take().expect("stdout is piped");
    let mut child_err = child.stderr.take().expect("stderr is piped");

    let finished = timeout(backstop, async {
        let (_, _, status) =
            tokio::join!(child_out.read_to_end(&mut out), child_err.read_to_end(&mut err), child.wait());
        status
    })
    .await;

    let result = match finished {
        Err(_) => {
            if let Some(pgid) = pgid {
                let _ = killpg(pgid, Signal::SIGKILL);
            }
            let _ = child.wait().await; // reap, so no zombie is left
            Err(format!("did not answer within {} s", backstop.as_secs()))
        }
        Ok(Err(e)) => Err(format!("could not be waited on ({e})")),
        Ok(Ok(s)) if s.success() => Ok(()),
        Ok(Ok(s)) => match (s.code(), s.signal()) {
            (Some(c), _) => Err(format!("exited with code {c}")),
            (_, Some(sig)) => Err(format!("was killed by signal {sig}")),
            _ => Err("exited abnormally".into()),
        },
    };
    (out, err, result)
}

/// Checks a module's stdout is a spec response. Returns its status.
fn check_response(stdout: &[u8]) -> Result<&'static str, String> {
    let Ok(Value::Object(m)) = serde_json::from_slice::<Value>(stdout) else {
        return Err("returned invalid JSON".into());
    };
    let status = m.get("status").and_then(Value::as_str).unwrap_or_default();
    let Some(status) = STATUSES.iter().find(|s| **s == status) else {
        return Err("returned an unknown status".into());
    };
    if let Some(missing) = FIELDS.iter().find(|f| !m.contains_key(**f)) {
        return Err(format!("is missing field {missing}"));
    }
    Ok(status)
}
