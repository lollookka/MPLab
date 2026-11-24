# Execution Controller and Language Selection

**Owner:** Group 2

The single door for running code in MPLab. It serves `POST /api/execute` ([API_SPEC.md](../API_SPEC.md)) and `GET /api/languages`, and hands each request to the module in `languages/<language>/`. Why it is built this way: [DESIGN.md](DESIGN.md).

## Build and run

Needs Rust (install for your user only, no admin rights: <https://rustup.rs>). POSIX only: Linux, macOS or WSL.

```bash
cd controller
cargo test                       # no Docker needed, uses fake modules
cargo build --release --locked
cd .. && controller/target/release/mplab-controller   # run from the repo root
```

It listens on `127.0.0.1` only and logs one JSON line per request to stderr.

| Variable | Default | Meaning |
|---|---|---|
| `MPLAB_PORT` | `8000` | Port on 127.0.0.1 |
| `MPLAB_MAX_CONCURRENT` | number of CPUs | How many modules run at once. Extra requests wait up to 30 s, then get 503 |
| `MPLAB_ROOT` | current directory | The repo root, the folder that holds `languages/` |

## Endpoints

```bash
curl -s localhost:8000/api/execute -d '{"language":"ruby","source_code":"puts 10 + 20"}'
# {"status":"success","stdout":"30\n","stderr":"","exit_code":0,"execution_time_ms":41}

curl -s localhost:8000/api/languages
# [{"value":"c","name":"C","available":false}, ... ten entries in API_SPEC §4 order]
```

- A `language` outside API_SPEC §4 (including wrong case, like `"Python"`) gets `unsupported_language`, and no module is started.
- A listed language whose module isn't connected, or whose module fails or takes over 30 s, gets `internal_error` with the reason in `stderr`.
- HTTP 200 whenever the request was well formed. 400 for a malformed request, 413 for a body over 1 MB, 503 when busy (safe to retry). Every error body still has the five spec fields.
- Set your HTTP client timeout to at least 65 s.
- CORS is open (`Access-Control-Allow-Origin: *`) for frontend development on another port.

## Connecting a language module

Add an executable file `languages/<language>/execute` to your module. Nothing in `controller/` changes; your module is live once it's merged and the VM does `git pull`. The contract (full version in DESIGN.md §5.4):

1. **Entry point:** `languages/<language>/execute`, committed executable: `git add --chmod=+x languages/<language>/execute`. Usually a two-line launcher:
   ```sh
   #!/bin/sh
   exec ruby "$(dirname "$0")/executor.rb"
   ```
2. **Input:** one JSON object on stdin, then EOF: `{"language", "source_code", "stdin"}`, all three always strings. The working directory is the repo root.
3. **Output:** one API_SPEC §2/§3 JSON object on stdout, exit code 0. Nothing else on stdout. It's passed to the caller byte for byte.
4. **Your own stderr** is logged, never shown to users. The submitted program's stderr goes in the response's `stderr` field.
5. **Limits:** enforce every API_SPEC §4 limit in your sandbox, and answer within 30 s total, or the controller kills your whole process group.
6. **Concurrency and clean-up:** many copies run at once, so use unique temp files, and name containers `mplab-…`. Leave nothing behind.
7. **Host requirements:** list what the VM needs (interpreter, Docker image and its build command) in your module README.

Check it before opening your PR:

```bash
printf '%s' '{"language":"ruby","source_code":"puts 1","stdin":""}' | languages/ruby/execute
curl -s localhost:8000/api/languages   # your language should show "available": true
```
