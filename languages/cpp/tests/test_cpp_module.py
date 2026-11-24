"""Tests for the MPLab C++ execution module (Group 2).

Stdlib unittest only. Every test spawns the module the way the controller
does: JSON request on stdin, JSON response on stdout, exit 0. Run from the
repo root:

    python3 -m unittest discover -s languages/cpp/tests -v
"""

import json
import os
import subprocess
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
EXECUTOR = os.path.join(REPO_ROOT, "languages", "cpp", "executor.py")
LAUNCHER = os.path.join(REPO_ROOT, "languages", "cpp", "execute")

HELLO_SOURCE = '#include <iostream>\nint main(){ std::cout << "Hello, MPLab!"; }'
HELLO_OUTPUT = "Hello, MPLab!"

RESPONSE_FIELDS = {
    "status",
    "stdout",
    "stderr",
    "exit_code",
    "execution_time_ms",
}


def run_module(request, raw=None, launcher=None):
    """Pipe one request to the module; return (exit code, parsed response)."""
    cmd = [launcher] if launcher else ["python3", EXECUTOR]
    proc = subprocess.run(
        cmd,
        input=raw if raw is not None else json.dumps(request).encode("utf-8"),
        cwd=REPO_ROOT,
        capture_output=True,
        timeout=60,
    )
    return proc.returncode, json.loads(proc.stdout.decode("utf-8"))


class ExecutorTests(unittest.TestCase):
    def test_success_hello_world(self):
        code, r = run_module({"language": "cpp", "source_code": HELLO_SOURCE})
        self.assertEqual(code, 0)
        self.assertEqual(set(r), RESPONSE_FIELDS)
        self.assertEqual(r["status"], "success")
        self.assertEqual(r["stdout"], HELLO_OUTPUT)
        self.assertEqual(r["exit_code"], 0)
        self.assertIsInstance(r["execution_time_ms"], int)
        self.assertGreaterEqual(r["execution_time_ms"], 0)

    def test_stdin_delivered(self):
        r = run_module(
            {
                "language": "cpp",
                "source_code": "#include <iostream>\n#include <string>\n"
                "int main(){ std::string line;\n"
                "while (std::getline(std::cin, line)) std::cout << line << '\\n'; }",
                "stdin": "first line\nsecond line\n",
            }
        )[1]
        self.assertEqual(r["status"], "success")
        self.assertEqual(r["stdout"], "first line\nsecond line\n")

    def test_compile_error(self):
        r = run_module(
            {"language": "cpp", "source_code": "int main(){ int x = 5\n}"}
        )[1]
        self.assertEqual(r["status"], "compile_error")
        self.assertEqual(r["stdout"], "")
        self.assertNotEqual(r["stderr"], "")  # GCC diagnostics present
        self.assertIsNone(r["exit_code"])
        self.assertIsNone(r["execution_time_ms"])

    def test_runtime_error_uncaught_exception(self):
        r = run_module(
            {
                "language": "cpp",
                "source_code": "#include <stdexcept>\n"
                'int main(){ throw std::runtime_error("boom"); }',
            }
        )[1]
        self.assertEqual(r["status"], "runtime_error")
        self.assertEqual(r["exit_code"], 134)  # 128 + SIGABRT(6), DESIGN.md §9
        self.assertIn("boom", r["stderr"])

    def test_timeout(self):
        r = run_module(
            {"language": "cpp", "source_code": "int main(){ volatile unsigned long i = 0; for(;;){ ++i; } }"}
        )[1]
        self.assertEqual(r["status"], "timeout")
        self.assertIsNone(r["exit_code"])
        self.assertEqual(r["execution_time_ms"], 5000)

    def test_unsupported_language_exact_shape(self):
        code, r = run_module({"language": "java", "source_code": HELLO_SOURCE})
        self.assertEqual(code, 0)
        self.assertEqual(
            r,
            {
                "status": "unsupported_language",
                "stdout": "",
                "stderr": "",
                "exit_code": None,
                "execution_time_ms": None,
            },
        )

    def test_output_cap(self):
        r = run_module(
            {
                "language": "cpp",
                "source_code": "#include <iostream>\n"
                'int main(){ for (int i = 0; i < 20000; ++i) std::cout << "0123456789"; }',
            }
        )[1]
        self.assertEqual(r["status"], "success")
        self.assertLessEqual(len(r["stdout"].encode("utf-8")), 65536)

    def test_launcher_end_to_end(self):
        self.assertTrue(os.access(LAUNCHER, os.X_OK), "execute must be executable")
        code, r = run_module(
            {"language": "cpp", "source_code": HELLO_SOURCE, "stdin": ""},
            launcher=LAUNCHER,
        )
        self.assertEqual(code, 0)
        self.assertEqual(set(r), RESPONSE_FIELDS)
        self.assertEqual(r["status"], "success")
        self.assertEqual(r["stdout"], HELLO_OUTPUT)
        self.assertEqual(r["exit_code"], 0)

    def test_malformed_envelope(self):
        code, r = run_module(None, raw=b"this is not json")
        self.assertEqual(code, 0)
        self.assertEqual(set(r), RESPONSE_FIELDS)
        self.assertEqual(r["status"], "internal_error")
        self.assertEqual(r["stderr"], "MPLab cpp module: malformed request envelope")
        self.assertEqual(r["stdout"], "")
        self.assertIsNone(r["exit_code"])
        self.assertIsNone(r["execution_time_ms"])


if __name__ == "__main__":
    unittest.main()
