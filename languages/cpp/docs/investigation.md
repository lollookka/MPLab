# C++ Language Investigation

**Course:** WIF3010 Programming Language Paradigms
**Project:** MPLab — multi-language programming lab
**Owner:** Group 2 (C++ execution module)
**Toolchain:** G++ (GNU C++ compiler)

This document summarises C++ as a language-design case study and how the G++
toolchain turns source code into a running process, ending with how these facts
shape the MPLab module. The binding contract is
[`API_SPEC.md`](../../API_SPEC.md): `POST /api/execute`, the response shape
`{status, stdout, stderr, exit_code, execution_time_ms}`, and the shared
limits (10 s compile, 5 s run, 256 MB memory, 64 KB output cap, network
blocked).

---

## 1. Programming Paradigm and Characteristics

C++ is deliberately **multi-paradigm**: it does not force one style of
problem-solving, but offers several that can be freely mixed, each running at
machine speed:

| Paradigm | C++ support | Typical features |
|---|---|---|
| Imperative | Statements that mutate state | Assignment, control flow, loops |
| Procedural | Functions as units of work | Free functions, overloading, default arguments |
| Object-oriented | Classes and inheritance | Encapsulation, virtual functions, polymorphism |
| Generic | Templates | Function/class templates, compile-time code generation |
| Functional (partial) | Since C++11 | Lambdas, `constexpr`, algorithms on ranges |

```cpp
int sum_of_squares(const std::vector<int>& v) {   // procedural + OOP (std::vector)
    int total = 0;
    std::for_each(v.begin(), v.end(),             // generic (template algorithm)
        [&](int x) { total += x * x; });          // functional (lambda + capture)
    return total;
}
```

Key characteristics:

- **Statically typed.** Every expression has a type fixed at compile time; the
  compiler uses types both to catch errors early and to select the exact
  machine instructions to emit.

- **Ahead-of-time (AOT) compiled.** The whole program is translated to native
  machine code before it ever runs, so startup is essentially instant —
  important for MPLab, where each submission pays the compile cost once and
  then executes.

- **RAII (Resource Acquisition Is Initialization).** C++'s central
  resource-management idiom: acquiring a resource (memory, file handle, lock)
  happens in a constructor, and releasing it happens in the destructor, which
  runs deterministically at the end of the object's lifetime — so cleanup
  cannot be forgotten the way it can with manual `malloc`/`free` pairs:

  ```cpp
  { std::vector<int> data(1000); }   // freed automatically at the closing brace
  ```

- **Zero-overhead abstraction principle.** The language's stated design rule:
  you do not pay (in time or memory) for an abstraction you do not use, and an
  abstraction you *do* use cannot be beaten by hand-writing the lower-level
  equivalent — the reason C++ is chosen where performance matters, and why it
  looks more complicated than languages that trade speed for simplicity.

Other characteristics worth noting: C++ is a superset of C in spirit, has no
garbage collector by design (RAII replaces it), and its large standard library
(`std`) covers containers, algorithms, strings, I/O streams, threads, and smart
pointers that make manual `new`/`delete` largely unnecessary.

---

## 2. Compiler Implementation and the Source-Code Execution Process

C++ has **no interpreter** in the G++ workflow: the driver `g++` orchestrates
a four-stage pipeline, and understanding it explains how the module separates
compile-time from runtime failures.

### 2.1 The four stages

| Stage | Tool | Input | Output | What happens |
|---|---|---|---|---|
| 1. Preprocessing | `cpp` | `.cpp` | `.i` | Textual substitution: `#include` pastes headers in, `#define` macros are expanded, conditionals (`#ifdef`) resolved, comments removed. Purely textual — no type knowledge yet. |
| 2. Compilation | `cc1plus` (G++ internal) | `.i` | `.s` (assembly) | Lexing, parsing, semantic analysis and type checking, optimisation, lowering to assembly for the target CPU. **Almost all real compile errors come from here.** |
| 3. Assembly | `as` | `.s` | `.o` (object file) | Assembly text becomes machine code plus symbol tables: a relocatable object module that is *almost* final, but with symbolic references still unresolved. |
| 4. Linking | `ld` | `.o` files + libraries | executable | Combines object files, resolves cross-file symbols, pulls in library code (e.g. `libstdc++` for `std::cout`), performs final address fix-up, writes the executable. |

### 2.2 How the driver orchestrates it

`g++ file.cpp -o prog` runs all four stages back to back and deletes the
intermediate files by default; each stage can also be invoked separately:

```bash
g++ -E file.cpp -o file.i   # stop after preprocessing
g++ -S file.cpp -o file.s   # stop after compiling to assembly
g++ -c file.cpp -o file.o   # stop after assembling
g++ file.cpp -o prog        # full pipeline to executable
```

A translation unit can already produce object code, but any function it
*calls* but does not *define* — for example `std::cout <<` — is left as an
unresolved symbol only the linker can satisfy, so a mis-spelled name that
exists nowhere fails at stage 4 (**linker error**), still before the program runs.

### 2.3 Compilation vs interpretation

| Aspect | Compiled (C++) | Interpreted (e.g. Python, JavaScript) |
|---|---|---|
| When errors are found | Mostly before running | Mostly at runtime |
| What is distributed | Native machine code | Source or bytecode + interpreter |
| Startup cost | Near zero; pay compile cost once | Pay parse/run cost every run |
| Runtime type decisions | Minimal (see §4 on RTTI) | Pervasive |

In an interpreted language, a type error on a line that never executes may
never be detected at all; in C++ the same error stops the build. That is why
MPLab runs a separate compile phase with its own 10-second budget before the
5-second run phase — the two have genuinely different failure modes.

---

## 3. Data Types, Variable Binding, and Scope

### 3.1 Fundamental types

C++ has a fixed set of built-in fundamental types plus user-defined types built
from them:

| Category | Types |
|---|---|
| Integer | `bool`, `char`, `short`, `int`, `long`, `long long` (each with `unsigned` variants) |
| Floating point | `float`, `double`, `long double` |
| No value | `void` |
| Pointer / reference | `T*`, `T&`, `T&&` |
| User-defined | `class`, `struct`, `union`, `enum`, `enum class`, `std::string`, containers |

Note that, unlike Java, the sizes of `int`, `long`, etc. are only *minimums*
guaranteed by the standard and are platform-dependent; query them with
`sizeof(T)` or `std::numeric_limits<T>`.

### 3.2 Type deduction: `auto`

Since C++11 the compiler can deduce a variable's type from its initialiser.
The type is still fixed at compile time — `auto` is *not* dynamic typing:

```cpp
auto x = 42;          // int — decided at compile time, not dynamic typing
auto v = std::vector<std::pair<int, std::string>>{};  // saves a huge type name
```

### 3.3 Static vs dynamic binding

C++ performs *static binding* by default: calls and variable uses are resolved
at compile time from the declared types. *Dynamic binding* exists in exactly
one well-defined place — a `virtual` function called through a base-class
pointer or reference, resolved at runtime via the **vtable** (a per-class table
of function pointers):

```cpp
struct Shape { virtual double area() const; };
struct Circle : Shape { double area() const override; };

Shape* s = new Circle;
s->area();   // dispatches to Circle::area at RUNTIME (dynamic binding)
```

Non-virtual functions, overloads, and templates are bound statically. Templates
are resolved at compile time, re-generating specialised code per instantiated
type — compile-time, not runtime, polymorphism.

### 3.4 Scope and lifetime

C++ scope can be organised by *where* a name lives and *how long* the object
does:

| Kind | Declared with | Lifetime |
|---|---|---|
| Block (local) scope | inside `{ }` | ends when the block exits (stack) |
| Global / namespace scope | at file top level | whole program |
| Dynamic (heap) storage | `new` / smart pointers | until `delete` or last owner |
| Static storage | `static`, thread-local | whole program / one thread |

**Stack objects** are created and destroyed automatically as functions run and
return — this is what makes RAII work for locals: the string below is freed at
the closing brace, even if an exception unwinds `f`.

**Heap objects** live until explicitly released; modern C++ wraps them in
owners (`std::unique_ptr`, `std::shared_ptr`) that are themselves stack
objects, so heap memory is reclaimed by RAII rather than by a garbage
collector.

### 3.5 Linkage and name mangling

**Linkage** describes whether a name can be seen from other translation units:
*internal* (file-only: `static` at file scope, anonymous namespaces),
*external* (visible across files: the default for functions and globals), or
*none* (locals).

C++ supports **function overloading**, so the linker cannot use plain source
names like `add` — several functions share it. The compiler therefore encodes
the full signature into a *mangled* name:

```cpp
int add(int, int);        // mangled to e.g. _Z3addii
int add(double, double);  // mangled to e.g. _Z3adddd  — different symbol!
```

G++ follows the Itanium C++ ABI mangling scheme (`c++filt` or `nm` decodes
it). For our module: linker error messages contain mangled names, and
`extern "C"` disables mangling so C libraries can be linked.

---

## 4. Type Checking and Error Handling

### 4.1 Static, strong checking

C++ type checking happens almost entirely at **compile time** and is *strong*:
implicit conversions between unrelated types are rejected, and template
arguments are checked at instantiation. `const`-correctness, overload
resolution, and access control (`private`/`protected`) are all enforced by the
compiler rather than by runtime checks.

```cpp
int x = 3.14;               // allowed: narrowing conversion (compiler warns)
int y = std::string("hi");  // REJECTED: no conversion exists — error before the build finishes
```

A large share of bad C++ submissions therefore never reach the run phase; G++
reports errors with file/line/column, an error code, and the offending source
line, which our module can forward verbatim in `stderr`.

### 4.2 Runtime type information (RTTI)

A small amount of type information survives to runtime, only when needed:
`dynamic_cast<T*>(p)` safely downcasts a polymorphic pointer/reference
(returning `nullptr`, or throwing for references, if the object is not really a
`T`), and `typeid(x)` returns a `std::type_info` describing the dynamic type:

```cpp
if (Circle* c = dynamic_cast<Circle*>(get_shape())) { /* runtime-true only for circles */ }
```

RTTI works only for *polymorphic* types (those with at least one virtual
function) and can be disabled with `-fno-rtti`, reinforcing that C++'s default
model is static, not dynamic.

### 4.3 Exceptions vs error codes

C++ offers two mainstream error-signalling styles:

**Exceptions** — `throw` unwinds the stack, running every destructor on the way
(grafting naturally onto RAII), until a matching `catch`:

```cpp
try {
    load_config("app.cfg");
} catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << '\n';   // non-zero exit code → runtime_error
}
```

**Error codes / status returns** — the function returns a value the caller must
check (`std::optional`, `std::expected` in C++23, plain `int` codes): faster and
explicit, but easy to forget to check.

The uncaught-exception rule matters for MPLab: if `main` lets an exception
escape, `std::terminate` aborts with a nonzero exit status (typically
`SIGABRT`, shell exit code 134) and "terminate called after throwing an
instance of ..." appears on `stderr` — classified as `runtime_error`, not
`compile_error`, because the program *started* before failing.

### 4.4 Undefined behaviour — a language-level risk

C++ prioritises performance over safety, so some constructs have **undefined
behaviour (UB)**: the standard imposes no requirement at all on what happens.

```cpp
int a[3];
int i = a[5];          // reading past the end: UB
if (*p == 0) {}        // reading an uninitialised pointer p: UB
```

UB is not a "crash at line N" guarantee — the optimiser may assume it cannot
happen and delete surrounding code, so symptoms can appear far from the cause.
Sanitizer tooling (`-fsanitize=address`, `-fsanitize=undefined`) can catch many
UB patterns, but that is runtime instrumentation with a performance cost, not
part of the base language.

For an execution sandbox this means a program that prints "hello" and exits 0
can still be harbouring UB. The module's job is containment (memory limit,
output cap, time limit), not proof of correctness; the diagnostics service can
offer sanitizer builds to lecturers as a separate option.

---

## 5. Runtime Requirements and Dependencies

### 5.1 What a compiled C++ binary depends on

Even a trivial `std::cout << "hi"` program is not self-contained. On Linux it
normally depends on:

| Library | Provides |
|---|---|
| `libstdc++.so` | C++ standard library: streams, containers, exceptions, `std::` machinery |
| `libm.so` | Math functions |
| `libgcc_s.so` | Compiler runtime helpers (128-bit arithmetic, unwinding) |
| `libc.so` (glibc) | The C library beneath everything: syscalls, `printf`-family, allocator glue |

`ldd prog` lists the shared libraries a binary needs at load time:

```bash
$ ldd prog
    linux-vdso.so.1
    libstdc++.so.6  => /usr/lib/.../libstdc++.so.6
    libgcc_s.so.1   => /lib/.../libgcc_s.so.1
    libc.so.6       => /lib/.../libc.so.6
    /lib64/ld-linux-x86-64.so.2   (the dynamic loader itself)
```

### 5.2 Static vs dynamic linking

**Dynamic linking** (G++ default): the executable stores only *references* to
the `.so` libraries, which the loader (`ld-linux`) maps in at process start —
small binaries and shared code across processes, but the exact library versions
must exist on the run machine, a portability concern if the sandbox image
differs from the build image.

**Static linking** (`g++ -static`): the needed library code is copied into the
executable — one self-contained file with no loader dependency, at the cost of
a larger binary. Useful when the run environment may not carry matching shared
libraries.

```bash
g++ file.cpp -o prog            # dynamic (default)
g++ -static file.cpp -o prog    # static: ldd prints "not a dynamic executable"
```

### 5.3 Minimal execution environment for the module

For MPLab the sandbox needs:

1. **Build time (≤ 10 s):** the `g++` binary plus its headers
   (`/usr/include/c++/...`), since even a trivial program links against the
   C++ runtime parts it uses.
2. **Run time (≤ 5 s):** `libstdc++` and glibc for dynamic linking, or nothing
   beyond the kernel ABI if the module compiles with `-static`.
3. **No network** — the API spec blocks it; C++ sockets exist, so sandbox
   policy must deny them regardless of what the program requests.
4. **Memory ceiling** — 256 MB is enforced by the controller's sandbox, not by
   the language; an allocation past it typically yields `std::bad_alloc` or an
   OOM kill, either of which the module reports as `runtime_error`.

A practical deployment choice: build submissions inside the same container
image that runs them, so compile-time and runtime library versions always
match, and prefer `-static` when images may diverge.

---

## 6. Relevance to the MPLab Module

This section maps the language facts above onto the runner design under
`API_SPEC.md`.

**Why compile-then-execute in two separate steps.** C++ is AOT-compiled with a
heavyweight compile phase (§2) that can itself fail for ordinary student
mistakes — a missing semicolon, a type mismatch, an undeclared name. Separating
the phases lets the module apply the **10 s compile limit** to stage 1–4 work
only and the **5 s run limit** to the child process only (matching the spec's
rule that the two budgets are counted separately), return
`execution_time_ms` for the *run* only, and keep compiler diagnostics in
`stderr` exactly as the spec directs for `compile_error`.

**How statuses map to the pipeline stages (§2):**

| Pipeline point where failure occurs | `status` to return | Why |
|---|---|---|
| Preprocess / compile / assemble / link (stages 1–4), or the 10 s compile limit is exceeded | `compile_error`, message in `stderr`, `exit_code = null`, `execution_time_ms = null` | The program never existed as a runnable process. |
| Process starts, then exits nonzero, is killed by the 256 MB limit, throws an uncaught exception, or crashes (segfault `139`) | `runtime_error`, real `exit_code`, message in `stderr` | The program *ran* — failure is a program property, not a build property. |
| Process exceeds the 5 s run limit | `timeout`, `exit_code = null`, `execution_time_ms =` the time limit | Distinct status per spec. |
| `g++` itself missing, sandbox setup fails | `internal_error` | System problem, not the user's code. |

Two compile-side details matter in practice: a **compile timeout is a
`compile_error`, not a `timeout`** (the spec defines `timeout` as a *run-time*
limit, and a program that blows the 10 s compiler budget failed *before
running*), and **linker errors are `compile_error`** too — undefined references
occur at stage 4, but before any process exists.

**Runtime classification (§4–§5).** Because C++ exceptions terminate with a
nonzero exit code and print to `stderr`, and the memory cap is enforced
externally, the module does not need to parse exception text — the exit code
plus the sandbox kill reason suffices to classify `success` / `runtime_error` /
`timeout`.

**Sandbox obligations carried by the module.** Whatever helper compiles and
runs submissions must enforce the output cap (truncate at 64 KB — C++ can print
in a tight loop faster than a reader drains a pipe) and network isolation. Both
belong to the runner, not the compiler: the language provides no built-in limit
mechanism, which is precisely why the shared API contract exists.

---

## Implementation notes for the MPLab module

The built module (`languages/cpp/`) follows directly from the sections above:

1. **Compile and run are two separate processes** (§2): the executor invokes
   `g++ main.cpp -o prog` as one child process, and only if that exits 0
   spawns a second process for the binary. This mirrors the pipeline
   description in §2 and is what makes `compile_error` (g++ failed — no
   program ever ran) vs `runtime_error` (the binary started, then exited
   nonzero or was killed) meaningful statuses rather than guesses.

2. **The sandbox is kernel rlimits plus process groups, not Docker** (§5.3):
   the compile child runs under `RLIMIT_AS` 2 GB / `RLIMIT_CPU` 11 s /
   `RLIMIT_CORE` 0, the run child under `RLIMIT_AS` 256 MB / `RLIMIT_CPU` 6 s
   / `RLIMIT_CORE` 0; every process starts in its own session so a timeout
   can SIGKILL the whole process group; network is blocked best-effort with
   `unshare --net` when it works on the host. This was chosen because it
   needs nothing beyond the toolchain itself and keeps every enforcement
   step explainable (Group 7 instead uses Docker per-run containers).

3. **The executor glue is Python** because the executor never interprets
   C++ — it only orchestrates processes: writing the source file, spawning
   `g++` and the binary, piping stdin/stdout, clipping output to 64 KB, and
   mapping exit codes (including 128 + signal) onto the API_SPEC statuses.
   See the README section "Why the executor is written in Python".

4. **Residual limits of this sandbox** (same-uid isolation only, no
   setuid/chroot; a grandchild that calls `setsid()` escapes the
   process-group kill) are documented in the README's sandbox notes.

---

*End of investigation. Written by Group 2 for WIF3010; original prose based on standard C++ (ISO/IEC 14882) and the GNU Compiler Collection's documented pipeline behaviour.*
