Ruby Module

Owner: Group 7
Language: Ruby
Interpreter: Ruby 3.4
Execution Environment: Docker

1. Overview

The Ruby Module is part of MPLab, an online multi-language programming and assessment platform.

It receives a JSON request through standard input, executes Ruby source code inside an isolated Docker container, and returns a JSON response through standard output.

2. Requirements

* Ruby 3.4 or later
* Docker Desktop or Docker Engine
* Docker daemon running

3. Installation and Setup

Clone the MPLab repository:

git clone https://github.com/lollookka/MPLab.git
cd MPLab

Build the Ruby Docker image:

docker build -t mplab-ruby:3.4 ./languages/ruby

Verify the image:

docker run --rm mplab-ruby:3.4

4. How the Module Works

The execution process consists of the following steps:

1. Read a JSON request from standard input.
2. Validate the requested language and input fields.
3. Save the Ruby source code into a temporary file.
4. Check Ruby syntax inside a Docker container.
5. Execute the Ruby program inside a separate Docker container.
6. Capture standard output and standard error.
7. Return the execution result in JSON format.
8. Remove the temporary file and Docker container.

5. Request Format

Example request:

{
  "language": "ruby",
  "source_code": "puts 10 + 20",
  "stdin": ""
}

* language: Must be "ruby".
* source_code: Ruby source code to execute.
* stdin: Optional standard input. Defaults to an empty string.

6. Response Format

Example successful response:

{
  "status": "success",
  "stdout": "30\n",
  "stderr": "",
  "exit_code": 0,
  "execution_time_ms": 169
}

Supported status values:

* success
* compile_error
* runtime_error
* timeout
* unsupported_language
* internal_error

7. Execution Limits

The Docker execution environment applies the following restrictions:

Resource	Limit
Execution time	5 seconds
Compilation timeout	10 seconds
Memory	256 MB
CPU	1 CPU
Output	64 KB
Network	Disabled
Process count	64

Additional Docker restrictions include a read-only root filesystem, dropped Linux capabilities, and disabled privilege escalation.

8. Running the Module

Run the executor from the MPLab repository root:

printf '%s' '{"language":"ruby","source_code":"puts 10 + 20","stdin":""}' | ruby languages/ruby/executor.rb

Expected output:

{
  "status": "success",
  "stdout": "30\n",
  "stderr": "",
  "exit_code": 0,
  "execution_time_ms": 169
}

The execution time may vary.

9. Automated Testing

Run the Ruby module tests:

ruby tests/ruby_executor_test.rb

The test suite covers:

1. Normal execution
2. Runtime errors
3. Syntax errors
4. Standard input
5. Unsupported languages
6. Execution timeout
7. Output size limit

10. Integration

The module communicates using JSON through standard input and standard output.

The controller can invoke the executor as a subprocess and pass the execution request through standard input. The JSON response is returned through standard output.

The exact invocation method should be coordinated with the controller team.

⸻

Group 7 – Ruby Module
MPLab Project 2026/2027