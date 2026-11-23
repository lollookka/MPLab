# Integration Testing and Quality Assurance

**Owner:** Group 7

## Overview

This directory contains automated tests for the MPLab execution modules.

## Ruby Executor Tests

The Ruby executor is tested using Ruby's Minitest framework.

Test file:
- `ruby_executor_test.rb`

## Test Coverage

The following scenarios are tested:

1. Normal Ruby program execution
2. Runtime error handling
3. Syntax error handling
4. Standard input handling
5. Unsupported language handling
6. Execution timeout handling
7. Output size limitation

## Running Tests

Run the following command from the project root:

```bash
ruby tests/ruby_executor_test.rb