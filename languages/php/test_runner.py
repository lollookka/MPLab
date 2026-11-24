import json
import subprocess
import sys
import unittest
from pathlib import Path


RUNNER = Path(__file__).with_name("runner.py")


class PhpRunnerTest(unittest.TestCase):
    def run_php(self, source, *, api=False, **job):
        command = [sys.executable, str(RUNNER)]
        if api:
            command.append("--api")
            job.setdefault("language", "php")
        completed = subprocess.run(
            command,
            input=json.dumps({"source_code": source, **job}),
            encoding="utf-8", capture_output=True, check=True, timeout=20,
        )
        self.assertEqual(completed.stderr, "")
        result = json.loads(completed.stdout)
        fields = {"status", "stdout", "stderr", "exit_code", "execution_time_ms"} if api else {
            "phase", "stdout", "stderr", "exit_code", "timed_out", "duration_ms",
        }
        self.assertEqual(set(result), fields)
        return result

    def test_success_reads_student_input(self):
        result = self.run_php('<?php echo "Double of 7 is " . (2 * (int) fgets(STDIN));', stdin="7\n")
        self.assertEqual((result["phase"], result["exit_code"]), ("run", 0))
        self.assertFalse(result["timed_out"])
        self.assertEqual(result["stdout"], "Double of 7 is 14")
        self.assertGreaterEqual(result["duration_ms"], 0)

    def test_syntax_error_is_marked_compile(self):
        result = self.run_php('<?php\necho "first"\necho "second";')
        self.assertEqual(result["phase"], "compile")
        self.assertEqual(result["stdout"], "")
        self.assertIn("unexpected token", result["stderr"])
        self.assertRegex(result["stderr"], r"main\.php.*line 3")
        self.assertIsNone(result["exit_code"])
        self.assertIsNone(result["duration_ms"])
        self.assertFalse(result["timed_out"])

    def test_runtime_error_retains_exit_code_and_output(self):
        result = self.run_php('<?php echo "start"; echo 10 / 0;')
        self.assertEqual(result["phase"], "run")
        self.assertNotEqual(result["exit_code"], 0)
        self.assertEqual(result["stdout"], "start")
        self.assertIn("Division by zero", result["stderr"])

    def test_timeout_uses_requested_limit(self):
        result = self.run_php('<?php usleep(5000000);', time_limit_ms=200)
        self.assertTrue(result["timed_out"])
        self.assertEqual(result["phase"], "run")
        self.assertIsNone(result["exit_code"])
        self.assertEqual(result["duration_ms"], 200)

    def test_output_is_truncated_without_losing_error(self):
        result = self.run_php('<?php echo str_repeat("x", 100000); echo 10 / 0;')
        self.assertNotEqual(result["exit_code"], 0)
        self.assertIn("Division by zero", result["stderr"])
        self.assertEqual(len((result["stdout"] + result["stderr"]).encode("utf-8")), 64 * 1024)

    def test_api_success_reads_student_input(self):
        result = self.run_php('<?php echo 2 * (int) fgets(STDIN);', stdin="7\n", api=True)
        self.assertEqual((result["status"], result["stdout"], result["stderr"], result["exit_code"]),
                         ("success", "14", "", 0))
        self.assertGreaterEqual(result["execution_time_ms"], 0)

    def test_api_lint_rejection_uses_compile_error(self):
        result = self.run_php('<?php\necho "first"\necho "second";', api=True)
        self.assertEqual((result["status"], result["stdout"]), ("compile_error", ""))
        self.assertRegex(result["stderr"], r"main\.php.*line 3")
        self.assertIsNone(result["exit_code"])
        self.assertIsNone(result["execution_time_ms"])

    def test_api_runtime_error_preserves_output_and_exit_code(self):
        result = self.run_php('<?php echo "start"; echo 10 / 0;', api=True)
        self.assertEqual((result["status"], result["stdout"]), ("runtime_error", "start"))
        self.assertIn("Division by zero", result["stderr"])
        self.assertEqual(result["exit_code"], 255)
        self.assertGreaterEqual(result["execution_time_ms"], 0)

    def test_api_timeout_uses_shared_limit(self):
        result = self.run_php('<?php usleep(8000000);', time_limit_ms=200, api=True)
        self.assertEqual(result["status"], "timeout")
        self.assertIsNone(result["exit_code"])
        self.assertEqual(result["execution_time_ms"], 5000)

    def test_api_rejects_other_languages(self):
        result = self.run_php("not a PHP program", language="ruby", api=True)
        self.assertEqual(result["status"], "unsupported_language")
        self.assertIsNone(result["exit_code"])
        self.assertIsNone(result["execution_time_ms"])


if __name__ == "__main__":
    unittest.main()
