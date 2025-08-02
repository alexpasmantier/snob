use anyhow::Result;
use std::fs;
use tempfile::TempDir;

// Note: These test fixtures create Python projects with clean import statements.
// Tests using these fixtures should set PYTHONPATH to include the src/ directory
// so Python can find the modules. Use test_utils::setup_pythonpath_for_project()
// for this purpose.

/// Creates a simple project with basic dependency chain:
/// main.py -> module.py
/// test_main.py, test_module.py
pub fn create_simple_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create source files
    fs::write(
        project_path.join("src/main.py"),
        r#"
from module import process_data

def main():
    result = process_data("hello")
    print(result)

if __name__ == "__main__":
    main()
"#,
    )?;

    fs::write(
        project_path.join("src/module.py"),
        r#"
def process_data(data):
    return data.upper()

def helper_function():
    return "helper"
"#,
    )?;

    // Create test files
    fs::write(
        project_path.join("tests/test_main.py"),
        r#"
from main import main

def test_main():
    main()  # Should not raise

def test_main_functionality():
    assert True
"#,
    )?;

    fs::write(
        project_path.join("tests/test_module.py"),
        r#"
from module import process_data, helper_function

def test_process_data():
    assert process_data("hello") == "HELLO"

def test_helper_function():
    assert helper_function() == "helper"
"#,
    )?;

    Ok(temp_dir)
}

/// Creates a complex project with transitive dependencies:
/// main.py -> module.py -> utils.py
/// test files for each module
pub fn create_complex_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create source files with transitive dependencies
    fs::write(
        project_path.join("src/main.py"),
        r#"
from module import process_data
from utils import format_output

def main():
    data = process_data("hello")
    result = format_output(data)
    print(result)

if __name__ == "__main__":
    main()
"#,
    )?;

    fs::write(
        project_path.join("src/module.py"),
        r#"
from utils import validate_input

def process_data(data):
    if not validate_input(data):
        raise ValueError("Invalid input")
    return data.upper()

def transform_data(data):
    return data.lower()
"#,
    )?;

    fs::write(
        project_path.join("src/utils.py"),
        r#"
def validate_input(data):
    return isinstance(data, str) and len(data) > 0

def format_output(data):
    return f"Result: {data}"

def common_utility():
    return 42
"#,
    )?;

    // Create test files
    fs::write(
        project_path.join("tests/test_main.py"),
        r#"

from main import main

def test_main():
    main()

def test_main_integration():
    assert True
"#,
    )?;

    fs::write(
        project_path.join("tests/test_module.py"),
        r#"

from module import process_data, transform_data
import pytest

def test_process_data():
    assert process_data("hello") == "HELLO"

def test_process_data_invalid():
    with pytest.raises(ValueError):
        process_data("")

def test_transform_data():
    assert transform_data("HELLO") == "hello"
"#,
    )?;

    fs::write(
        project_path.join("tests/test_utils.py"),
        r#"

from utils import validate_input, format_output, common_utility

def test_validate_input():
    assert validate_input("hello") == True
    assert validate_input("") == False
    assert validate_input(None) == False

def test_format_output():
    assert format_output("test") == "Result: test"

def test_common_utility():
    assert common_utility() == 42
"#,
    )?;

    Ok(temp_dir)
}

/// Creates a project with isolated files (no dependencies)
pub fn create_isolated_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create isolated files
    fs::write(
        project_path.join("src/isolated.py"),
        r#"
def standalone_function():
    return "standalone"

class StandaloneClass:
    def method(self):
        return "method"
"#,
    )?;

    fs::write(
        project_path.join("src/another_isolated.py"),
        r#"
def another_function():
    return "another"
"#,
    )?;

    // Create corresponding test files
    fs::write(
        project_path.join("tests/test_isolated.py"),
        r#"

from isolated import standalone_function, StandaloneClass

def test_standalone_function():
    assert standalone_function() == "standalone"

def test_standalone_class():
    obj = StandaloneClass()
    assert obj.method() == "method"
"#,
    )?;

    fs::write(
        project_path.join("tests/test_another_isolated.py"),
        r#"

from another_isolated import another_function

def test_another_function():
    assert another_function() == "another"
"#,
    )?;

    Ok(temp_dir)
}

/// Creates a project with configuration file (snob.toml)
pub fn create_project_with_config() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests/mandatory"))?;

    // Create snob.toml configuration
    fs::write(
        project_path.join("snob.toml"),
        r#"
[general]
verbosity_level = 1
quiet = false

[files]
ignores = ["src/ignored.py"]
run-all-tests-on-change = ["src/critical.py"]

[tests]
ignores = ["tests/test_expensive.py"]
always-run = ["tests/mandatory/test_mandatory.py"]
"#,
    )?;

    // Create source files
    fs::write(
        project_path.join("src/main.py"),
        r#"
def main_function():
    return "main"
"#,
    )?;

    fs::write(
        project_path.join("src/critical.py"),
        r#"
# This file should trigger all tests when changed
CRITICAL_CONSTANT = "important"

def critical_function():
    return CRITICAL_CONSTANT
"#,
    )?;

    fs::write(
        project_path.join("src/ignored.py"),
        r#"
# This file should be ignored by snob
def ignored_function():
    return "ignored"
"#,
    )?;

    // Create test files
    fs::write(
        project_path.join("tests/test_main.py"),
        r#"

from main import main_function

def test_main_function():
    assert main_function() == "main"
"#,
    )?;

    fs::write(
        project_path.join("tests/test_critical.py"),
        r#"

from critical import critical_function, CRITICAL_CONSTANT

def test_critical_function():
    assert critical_function() == "important"

def test_critical_constant():
    assert CRITICAL_CONSTANT == "important"
"#,
    )?;

    fs::write(
        project_path.join("tests/test_expensive.py"),
        r#"
# This test should be ignored by configuration
def test_expensive_operation():
    # Simulate expensive test
    assert sum(range(1000)) == 499500
"#,
    )?;

    fs::write(
        project_path.join("tests/mandatory/test_mandatory.py"),
        r#"
# This test should always run
def test_mandatory_check():
    assert True

def test_always_required():
    assert 1 + 1 == 2
"#,
    )?;

    Ok(temp_dir)
}

/// Creates a project to test different test file patterns
pub fn create_test_pattern_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create source file
    fs::write(
        project_path.join("src/main.py"),
        r#"
def main_function():
    return "main"

class MainClass:
    def method(self):
        return "method"
"#,
    )?;

    // Create test files with different patterns
    fs::write(
        project_path.join("tests/test_main.py"),
        r#"

from main import main_function, MainClass

def test_main_function():
    assert main_function() == "main"

def test_main_class():
    obj = MainClass()
    assert obj.method() == "method"
"#,
    )?;

    fs::write(
        project_path.join("tests/main_test.py"),
        r#"

from main import main_function

def test_alternative_pattern():
    assert main_function() == "main"
"#,
    )?;

    Ok(temp_dir)
}

/// Creates a project with nested packages
pub fn create_nested_package_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create nested directory structure
    fs::create_dir_all(project_path.join("src/package/subpackage"))?;
    fs::create_dir_all(project_path.join("tests/package/subpackage"))?;

    // Create __init__.py files
    fs::write(project_path.join("src/__init__.py"), "")?;
    fs::write(project_path.join("src/package/__init__.py"), "")?;
    fs::write(project_path.join("src/package/subpackage/__init__.py"), "")?;

    // Create source files
    fs::write(
        project_path.join("src/package/subpackage/deep_module.py"),
        r#"
def deep_function():
    return "deep"

class DeepClass:
    def deep_method(self):
        return "deep_method"
"#,
    )?;

    fs::write(
        project_path.join("src/package/top_module.py"),
        r#"
from .subpackage.deep_module import deep_function

def top_function():
    return f"top_{deep_function()}"
"#,
    )?;

    // Create test files
    fs::write(
        project_path.join("tests/package/subpackage/test_deep_module.py"),
        r#"

from package.subpackage.deep_module import deep_function, DeepClass

def test_deep_function():
    assert deep_function() == "deep"

def test_deep_class():
    obj = DeepClass()
    assert obj.deep_method() == "deep_method"
"#,
    )?;

    fs::write(
        project_path.join("tests/package/test_top_module.py"),
        r#"

from package.top_module import top_function

def test_top_function():
    assert top_function() == "top_deep"
"#,
    )?;

    Ok(temp_dir)
}

/// Creates a project with relative imports
pub fn create_relative_import_project() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(project_path.join("src/package"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create __init__.py files
    fs::write(project_path.join("src/__init__.py"), "")?;
    fs::write(project_path.join("src/package/__init__.py"), "")?;

    // Create source files with relative imports
    fs::write(
        project_path.join("src/package/base.py"),
        r#"
def base_function():
    return "base"

BASE_CONSTANT = "base_value"
"#,
    )?;

    fs::write(
        project_path.join("src/package/consumer.py"),
        r#"
from .base import base_function, BASE_CONSTANT

def consumer_function():
    return f"consumer_{base_function()}"

def get_base_constant():
    return BASE_CONSTANT
"#,
    )?;

    fs::write(
        project_path.join("src/package/sibling.py"),
        r#"
from . import base
from .consumer import consumer_function

def sibling_function():
    return f"sibling_{base.base_function()}"

def use_consumer():
    return consumer_function()
"#,
    )?;

    // Create test files
    fs::write(
        project_path.join("tests/test_base.py"),
        r#"

from package.base import base_function, BASE_CONSTANT

def test_base_function():
    assert base_function() == "base"

def test_base_constant():
    assert BASE_CONSTANT == "base_value"
"#,
    )?;

    fs::write(
        project_path.join("tests/test_consumer.py"),
        r#"

from package.consumer import consumer_function, get_base_constant

def test_consumer_function():
    assert consumer_function() == "consumer_base"

def test_get_base_constant():
    assert get_base_constant() == "base_value"
"#,
    )?;

    fs::write(
        project_path.join("tests/test_sibling.py"),
        r#"

from package.sibling import sibling_function, use_consumer

def test_sibling_function():
    assert sibling_function() == "sibling_base"

def test_use_consumer():
    assert use_consumer() == "consumer_base"
"#,
    )?;

    Ok(temp_dir)
}
