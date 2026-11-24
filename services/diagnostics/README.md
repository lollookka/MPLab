# MPLab Execution Diagnostics

This Group 8 module turns an API v1 execution response into a short label for the editor and optional details. The web client can import `describeExecution(result, language)` from `describe-execution.mjs` after calling `POST /api/execute`.

For a failure, show `title` and `message`. If `line` is present in the open source file, the editor can highlight it. Show `details` as plain text in an expandable section. For a successful response, the helper returns `null`; the existing output panel can continue showing `stdout` and any warnings in `stderr`.

The helper currently extracts a source line from PHP parse and fatal errors. Other languages use the status label and their original diagnostics until their error formats can be verified. The API response itself is unchanged. The helper is ready for local use and awaits wiring into the shared web client.

Run `node --test services/diagnostics/describe-execution.test.mjs` from the repository root.
