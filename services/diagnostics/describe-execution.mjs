const labels = {
  compile_error: "Compilation or syntax error",
  runtime_error: "Runtime error",
  timeout: "Time limit exceeded",
  unsupported_language: "Unsupported language",
  internal_error: "Execution unavailable",
};

export function describeExecution(result, language) {
  if (!Object.hasOwn(labels, result.status)) return null;

  const details = result.stderr.trim();
  const firstLine = details.split(/\r?\n/, 1)[0];
  const phpError = language === "php" &&
    (result.status === "compile_error" || result.status === "runtime_error")
    ? /^(?:PHP )?(?:Parse error|Fatal error): (.*) in main\.php(?: on line |:)(\d+)$/.exec(firstLine)
    : null;

  return {
    title: labels[result.status],
    message: phpError?.[1] ?? (firstLine || labels[result.status]),
    line: phpError ? Number(phpError[2]) : null,
    details,
  };
}
