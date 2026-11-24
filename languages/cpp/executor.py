"""MPLab C++ execution module (Group 2).

The whole module in one file. It reads ONE JSON request
``{language, source_code, stdin}`` from stdin (the controller pipes it
through ``languages/cpp/execute``, controller/DESIGN.md §5.4), compiles the
source with ``g++``, runs the binary under a small sandbox, and writes
exactly ONE JSON response ``{status, stdout, stderr, exit_code,
execution_time_ms}`` (API_SPEC §2/§3) to stdout, then exits 0. The module's
own diagnostics go to stderr (DESIGN.md §5.7), never to stdout.

Statuses (API_SPEC §3): ``success``, ``compile_error``, ``runtime_error``,
``timeout``, ``unsupported_language``, ``internal_error``.
``exit_code``/``execution_time_ms`` are null for ``compile_error``,
``unsupported_language`` and ``internal_error``. A program killed by a
signal reports ``exit_code = 128 + signal number`` (DESIGN.md §9:
SIGSEGV → 139, SIGABRT → 134) with a ``killed by signal SIGxxx`` line
appended after the program's own stderr. A run timeout reports
``exit_code = null`` and ``execution_time_ms = 5000``. Output (stdout and
stderr) is truncated to 64 KB and decoded as UTF-8 with errors replaced.

Sandbox: each request uses one temp directory (prefix ``mplab-cpp-``,
removed afterwards). The compile step runs ``g++`` under rlimits
(2 GB address space, CPU limit 11 s, no core dumps) with ``TMPDIR`` pointed
into the temp dir; the program runs under rlimits (256 MB address space,
CPU limit 6 s, no core dumps). Every process starts in its own session so a
timeout can SIGKILL the whole process group. Network is blocked best-effort
with ``unshare --net`` when that works on this host; otherwise the program
runs directly with one warning line on this module's stderr. Documented
residuals: isolation is same-uid only (no setuid/chroot), a grandchild that
calls ``setsid()`` escapes the process-group kill, and output is buffered
by ``communicate()`` (bounded by the 64 KB truncation and the rlimits).
Python 3 standard library only.
"""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time

OUTPUT_LIMIT_BYTES = 64 * 1024
COMPILE_LIMIT_S = 10.0
RUN_LIMIT_S = 5.0
COMPILE_MEMORY_BYTES = 2 * 1024**3
MEMORY_LIMIT_BYTES = 256 * 1024 * 1024
MALFORMED_STDERR = "MPLab cpp module: malformed request envelope"


def _response(status, stdout="", stderr="", exit_code=None, execution_time_ms=None):
    return {
        "status": status,
        "stdout": stdout,
        "stderr": stderr,
        "exit_code": exit_code,
        "execution_time_ms": execution_time_ms,
    }


def _internal_error(stderr):
    return _response("internal_error", stderr=stderr)


def _limits(memory_bytes, cpu_seconds):
    """preexec_fn body: rlimits applied in the child before exec."""

    def apply():
        import resource

        resource.setrlimit(resource.RLIMIT_AS, (memory_bytes, memory_bytes))
        resource.setrlimit(resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

    return apply


def _clip(data):
    return data[:OUTPUT_LIMIT_BYTES].decode("utf-8", errors="replace")


def _kill_group(proc):
    try:
        os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
    except OSError:
        pass


_unshare_ok = None


def _netns_cmd(binary):
    """Run command: ``unshare --net`` when it works here (probed once, then
    cached), otherwise direct mode with one loud warning line."""
    global _unshare_ok
    if _unshare_ok is None:
        try:
            probe = subprocess.run(
                ["unshare", "--net", "true"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=5,
            )
            _unshare_ok = probe.returncode == 0
        except Exception:
            _unshare_ok = False
    if _unshare_ok:
        return ["unshare", "--net", binary]
    sys.stderr.write(
        "MPLab cpp module: network NOT blocked (unshare unavailable)\n"
    )
    sys.stderr.flush()
    return [binary]


def _compile(workdir):
    """g++ main.cpp → prog. Returns None on success, else diagnostics."""
    env = dict(os.environ, TMPDIR=workdir)
    proc = subprocess.Popen(
        ["g++", "main.cpp", "-o", "prog"],
        cwd=workdir,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
        preexec_fn=_limits(COMPILE_MEMORY_BYTES, int(COMPILE_LIMIT_S) + 1),
        env=env,
    )
    try:
        _, err = proc.communicate(timeout=COMPILE_LIMIT_S)
    except subprocess.TimeoutExpired:
        _kill_group(proc)
        _, err = proc.communicate()
    return None if proc.returncode == 0 else _clip(err or b"")


def _run(workdir, stdin_data):
    cmd = _netns_cmd("./prog")
    try:
        proc = _popen_run(cmd, workdir)
    except OSError:
        if cmd[0] != "unshare":
            raise
        sys.stderr.write(
            "MPLab cpp module: network NOT blocked (unshare unavailable)\n"
        )
        sys.stderr.flush()
        proc = _popen_run(cmd[2:], workdir)

    timed_out = False
    started = time.monotonic()
    try:
        out, err = proc.communicate(
            input=stdin_data.encode("utf-8"), timeout=RUN_LIMIT_S
        )
    except subprocess.TimeoutExpired:
        timed_out = True
        _kill_group(proc)
        try:  # bounded reap; never hang even if a grandchild keeps pipes open
            out, err = proc.communicate(timeout=2)
        except subprocess.TimeoutExpired:
            try:
                proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
            for pipe in (proc.stdin, proc.stdout, proc.stderr):
                try:
                    pipe.close()
                except OSError:
                    pass
            out, err = b"", b""
    elapsed_ms = int((time.monotonic() - started) * 1000)
    out_text, err_text = _clip(out or b""), _clip(err or b"")

    if timed_out:
        return _response("timeout", out_text, err_text, None, int(RUN_LIMIT_S * 1000))
    code = proc.returncode
    if code < 0:  # killed by signal N == -returncode
        name = signal.Signals(-code).name
        err_text = (err_text + "\n" if err_text else "") + "killed by signal %s" % name
        code = 128 - code
    status = "success" if proc.returncode == 0 else "runtime_error"
    return _response(status, out_text, err_text, code, elapsed_ms)


def _popen_run(cmd, workdir):
    return subprocess.Popen(
        cmd,
        cwd=workdir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
        preexec_fn=_limits(MEMORY_LIMIT_BYTES, int(RUN_LIMIT_S) + 1),
    )


def execute(request):
    if (
        not isinstance(request, dict)
        or not isinstance(request.get("language"), str)
        or not isinstance(request.get("source_code"), str)
        or not isinstance(request.get("stdin", ""), str)
    ):
        return _internal_error(MALFORMED_STDERR)
    if request["language"] != "cpp":
        return _response("unsupported_language")

    workdir = tempfile.mkdtemp(prefix="mplab-cpp-")
    try:
        with open(os.path.join(workdir, "main.cpp"), "wb") as f:
            f.write(request["source_code"].encode("utf-8"))
        compile_stderr = _compile(workdir)
        if compile_stderr is not None:
            return _response("compile_error", stderr=compile_stderr)
        return _run(workdir, request.get("stdin", ""))
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


def main():
    try:
        request = json.loads(sys.stdin.buffer.read())
    except Exception:
        request = None
    try:
        response = execute(request)
    except Exception as exc:  # never leak a traceback into a response
        response = _internal_error("MPLab cpp module: %s" % (exc,))
    sys.stdout.buffer.write((json.dumps(response) + "\n").encode("utf-8"))
    sys.stdout.buffer.flush()


if __name__ == "__main__":
    main()
