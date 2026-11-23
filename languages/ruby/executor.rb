
require "json"
require "open3"
require "tempfile"
require "timeout"
require "securerandom"

# MPLab Ruby Execution Module
# Group 7

OUTPUT_LIMIT = 64 * 1024
COMPILE_TIMEOUT = 10
RUN_TIMEOUT = 5
DOCKER_IMAGE = "mplab-ruby:3.4"

def create_response(status, stdout = "", stderr = "",
                    exit_code = nil, execution_time_ms = nil)
  {
    status: status,
    stdout: stdout,
    stderr: stderr,
    exit_code: exit_code,
    execution_time_ms: execution_time_ms
  }
end

def docker_command(file_path, container_name, *ruby_args)
 [
  "docker", "run", "--rm", "-i",
  "--name", container_name,
    "--network", "none",
    "--memory=256m",
    "--memory-swap=256m",
    "--cpus=1",
    "--pids-limit=64",
    "--read-only",
    "--tmpfs", "/tmp:rw,noexec,nosuid,size=16m,uid=65534,gid=65534",
    "--cap-drop=ALL",
    "--security-opt=no-new-privileges",
    "--user=65534:65534",
    "-v", "#{file_path}:/workspace/program.rb:ro",
    DOCKER_IMAGE,
    "ruby",
    *ruby_args
  ]
end

def run_command(command, stdin_data, time_limit, container_name)
  stdout = ""
  stderr = ""
  remaining = OUTPUT_LIMIT
  mutex = Mutex.new

  start_time = Process.clock_gettime(Process::CLOCK_MONOTONIC)

  Open3.popen3(*command, pgroup: true) do |stdin, out, err, wait_thr|

    readers = [
      Thread.new do
        begin
          loop do
            chunk = out.read(4096)
            break if chunk.nil? || chunk.empty?

            mutex.synchronize do
              allowed = [chunk.bytesize, remaining].min
              stdout << chunk.byteslice(0, allowed) if allowed > 0
              remaining -= allowed
            end
          end
        rescue IOError
        end
      end,

      Thread.new do
        begin
          loop do
            chunk = err.read(4096)
            break if chunk.nil? || chunk.empty?

            mutex.synchronize do
              allowed = [chunk.bytesize, remaining].min
              stderr << chunk.byteslice(0, allowed) if allowed > 0
              remaining -= allowed
            end
          end
        rescue IOError
        end
      end
    ]

    writer = Thread.new do
      begin
        stdin.write(stdin_data)
      rescue IOError, Errno::EPIPE
      ensure
        stdin.close rescue nil
      end
    end

    timed_out = false

    begin
      Timeout.timeout(time_limit) do
        wait_thr.value
      end
    rescue Timeout::Error
      timed_out = true

      begin
        Process.kill("KILL", -wait_thr.pid)
      rescue Errno::ESRCH
      end

      wait_thr.join

      # Remove the container if its Docker client timed out
      system(
        "docker", "rm", "-f", container_name,
        out: File::NULL,
        err: File::NULL
      )
    ensure
      writer.join
      readers.each(&:join)
    end

    elapsed = (
      Process.clock_gettime(Process::CLOCK_MONOTONIC) - start_time
    ) * 1000

    [
      stdout,
      stderr,
      timed_out ? nil : wait_thr.value,
      timed_out,
      elapsed.round
    ]
  end
end

def execute_request(request)
  unless request.is_a?(Hash)
    return create_response("internal_error", "", "Invalid request")
  end

  unless request["language"] == "ruby"
    return create_response("unsupported_language")
  end

  source_code = request["source_code"]
  stdin_data = request.fetch("stdin", "")

  unless source_code.is_a?(String) && stdin_data.is_a?(String)
    return create_response(
      "internal_error", "", "Invalid source_code or stdin"
    )
  end

  Tempfile.create(["mplab", ".rb"]) do |file|
    file.write(source_code)
    file.flush
    file.chmod(0o444)

    # Step 1: Check Ruby syntax inside Docker
    compile_name = "mplab-compile-#{SecureRandom.hex(6)}"

    begin
      _, syntax_stderr, syntax_status, compile_timeout, =
        run_command(
          docker_command(file.path, compile_name, "-c", "/workspace/program.rb"),
          "",
          COMPILE_TIMEOUT,
          compile_name
        )

      if compile_timeout || !syntax_status&.success?
        return create_response(
          "compile_error",
          "",
          syntax_stderr.empty? ? "Ruby syntax check failed or timed out" : syntax_stderr,
          nil,
          nil
        )
      end
    rescue StandardError => e
      return create_response("internal_error", "", e.message)
    end

    # Step 2: Execute Ruby program inside Docker
    run_name = "mplab-run-#{SecureRandom.hex(6)}"

    begin
      stdout, stderr, status, timed_out, elapsed =
        run_command(
          docker_command(file.path, run_name, "/workspace/program.rb"),
          stdin_data,
          RUN_TIMEOUT,
          run_name
        )

      if timed_out
        return create_response(
          "timeout", stdout, stderr, nil, RUN_TIMEOUT * 1000
        )
      end

      if status&.success?
        create_response("success", stdout, stderr, 0, elapsed)
      else
        create_response(
          "runtime_error",
          stdout,
          stderr,
          status&.exitstatus || 1,
          elapsed
        )
      end
    rescue StandardError => e
      create_response("internal_error", "", e.message)
    end
  end
rescue StandardError => e
  create_response("internal_error", "", e.message)
end

# Read JSON request and return JSON response
begin
  request = JSON.parse(STDIN.read)
  response = execute_request(request)
  puts JSON.generate(response)
rescue JSON::ParserError => e
  puts JSON.generate(
    create_response("internal_error", "", e.message)
  )
end
