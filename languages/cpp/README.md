# C++ Module

**Owner:** Group 2

**Compiler / interpreter:** G++

This module receives a request in the format described in [API_SPEC.md](../../API_SPEC.md) (Section 1) and returns a response in the Section 2 format.

## Toolchain installation

G++ was **already installed** in the current environment, so no package-manager
installation was required. If it had been missing, the install command would be:

```bash
sudo apt-get install -y g++
# or, when already root:
apt-get install -y g++
```

Verification output (`g++ --version`, with `which g++` → `/usr/bin/g++`):

```
g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0
Copyright (C) 2023 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
```

**Exact version string:** `g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` (GCC 13.3.0, Ubuntu 24.04).

## Verifying the toolchain

Three tiny demo programs live outside the repo in `/tmp/opencode/taskb/`
(no demo sources or binaries are kept inside the repo).

### 1. `hello_ok.cpp` — success path

A minimal program printing `hello from C++`. Compiled with
`g++ hello_ok.cpp -o hello_ok` and executed directly.

| Step | Result |
|---|---|
| Compile exit code | `0` |
| Run exit code | `0` |
| stdout | `hello from C++` |
| stderr | *(empty)* |

Expected module mapping: `status = success`, `exit_code = 0`, `stdout` captured,
empty `stderr`.

### 2. `syntax_err.cpp` — compile error path

The program is missing a semicolon after `int x = 5`, so `g++` refuses to
produce a binary.

| Step | Result |
|---|---|
| Compile exit code | `1` (non-zero) |
| stderr | `/tmp/opencode/taskb/syntax_err.cpp:5:5: error: expected ‘,’ or ‘;’ before ‘std’` (plus source line context from GCC) |
| Binary produced | none |

Expected module mapping: `status = compile_error`, GCC diagnostics returned in
`stderr`, no run attempted.

### 3. `runtime_err.cpp` — runtime error path

The program compiles cleanly but throws an uncaught `std::runtime_error("boom")`.

| Step | Result |
|---|---|
| Compile exit code | `0` |
| Run exit code | `134` (SIGABRT, core dumped) |
| stdout | *(empty)* |
| stderr | `terminate called after throwing an instance of 'std::runtime_error'` / `  what():  boom` |

Expected module mapping: `status = runtime_error`, non-zero `exit_code` (e.g.
`134`, or `139` for a null-pointer crash), crash message captured in `stderr`.

## Why the executor is written in Python

The executor is glue, not language work: it parses one JSON request, spawns
`g++`, sets resource limits, reads two pipes, and maps exit codes to statuses.
The C++ side of the job — compiling and executing student code — is done by
G++ and the Linux kernel, so the plumbing language is irrelevant to what the
module runs. Python 3 is already guaranteed on the VM (Group 4's CPython is a
platform toolchain), and the standard library covers everything the executor
needs (`json`, `subprocess`, `resource`) with zero dependencies. A C++
executor would need a third-party JSON library and a build step to do the
same plumbing. The controller (DESIGN.md §5.4) accepts the module as any
executable with a shebang — it just runs `languages/cpp/execute` — so nothing
downstream cares either. The C++ that matters for the course is the compiled
student code, the G++ pipeline, and the example programs, not the glue.

## Who needs to install G++

Only the machine that runs this module — the faculty VM — needs G++, and it
is installed once with the command in "Toolchain installation" above. Group
2's README documents this per the module contract (DESIGN.md §5.4 point 8:
each module lists its own host requirements). No other group needs G++ for
their own modules — each module lists its own toolchain (Ruby for Group 7,
CPython for Group 4, GHC for Group 9), and Group 10's deployment
coordination gathers these — though any group may install G++ if they want
to run this module's tests locally for integration testing. The installed
version matters beyond "present": student binaries link against the
`libstdc++` shipped with that G++, which is why the verified version string
is pinned above. For contrast: Group 7's Docker approach bundles Ruby
inside their image but requires Docker instead — every module needs exactly
one thing installed on the VM.

## Execution limits

Per [API_SPEC.md](../../API_SPEC.md) (Section 4), identical for every language:

| Limit | Value | If exceeded |
|---|---|---|
| Compile time | 10 seconds | `compile_error` with a message in `stderr` |
| Run time | 5 seconds | `timeout` |
| Memory | 256 MB | `runtime_error` |
| Output size | 64 KB | Extra output is cut off |
| Network access | Blocked | — |

Compile time and run time are counted separately; `execution_time_ms` measures
run time only.

## Execution module

Implemented as one file plus a launcher, satisfying the module contract of
[controller/DESIGN.md](../../controller/DESIGN.md) §5.4:

| File | Role |
|---|---|
| `executor.py` | The whole module: reads one JSON request `{language, source_code, stdin}` on stdin, compiles with `g++`, runs the binary, writes exactly one JSON response in the API_SPEC §2/§3 shape on stdout, exits 0. The module's own diagnostics go to stderr. |
| `execute` | Two-line `#!/bin/sh` launcher for `executor.py` — the controller's entry point at `languages/cpp/execute` |

**Contract summary:** `language` must be `"cpp"` (case-sensitive; anything
else → `unsupported_language`). Statuses: `success`, `compile_error`,
`runtime_error`, `timeout`, `unsupported_language`, `internal_error`;
`exit_code`/`execution_time_ms` are `null` for `compile_error`,
`unsupported_language` and `internal_error`. A program killed by a signal
reports `exit_code = 128 + signal` (DESIGN.md §9: SIGSEGV → 139,
SIGABRT → 134) with a `killed by signal SIGxxx` line appended to stderr; a
run timeout reports `exit_code = null` and `execution_time_ms = 5000`
(10 s compile, 5 s run limits per API_SPEC §4). stdout and stderr are
truncated to 64 KB and decoded as UTF-8 with errors replaced. A malformed
request envelope (bad JSON, wrong types, missing `language`/`source_code`)
gets `internal_error`; a missing `stdin` defaults to `""`.

**Sandbox:** each request gets a per-call temp directory (`mplab-cpp-*`,
removed afterwards). The compile step runs under rlimits (2 GB address
space, CPU limit 11 s, no core dumps) with `TMPDIR` pointed into the temp
dir; the program runs under rlimits (256 MB address space, CPU limit 6 s,
no core dumps), each process in its own session so a timeout can SIGKILL
the whole process group. Network is blocked best-effort with
`unshare --net` when it works on this host; otherwise the run proceeds
directly and the module writes one warning line to its own stderr (the
controller's log channel, DESIGN.md §5.7). Documented residuals: isolation
is same-uid only (no setuid/chroot), a grandchild that calls `setsid()`
escapes the process-group kill, and captured output is buffered by
`communicate()` (bounded by the 64 KB truncation and the rlimits). No
Docker is needed.

**Host requirements:** Python 3, g++, and optionally `unshare`
(util-linux) for network blocking.

**Committing the launcher's execute bit:**

```bash
git add --chmod=+x languages/cpp/execute
```

**Tests** (stdlib `unittest`, from the repo root):

```bash
python3 -m unittest discover -s languages/cpp/tests -v
```
