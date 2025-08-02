<div align="center">

<h1>🧐 Snob</h1>

_Only run tests that matter, saving time and resources._

[![Rust](https://img.shields.io/badge/rust-1.88+-green.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![PyPI](https://img.shields.io/pypi/v/pytest-snob.svg)](https://pypi.org/project/pytest-snob/)

</div>

Snob speeds up your development workflow and reduces CI testing costs dramatically by analyzing your Python project's dependency graph to intelligently select which tests to run based on code changes.

## 🚀 Quick Start

### Installation

**🚀 Quick Install (Recommended)**

```bash
curl -sSL https://raw.githubusercontent.com/alexpasmantier/snob/main/install.sh | bash
```

**Standalone CLI**

```bash
cargo install snob
```

**Pytest Plugin**

```bash
pip install pytest-snob
```

### Basic Usage (CLI)

Snob can be used as a standalone CLI tool and works best when paired with a version control system like Git and a Python testing framework (e.g. pytest).

The most common usage is to run Snob with your changed files to get a list of affected tests:

```bash
snob $(git diff --name-only)  # lists tests affected by your changes

# tests/test_file_1.py
# tests/test_file_2.py
# tests/test_file_3.py
```

And then use those results as input to your test runner:

```bash
snob $(git diff --name-only) | xargs pytest

# INFO snob: Analyzed 405 files in 8.513462ms
# INFO snob: Found 27/124 impacted tests
# ============ test session starts ============
# ... collected 27 items
```

**Using Snob with Pytest**

Snob can also be used as a pytest plugin to automatically select tests based on your code changes.

```bash
# Test changes since a specific commit
pytest --commit-range d68ae21..af8acc9

# Test changes since main branch
pytest --commit-range main..HEAD
```

## ⚙️ Configuration

Snob configuration can either be loaded from:

- a `snob.toml` file in your project root
- a `[tool.snob]` section in your `pyproject.toml`

<details>
<summary>configuration options</summary>

```toml
[general]
# Logging verbosity (0=error, 1=warn, 2=info, 3=debug, 4=trace)
verbosity_level = 2
# Whether to disable all logging output
quiet = false

[files]
# The files listed here will be ignored by snob when crawling the workspace.
# This can be useful for excluding generated files, migrations, or scripts that don't affect the project's dependency graph.
ignores = [
    "migrations/**/*.py",
    "scripts/**/*.py",
    "**/generated_*.py"
]

# The files listed here will trigger all tests when changed.
# This is useful for critical files like `conftest.py`, `pytest.ini`, or `requirements.txt` for which you want to
# rerun the entire test suite.
run-all-tests-on-change = [
    "conftest.py",
    "pytest.ini",
    "requirements.txt"
]

[tests]
# These test files will always be run, regardless of changes.
# This is useful for health checks, smoke tests, or critical tests that should always run.
always-run = [
    "tests/health_check.py",
    "tests/smoke_test.py"
]

# These test files will never be run automatically by snob, but can still be run manually.
# This can be useful for long-running tests, integration tests, or tests that require special setup which you do not
# wish to run without deciding to do so explicitly.
ignores = [
"tests/slow/**/*.py",
"tests/integration/external_api_*.py"
]
```

**Alternative: Use `pyproject.toml`**

Same format as above, but placed under the `[tool.snob]` section:

```toml
[tool.snob]
verbosity_level = 1

[tool.snob.files]
ignores = ["migrations/**/*.py"]

[tool.snob.tests]
always-run = ["tests/smoke_test.py"]
```

</details>

## 🧪 Understanding Test Selection

Snob analyzes your codebase to build a dependency graph of files and tests. It uses this graph to determine which tests
are affected by changes in your code.

This graph can be printed out in a visual format using Graphviz, which can help you understand how your code and tests
are related.

```bash
# Generate a dependency graph of your codebase and dump it to `deps.dot`
snob --dot-graph deps.dot $(git diff --name-only)

# Convert the dot file to a PNG image using Graphviz
dot -Tpng deps.dot -Ksfdp -o graph.png
```

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## 📊 Performance

Snob is fast enough to be entirely transparent in your workflow.

On modern hardware, it should be able to handle 1M+ loc codebases in as little as 100ms which is faster than most test
runners even need to initialize.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**⭐ [Star us on GitHub](https://githugb.com/alexpasmantier/snob) • 🐛 [Report Issues](https://github.com/alexpasmantier/snob/issues) • 🤝 [Contribute](https://github.com/alexpasmantier/snob/CONTRIBUTING.md)**

</div>
