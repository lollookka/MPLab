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

## Concept demonstration (Task D)

Five self-contained example programs live in `examples/`, one per course
concept. Each compiles with plain `g++ file.cpp` (GCC 13.3.0, C++17), runs in
milliseconds, and explains the concept in comments and output at the point it
happens — so pasting one into the MPLab editor doubles as a lesson.

### 1. `examples/01_data_types.cpp` — Static typing & data types

`sizeof` and `numeric_limits` for `int`/`double`/`char`/`bool`, integer vs
floating division, implicit narrowing, and a stdin integer echoed with its
type's range. Key output (stdin `42`):

```
int    : 4 bytes, range -2147483648 .. 2147483647
7 / 2      = 3          (int / int -> int, remainder dropped)
7.0 / 2.0  = 3.5      (double / double -> double)
double 3.14 -> int : 3 (fractional part lost, no error)
You typed: 42
It fits in 'int', whose range is -2147483648 .. 2147483647
```

*Look for:* the same `/` operator gives `3` or `3.5` depending on operand
types — the type, not the operator, decides. Sample stdin: type `42` in
MPLab's input box.

### 2. `examples/02_scope_lifetime.cpp` — Scope, lifetime & RAII

Nested-block shadowing, a stack int vs an explicitly `delete`d heap int, and
an RAII `Resource` class printing construction/destruction order. Key output
(stdin `db-connection`):

```
inside  block, x = inner (this x shadows the outer one)
back outside, x = outer (inner x is destroyed, scope ended)
  [acquire] resource 'db-connection' opened
  [acquire] resource 'spare-conn' opened
  [release] resource 'spare-conn' closed
  [release] resource 'db-connection' closed
```

*Look for:* destruction in reverse construction order — the destructor runs
at the closing brace with no `delete` anywhere. Sample stdin: type
`db-connection`.

### 3. `examples/03_type_checking.cpp` — Static type checking

`const`-correctness, `auto` deduction printed via `typeid`, and a
commented-out block that would NOT compile with the real GCC 13.3.0 error
texts (e.g. `int x = "hello";` →
`error: invalid conversion from 'const char*' to 'int' [-fpermissive]`).
Key output:

```
auto 42    -> i
auto 4.5   -> d
auto 'c'   -> c
auto string-> NSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
int(42) + double(4.5) = 46.5
```

*Look for:* the commented-out lines — uncommenting them makes g++ reject the
file; that rejection is the type system. (`i`/`d`/`c` are GCC's mangled names
for int/double/char.)

### 4. `examples/04_parameter_passing.cpp` — Parameter passing

By-value vs by-reference vs by-const-reference on one `std::string`, plus
`std::move` handing over the string's buffer. Key output:

```
after by_value : "hello"  (copy was changed, original untouched)
after by_ref   : "hello (modified in callee)"  (callee modified the caller's object)
moved-from size: 0  moved-to size: 10
```

*Look for:* identical callee code, three different effects on the caller's
object; the `const&` version literally cannot compile if it modifies.

### 5. `examples/05_dynamic_binding.cpp` — Binding: static vs dynamic

One `Dog` object called through an `Animal*`: the non-virtual call is bound
statically by the declared type (base version runs, Dog's never reached), the
virtual call is bound dynamically via the vtable (Dog's override runs). Key
output:

```
non-virtual call (static binding, at compile time):
  Animal::speak_nonvirtual (bound by declared type)
virtual call (dynamic binding, at run time):
  Dog::speak_virtual (reached via the vtable)
through Dog*, Dog::speak_nonvirtual now runs:
  Dog::speak_nonvirtual (never reached via Animal*)
```

*Look for:* the same pointer `a` of the same object producing two different
functions — only the `virtual` one consults the object's real type at run
time.

**Running these in MPLab:** paste any file into the editor with language
`cpp` and press Run. `01` and `02` read stdin: put a sample integer (e.g.
`42`) or a sample resource name (e.g. `db-connection`) in the input box
before running; the other three need no input.
