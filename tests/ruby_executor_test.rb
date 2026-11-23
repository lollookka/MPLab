
require "minitest/autorun"
require "json"
require "open3"
require "rbconfig"

# Group 7 - Ruby Executor Tests

EXECUTOR = File.expand_path(
  "../languages/ruby/executor.rb",
  __dir__
)

class RubyExecutorTest < Minitest::Test

  def execute_request(language, source_code, stdin = "")
    request = {
      language: language,
      source_code: source_code,
      stdin: stdin
    }

    stdout, stderr, process_status = Open3.capture3(
      RbConfig.ruby,
      EXECUTOR,
      stdin_data: JSON.generate(request)
    )

    assert process_status.success?, "Executor process failed: #{stderr}"

    JSON.parse(stdout)
  end

  # Test 1: Normal execution
  def test_normal_execution
    result = execute_request("ruby", "puts 10 + 20")

    assert_equal "success", result["status"]
    assert_equal "30\n", result["stdout"]
    assert_equal 0, result["exit_code"]
  end

  # Test 2: Runtime error
  def test_runtime_error
    result = execute_request("ruby", "puts 10 / 0")

    assert_equal "runtime_error", result["status"]
    assert_includes result["stderr"], "ZeroDivisionError"
    refute_equal 0, result["exit_code"]
  end

  # Test 3: Syntax error
  def test_syntax_error
    result = execute_request("ruby", "puts 10 +")

    assert_equal "compile_error", result["status"]
    assert_nil result["exit_code"]
    assert_nil result["execution_time_ms"]
  end

  # Test 4: Standard input
  def test_stdin
    result = execute_request(
      "ruby",
      'name = gets.chomp; puts "Hello, #{name}!"',
      "Tom\n"
    )

    assert_equal "success", result["status"]
    assert_equal "Hello, Tom!\n", result["stdout"]
  end

  # Test 5: Unsupported language
  def test_unsupported_language
    result = execute_request("python", "print(123)")

    assert_equal "unsupported_language", result["status"]
    assert_nil result["exit_code"]
  end

  # Test 6: Execution timeout
  def test_timeout
    result = execute_request("ruby", "loop { }")

    assert_equal "timeout", result["status"]
    assert_nil result["exit_code"]
    assert_equal 5000, result["execution_time_ms"]
  end

  # Test 7: Output size limit
  def test_output_limit
    result = execute_request(
      "ruby",
      'puts "A" * 100000'
    )

    assert_equal "success", result["status"]
    assert_operator result["stdout"].bytesize, :<=, 65536
  end

end
