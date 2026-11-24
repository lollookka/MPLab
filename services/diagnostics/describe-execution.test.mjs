import assert from "node:assert/strict";
import test from "node:test";
import { describeExecution } from "./describe-execution.mjs";

test("successful runs do not show an error", () => {
  assert.equal(describeExecution({ status: "success", stderr: "" }, "php"), null);
});

test("PHP syntax errors show the reported source line", () => {
  const stderr = 'Parse error: syntax error, unexpected token "echo" in main.php on line 3';
  assert.deepEqual(describeExecution({ status: "compile_error", stderr }, "php"), {
    title: "Compilation or syntax error",
    message: 'syntax error, unexpected token "echo"',
    line: 3,
    details: stderr,
  });
});

test("PHP runtime errors retain the stack trace as details", () => {
  const stderr = "Fatal error: Uncaught DivisionByZeroError: Division by zero in main.php:3\nStack trace:\n#0 {main}";
  const diagnostic = describeExecution({ status: "runtime_error", stderr }, "php");
  assert.equal(diagnostic.message, "Uncaught DivisionByZeroError: Division by zero");
  assert.equal(diagnostic.line, 3);
  assert.equal(diagnostic.details, stderr);
});

test("other languages retain their diagnostics as text", () => {
  const stderr = "ZeroDivisionError: division by zero\n<trace>";
  assert.deepEqual(describeExecution({ status: "runtime_error", stderr }, "python"), {
    title: "Runtime error",
    message: "ZeroDivisionError: division by zero",
    line: null,
    details: stderr,
  });
});

test("a timeout has a useful message when stderr is empty", () => {
  assert.deepEqual(describeExecution({ status: "timeout", stderr: "" }, "php"), {
    title: "Time limit exceeded", message: "Time limit exceeded", line: null, details: "",
  });
});
