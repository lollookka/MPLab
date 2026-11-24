"""Execute one PHP job. Use the module launcher to isolate student code."""

import json
import os
import signal
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
from threading import Thread
from time import monotonic


LINT_TIMEOUT_SECONDS = 10
MAX_RUN_MS = 5_000
MAX_MEMORY_MB = 256
MAX_OUTPUT_BYTES = 64 * 1024


def capture(stream, output):
    with stream:
        while chunk := stream.read(8192):
            available = MAX_OUTPUT_BYTES - len(output)
            output.extend(chunk[:available])


def run_php(php, source, arguments, input_file, timeout):
    stdout = bytearray()
    stderr = bytearray()
    started = monotonic()
    with input_file.open("rb") as stdin, subprocess.Popen(
        [php, "-n", "-d", "display_errors=stderr", "-d", "log_errors=0", *arguments, source.name],
        cwd=source.parent,
        stdin=stdin,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=os.name == "posix",
    ) as process:
        readers = [Thread(target=capture, args=(stream, output)) for stream, output in (
            (process.stdout, stdout), (process.stderr, stderr),
        )]
        for reader in readers:
            reader.start()
        timed_out = False
        try:
            exit_code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            exit_code = None
        finally:
            # End any remaining processes in this job before draining the pipes.
            if os.name == "posix":
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            elif process.poll() is None:
                process.kill()
            process.wait()
            duration_ms = int((monotonic() - started) * 1000)
            for reader in readers:
                reader.join()

    # Reserve room for diagnostics first so long output cannot hide an error.
    error = stderr.decode("utf-8", errors="replace").replace(str(source), source.name)
    error = error.encode("utf-8")[:MAX_OUTPUT_BYTES].decode("utf-8", errors="ignore")
    stdout_budget = MAX_OUTPUT_BYTES - len(error.encode("utf-8"))
    output = stdout[:stdout_budget].decode("utf-8", errors="replace")
    output = output.encode("utf-8")[:stdout_budget].decode("utf-8", errors="ignore")
    return output, error, exit_code, timed_out, duration_ms


def limit(job, field, maximum):
    value = job.get(field, maximum)
    if type(value) is not int or value < 1:
        raise ValueError(f"{field} must be a positive integer")
    return min(value, maximum)


def execute(job, php):
    if not isinstance(job, dict):
        raise ValueError("PHP runner input must be a JSON object")
    source_code = job.get("source_code")
    stdin = job.get("stdin", "")
    if not isinstance(source_code, str) or not isinstance(stdin, str):
        raise ValueError("source_code and stdin must be strings")
    run_ms = limit(job, "time_limit_ms", MAX_RUN_MS)
    memory_mb = limit(job, "memory_limit_mb", MAX_MEMORY_MB)
    with TemporaryDirectory(prefix="mplab-php-") as work:
        source = Path(work) / "main.php"
        source.write_bytes(source_code.encode("utf-8"))
        input_file = Path(work) / "stdin.txt"
        input_file.write_bytes(stdin.encode("utf-8"))
        php_limits = ["-d", f"memory_limit={memory_mb}M"]
        lint_out, lint_error, lint_code, lint_timeout, _ = run_php(
            php, source, [*php_limits, "-l"], input_file, LINT_TIMEOUT_SECONDS,
        )
        if lint_code != 0:
            if lint_timeout:
                lint_error = "PHP syntax check timed out."
            return {
                "phase": "compile", "stdout": "", "stderr": lint_error or lint_out,
                "exit_code": None, "timed_out": False, "duration_ms": None,
            }

        stdout, stderr, exit_code, timed_out, duration_ms = run_php(
            php, source, [*php_limits, "-f"], input_file, run_ms / 1000,
        )
        return {
            "phase": "run", "stdout": stdout, "stderr": stderr, "exit_code": exit_code,
            "timed_out": timed_out, "duration_ms": run_ms if timed_out else duration_ms,
        }


def api_result(status, stdout="", stderr="", exit_code=None, execution_time_ms=None):
    return {
        "status": status, "stdout": stdout, "stderr": stderr,
        "exit_code": exit_code, "execution_time_ms": execution_time_ms,
    }


def execute_api(job, php):
    if not isinstance(job, dict) or not isinstance(job.get("language"), str):
        raise ValueError("language must be a string")
    if job["language"] != "php":
        return api_result("unsupported_language", stderr="This module only supports PHP.")

    result = execute({"source_code": job.get("source_code"), "stdin": job.get("stdin", "")}, php)
    if result["phase"] == "compile":
        return api_result("compile_error", stderr=result["stderr"])
    if result["timed_out"]:
        return api_result("timeout", result["stdout"], result["stderr"], execution_time_ms=MAX_RUN_MS)

    exit_code = result["exit_code"]
    if exit_code < 0:
        exit_code = 128 - exit_code
    status = "success" if exit_code == 0 else "runtime_error"
    return api_result(status, result["stdout"], result["stderr"], exit_code, result["duration_ms"])


def main():
    api_mode = sys.argv[1:] == ["--api"]
    try:
        job = json.loads(sys.stdin.buffer.read())
        php = os.environ.get("MPLAB_PHP_BIN", "php")
        response = execute_api(job, php) if api_mode else execute(job, php)
    except (OSError, ValueError) as error:
        print(f"PHP runner failed: {error}", file=sys.stderr)
        if not api_mode:
            return 1
        response = api_result("internal_error", stderr="The PHP runner could not process this request.")
    print(json.dumps(response))
    return 0


if __name__ == "__main__":
    sys.exit(main())
