# MPLab Execution API Specification

Maintained by Group 1 (Shared API specification and integration standards).

**Version:** 1.0 (19 November 2026)

All groups must follow this specification. To suggest a change, open an issue or contact the Group 1 leader.

## 1. Endpoint and Request

**Endpoint:** `POST /api/execute`

**Request body (JSON):**

| Field | Type | Required | Meaning |
|---|---|---|---|
| `language` | text | yes | Which language to run (see Section 4 for allowed names) |
| `source_code` | text | yes | The program code |
| `stdin` | text | no (default `""`) | Input given to the program |

**Example request:**

```json
{
  "language": "c",
  "source_code": "#include <stdio.h>\nint main(){ printf(\"hi\"); return 0; }",
  "stdin": ""
}
```

## 2. Response

**Response body (JSON):**

| Field | Type | Meaning |
|---|---|---|
| `status` | text | Result summary (see Section 3) |
| `stdout` | text | Normal program output |
| `stderr` | text | Error messages (compiler or runtime) |
| `exit_code` | number or `null` | Program's exit code. `null` if the program never ran |
| `execution_time_ms` | number or `null` | How long the program ran, in milliseconds. `null` if it never ran |

**Example response:**

```json
{
  "status": "success",
  "stdout": "hi",
  "stderr": "",
  "exit_code": 0,
  "execution_time_ms": 12
}
```

## 3. Status Values

Every module must use exactly one of these values for `status`:

| Status | Meaning |
|---|---|
| `success` | The program ran and finished with exit code 0 |
| `compile_error` | The code was rejected before it ran (compile error or syntax error, for any language) |
| `runtime_error` | The program started running, then crashed or finished with a non-zero exit code (this includes going over the memory limit) |
| `timeout` | The program ran longer than the time limit and was stopped |
| `unsupported_language` | The `language` value is not in the list in Section 4 |
| `internal_error` | Something went wrong in the system itself, not in the submitted code |

**Field values for each status:**

| Status | `exit_code` | `execution_time_ms` |
|---|---|---|
| `success` | `0` | number |
| `runtime_error` | the real exit code | number |
| `timeout` | `null` | the time limit |
| `compile_error`, `unsupported_language`, `internal_error` | `null` | `null` |

For `compile_error` and `runtime_error`, the error message goes in `stderr`.

## 4. Language Names and Limits

**Allowed `language` values** (lowercase, exactly as written):

| Group | Language | `language` value |
|---|---|---|
| 1 | C | `c` |
| 2 | C++ | `cpp` |
| 3 | Java | `java` |
| 4 | Python | `python` |
| 5 | JavaScript | `javascript` |
| 6 | Go | `go` |
| 7 | Ruby | `ruby` |
| 8 | PHP | `php` |
| 9 | Haskell | `haskell` |
| 10 | Prolog | `prolog` |

Any other value returns `unsupported_language`.

**Execution limits** (same for every language):

| Limit | Value | If exceeded |
|---|---|---|
| Compile time | 10 seconds | `compile_error` with a message in `stderr` |
| Run time | 5 seconds | `timeout` |
| Memory | 256 MB | `runtime_error` |
| Output size | 64 KB | Extra output is cut off |
| Network access | Blocked | — |

Compile time and run time are counted separately. `execution_time_ms` measures the run time only.

## 5. Integration Rules

### 5.1 Folder Structure

```
mplab/
├── README.md          ← how to use the repo
├── API_SPEC.md        ← this document
├── frontend/          ← Group 3 (web interface and code editor)
├── controller/        ← Group 2 (execution controller and language selection)
├── languages/         ← one folder per language, named with the Section 4 values
│   ├── c/             ← Group 1
│   ├── cpp/           ← Group 2
│   ├── java/          ← Group 3
│   ├── python/        ← Group 4
│   ├── javascript/    ← Group 5
│   ├── go/            ← Group 6
│   ├── ruby/          ← Group 7
│   ├── php/           ← Group 8
│   ├── haskell/       ← Group 9
│   └── prolog/        ← Group 10
├── services/          ← shared platform features (lecturer, assessment, submissions, diagnostics, comparison)
├── tests/             ← Group 7 (integration testing)
└── docs/              ← Group 10 (documentation and deployment)
```

Each group only edits its own folders unless agreed with the owning group.

### 5.2 How a Language Module Plugs In

1. The website sends a request (Section 1) to `POST /api/execute`.
2. The execution controller (Group 2) reads the `language` field and passes the request to the matching module in `languages/`.
3. The language module compiles or interprets and runs the code inside the sandbox, using the limits in Section 4.
4. The module returns a response in the Section 2 format, using a status from Section 3.

Every language module must accept the same request and return the same response format.

### 5.3 Branches

- `main` is the official version. Nobody pushes to `main` directly.
- Work on your own branch, named `group<number>/<short-description>`.
  Examples: `group1/c-executor`, `group4/lecturer-page`

### 5.4 Pull Requests

- All changes reach `main` through a pull request (PR).
- A PR that only changes files inside your own `languages/<your-language>/` folder can be approved by a member of your own group.
- Any PR that changes shared files (`controller/`, `frontend/`, `services/`, `tests/`, `docs/`, or this specification) needs approval from at least one member of another group.
- Start PR titles with your group number, for example `G1: Add GCC compile step`.
