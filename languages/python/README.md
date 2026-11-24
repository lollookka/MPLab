# Python Execution Module — MPLab

**Course:** WIF3010 Programming Language Paradigms  
**Project:** MPLab — Multi-Language Programming Lab  
**Group:** Group 4 — Team Fahh  
**Programming Language:** Python  
**Interpreter:** CPython  
**Academic Session:** Semester 1, 2026/2027  
**Institution:** Universiti Malaya

## 1. Overview

The Python Execution Module is developed by Group 4 as part of the MPLab multi-language programming platform.

MPLab allows users to write, execute and test programs in ten programming languages. Group 4 is responsible for implementing the Python language module and the shared Lecturer Question Management component.

The Python module receives execution requests from the execution controller developed by Group 2. It is responsible for executing Python source code in an approved isolated environment, capturing program output and errors, and returning execution results according to the official MPLab API specification maintained by Group 1.

This repository folder contains the planned implementation, documentation, programming demonstrations and testing resources for Python Tasks A–E.

**Current Status:** Initial project setup. Implementation and testing are pending.

## 2. Group Members and Responsibilities

Group 4 consists of six members.

| No. | Member | Assigned Responsibilities |
|---|---|---|
| 1 | Shaif Ahmad Bin Tufail Ahmad | Task E Integration and Testing, Project Coordination, GitHub Management |
| 2 | Keertti A/P Rasanran | Task A Python Language Investigation, Lecturer Question Creation and Editing |
| 3 | Sanchay A/L Ravindran | Task C Output and Error Handling, Lecturer Question Retrieval and Deletion |
| 4 | Yashveena S. Jayaganthan | Task D Five Python Demonstrations, Task E Testing |
| 5 | Maredu Anes | Task B CPython Installation, Task C Interpreter Execution |
| 6 | Liang Bohao | Lecturer Question Validation, Storage and Quality Assurance |

Group 4 has two main development components:

**Python Language Module (Tasks A–E)**  
Official folder: `languages/python/`

**Lecturer Question Management**  
Official folder: `services/lecturer/`

Liang Bohao's primary assignment is the Lecturer Question Management component. The component is documented separately under `services/lecturer/`.

## 3. Planned Folder Structure

```text
languages/python/
├── README.md
├── execute
├── executor.py
├── result_handler.py
├── docs/
│   ├── investigation.md
│   └── installation.md
├── examples/
│   ├── 01_data_types.py
│   ├── 02_variable_scope.py
│   ├── 03_variable_binding.py
│   ├── 04_parameter_passing.py
│   └── 05_recursion.py
└── tests/
    └── test_python_module.py
```

The structure above is the proposed organization for the Python module. Files will be added as the assigned members complete their implementation.

### File Responsibilities

| File / Folder | Description | Responsible Member |
|---|---|---|
| `README.md` | Module overview and development documentation | Shaif |
| `execute` | Executable launcher for the controller | Maredu |
| `executor.py` | Python execution and standard input handling | Maredu |
| `result_handler.py` | Standard output, standard error and execution status handling | Sanchay |
| `docs/investigation.md` | Python language investigation (Task A) | Keertti |
| `docs/installation.md` | CPython installation and verification (Task B) | Maredu |
| `examples/` | Five Python programming concept demonstrations (Task D) | Yashveena |
| `tests/` | Execution and integration tests (Task E) | Yashveena and Shaif |

The executor file structure may be adjusted during implementation as long as the final module follows the agreed integration contract.

## 4. Requirements and Installation

### 4.1 Required Software

The proposed development environment includes:

- CPython interpreter
- Python 3.13 for preliminary local development
- A Linux environment compatible with Group 2's current controller
- An approved isolated execution environment
- Additional dependencies required by the selected sandbox

The final Python version and deployment configuration must be confirmed for the faculty VM.

### 4.2 CPython Installation Verification

After installation, the interpreter can be verified using:

```bash
python3 --version
python3 -c "print('Hello from Python')"
```

On Windows, the equivalent commands may use `python` or `py`.

Task B will document the actual installation procedure, interpreter version, configuration, successful program execution and error-handling evidence.

**Documentation:** `docs/installation.md`

## 5. Python Execution Module

### 5.1 Execution Workflow

The Python execution module is planned to perform the following operations:

1. Receive an execution request in JSON format.
2. Validate the incoming request.
3. Validate the Python source code syntax.
4. Execute the submitted program inside an approved sandbox.
5. Supply standard input when required.
6. Capture standard output and standard error.
7. Detect execution errors and timeouts.
8. Format the result according to the official API specification.
9. Return a JSON response to the execution controller.
10. Clean up temporary execution resources.

### 5.2 Integration with Group 2's Controller

Group 2's current controller implementation expects the following executable entry point:

`languages/python/execute`

The controller invokes this launcher and sends one JSON request through standard input.

The Python module must return exactly one valid JSON response through standard output.

The module must not print additional debug messages to standard output because this would interfere with JSON parsing.

The launcher must have executable permissions in the deployment environment.

Module-level diagnostics should be separated from the submitted program's standard error.

### 5.3 Planned Implementation Files

**`execute`**

An executable launcher used by the controller to start the Python execution module.

**`executor.py`**

Handles request processing, Python interpreter execution, standard input and the approved sandbox.

**`result_handler.py`**

Handles captured output, execution status classification and response formatting.

These files will be developed and integrated by Maredu and Sanchay.

## 6. API Request Format

The official MPLab execution endpoint is:

`POST /api/execute`

The HTTP endpoint belongs to Group 2's controller. Group 4's Python module receives the execution request through standard input.

### Example Request

```json
{
  "language": "python",
  "source_code": "print('Hello MPLab')",
  "stdin": ""
}
```

### Request Fields

| Field | Type | Description |
|---|---|---|
| `language` | String | Requested programming language; must be `python` |
| `source_code` | String | Python source code to execute |
| `stdin` | String | Optional standard input; defaults to an empty string |

The programming language identifier is case-sensitive.

## 7. API Response Format

The Python module must return a JSON object containing the five required fields.

### Example Successful Response

```json
{
  "status": "success",
  "stdout": "Hello MPLab\n",
  "stderr": "",
  "exit_code": 0,
  "execution_time_ms": 12
}
```

The execution time shown is illustrative and is not a measured test result.

### Response Fields

| Field | Description |
|---|---|
| `status` | Execution result |
| `stdout` | Normal program output |
| `stderr` | Error messages |
| `exit_code` | Program exit code or `null` |
| `execution_time_ms` | Program execution duration in milliseconds or `null` |

### Supported Execution Statuses

| Status | Description |
|---|---|
| `success` | Program completed successfully with exit code 0 |
| `compile_error` | Python syntax validation failed before execution |
| `runtime_error` | Program failed during execution or exceeded its memory limit |
| `timeout` | Program exceeded the execution time limit |
| `unsupported_language` | Unsupported programming language |
| `internal_error` | Internal module or platform error |

For `compile_error`, `unsupported_language` and `internal_error`, both `exit_code` and `execution_time_ms` must be `null`.

For `timeout`, `exit_code` must be `null` and `execution_time_ms` must represent the configured time limit.

For `runtime_error`, the actual program exit code and execution duration must be reported.

## 8. Execution Limits and Security

The Python execution environment must follow the limits specified in `API_SPEC.md`.

| Resource | Limit |
|---|---|
| Compilation / syntax validation time | 10 seconds |
| Program execution time | 5 seconds |
| Memory usage | 256 MB |
| Output size | 64 KB |
| Network access | Blocked |

Submitted Python programs must run inside an approved isolated execution environment rather than unrestricted within the MPLab web application process.

The implementation must support resource cleanup and concurrent requests without interfering with other executions.

The final sandbox configuration and security verification results will be documented when implementation is complete.

**Status:** Sandbox implementation and verification pending.

## 9. Task A — Python Language Investigation

**Assigned Member:** Keertti A/P Rasanran

Task A investigates Python's programming language design and execution characteristics.

The investigation will cover:

- Programming paradigms supported by Python.
- Python language characteristics.
- CPython implementation and execution process.
- Python data types.
- Dynamic typing and type checking.
- Variable scope and lifetime.
- Variable binding.
- Parameter passing.
- Error handling and exceptions.

The investigation should explain how these concepts relate to Python programming and the MPLab execution module.

**Planned Documentation:** `docs/investigation.md`

## 10. Task B — CPython Installation and Configuration

**Assigned Member:** Maredu Anes

Task B focuses on preparing the Python interpreter environment.

The work includes:

1. Installing CPython in the selected environment.
2. Verifying the installed interpreter version.
3. Configuring the Python environment.
4. Executing a successful Python program.
5. Demonstrating syntax error detection.
6. Demonstrating runtime error handling.
7. Recording installation and verification evidence.

The installation procedure for the final deployment environment must be documented separately from preliminary Windows development.

**Planned Documentation:** `docs/installation.md`

## 11. Task C — Python Execution and Error Handling

**Assigned Members:** Maredu Anes and Sanchay A/L Ravindran

Task C implements the Python execution functionality required by MPLab.

### Maredu Anes — Interpreter Execution

Responsibilities include:

- Processing submitted Python source code.
- Invoking the Python interpreter.
- Supporting standard input.
- Integrating the approved sandbox.
- Preparing the executable launcher.
- Assisting with execution testing.

### Sanchay A/L Ravindran — Output and Error Handling

Responsibilities include:

- Capturing standard output.
- Capturing standard error.
- Detecting and classifying execution failures.
- Mapping errors to official API statuses.
- Formatting JSON execution responses.
- Supporting testing and troubleshooting.

The completed module must comply with the official MPLab execution API specification.

## 12. Task D — Python Programming Concept Demonstrations

**Assigned Member:** Yashveena S. Jayaganthan

Task D requires at least five Python demonstrations illustrating programming language concepts.

### Planned Demonstrations

| Program | Programming Concept |
|---|---|
| `01_data_types.py` | Python data types and dynamic typing |
| `02_variable_scope.py` | Local and global variable scope |
| `03_variable_binding.py` | Variable binding and reassignment |
| `04_parameter_passing.py` | Parameter passing and object references |
| `05_recursion.py` | Recursive function execution |

Each program should provide clear examples of its assigned programming language concept.

The programs should include explanatory comments, sample input where applicable, and expected output.

The completed demonstrations must be tested through the integrated MPLab platform.

**Planned Folder:** `examples/`

## 13. Task E — Integration and Testing

**Assigned Members:**
- Shaif Ahmad Bin Tufail Ahmad — Integration and Coordination
- Yashveena S. Jayaganthan — Testing and Verification
- Sanchay A/L Ravindran — Error Handling Support

Task E ensures that the Python module functions correctly with the shared execution controller and MPLab platform.

### Integration Responsibilities

- Coordinate with Group 2 regarding the execution interface.
- Ensure the Python module uses the expected executable launcher.
- Verify JSON request and response compatibility.
- Check standard input and output handling.
- Coordinate integration issues and fixes.
- Review implementation changes.
- Prepare GitHub pull requests for integration.

### Planned Test Cases

| Test Case | Expected Result |
|---|---|
| Valid Python program | `success` |
| Python syntax error | `compile_error` |
| Python runtime exception | `runtime_error` |
| Program requiring standard input | Input processed correctly |
| Program exceeding runtime limit | `timeout` |
| Unsupported language | `unsupported_language` |
| Program exceeding output limit | Output restricted to the configured limit |
| Program exceeding memory limit | `runtime_error` |
| Attempted network access | Network access blocked |
| Controller integration | Valid response returned through the controller |

The assigned members will implement automated tests and record execution evidence.

**Planned Folder:** `tests/`

**Status:** Integration testing pending.

## 14. Lecturer Question Management

In addition to Python Tasks A–E, Group 4 is responsible for Lecturer Question Management.

**Official Folder:** `services/lecturer/`

**Development Branch:** `group4/lecturer-questions`

### Assigned Members

| Member | Responsibilities |
|---|---|
| Keertti A/P Rasanran | Question creation and editing |
| Sanchay A/L Ravindran | Question retrieval and deletion |
| Liang Bohao | Question validation, storage and quality assurance |
| Shaif Ahmad Bin Tufail Ahmad | Integration coordination |

### Planned Features

- Create programming questions.
- Retrieve questions.
- Edit existing questions.
- Delete questions.
- Validate question information.
- Store and manage question records.

The proposed implementation uses Python and SQLite.

The final database schema, REST API endpoints and integration requirements must be coordinated with the relevant groups.

Lecturer Question Management is documented and developed separately under `services/lecturer/`.

## 15. GitHub Development Workflow

**Official Repository:** https://github.com/lollookka/MPLab

**Python Development Branch:** `group4/python-executor`

**Lecturer Question Management Branch:** `group4/lecturer-questions`

### Contribution Rules

1. All development must take place on branches rather than directly on `main`.
2. Members should work on their assigned components.
3. Changes should be committed using meaningful commit messages.
4. Completed features should be tested before integration.
5. Pull requests must use titles beginning with `G4:`.
6. Pull requests must receive the required approval before merging.
7. Group 4 must not modify folders belonging to other groups without agreement.

According to the official API specification:

- Changes limited to `languages/python/` can be approved by a member of Group 4.
- Changes to shared folders such as `services/lecturer/` require approval from at least one member of another group.

The repository is public. Passwords, credentials, API keys and faculty VM access information must not be committed.

## 16. Development Status

**Current Phase:** Initial Setup

| Component | Status |
|---|---|
| Group member responsibilities | Defined |
| Repository structure | Planned |
| Python module README | Initial documentation |
| Task A investigation | Pending contribution |
| Task B installation | Pending contribution |
| Task C Python execution module | Pending implementation |
| Task D Python demonstrations | Pending implementation |
| Task E integration and testing | Pending implementation |
| Lecturer Question Management | Separate component |
| Sandbox configuration | Pending implementation and verification |

This README describes the planned implementation and integration approach. It does not indicate that unfinished components have been implemented, deployed or successfully tested.

## 17. References

- [Official MPLab README](../../README.md)
- [MPLab API Specification](../../API_SPEC.md)
- [Group 2 Execution Controller README](../../controller/README.md)

---

**Group 4 — Team Fahh**  
WIF3010 Programming Language Paradigms  
Universiti Malaya  
Semester 1, 2026/2027
