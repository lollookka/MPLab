# MPLab PHP Runner

**Owner:** Group 8 Crashouts

The MPLab controller starts `languages/php/execute` from the repository root. It writes one UTF-8 JSON request to the launcher’s stdin and closes the stream. The launcher runs one PHP job in a disposable Docker container. The module prints a single public API v1 response to stdout. Module logs go to stderr. The controller can forward the response directly.

The request contains `language: "php"`, `source_code` and `stdin`. The response contains `status`, `stdout`, `stderr`, `exit_code` and `execution_time_ms`. The runner checks the syntax with `php -l` before evaluating the program.

| Result | Public API status |
| --- | --- |
| Syntax rejected or lint exceeded 10 seconds | `compile_error` with null exit code and execution time |
| Execution reached the 5-second limit | `timeout` with null exit code and execution time of 5000 ms |
| Program exited with code 0 | `success` |
| Program exited with a nonzero code | `runtime_error` |
| Request could not be read or PHP CLI could not start | `internal_error` |

## Run in the approved environment

Use a POSIX host with approved Docker access. The image contains Python and PHP CLI. Each job runs as an unprivileged user with no network, no host mounts, a read-only root filesystem and writable temporary space. The launcher caps the container at one CPU, 256 MiB RAM and 32 processes. The runner caps the combined `stdout` and `stderr` at 64 KiB. The image gives each job a 22-second overall deadline, ahead of the controller’s 30-second backstop.

From the repository root:

```sh
docker build -t mplab-php:local languages/php
chmod +x languages/php/execute
printf '%s' '{"language":"php","source_code":"<?php echo 2 * (int) fgets(STDIN);","stdin":"7\n"}' | languages/php/execute
```

Expect `status: "success"`, `stdout: "14"` and `exit_code: 0`. Repeat through the controller’s `POST /api/execute` endpoint after starting it with this repository as `MPLAB_ROOT`. Verify normal output, syntax error, runtime error, timeout and the container memory limit in the approved environment before release. Record the executable bit with `git add --chmod=+x languages/php/execute` and keep LF line endings for the shell script.

## Test the module locally

Run `python3 languages/php/test_runner.py` from the repository root with PHP CLI available. `MPLAB_PHP_BIN` can point to a trusted local PHP executable. The tests and `runner.py --api` can run with the included examples on a developer machine. Send student submissions through `execute` in the approved sandbox. The older `runner.sh` continues to expose the local draft format for existing callers.

The [example programs](examples/README.md) include five course demonstrations and two error cases. The [API test requests](examples/api-test-cases.json) cover a successful run, a syntax error and a runtime error.

References: [MPLab API specification](../../API_SPEC.md) and [controller runner contract](https://github.com/lollookka/MPLab/blob/4efd4aedef128284173fba1921fb450ff6ea874f/controller/README.md).
