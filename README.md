<div align="center">
    <img width="1280" height="329" alt="snob-logo" src="https://github.com/user-attachments/assets/35b937de-2cee-4f7e-b399-0cbe92f77c35" />

_Only run tests that matter, saving time and resources._

![Static Badge](https://img.shields.io/badge/alpha-cyan?label=state)
[![Rust](https://img.shields.io/badge/rust-1.88+-green.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![PyPI](https://img.shields.io/pypi/v/pytest-snob.svg)](https://pypi.org/project/pytest-snob/)

</div>

## 📖 Rationale

Most of the time, running your full test suite is a waste of time and resources, since only a portion of the files has changed since your last commit / CI run / deploy.

Snob speeds up your development workflow dramatically by analyzing your Python project's dependency graph to intelligently select which tests to run based on code changes.

## What Snob isn't
Snob doesn’t predict failures—it selects tests based on static import dependencies.

It’s aimed at reducing locally run test suite size dramatically (often skipping 99% of irrelevant tests).

It’s not intended to replace CI or full regression testing, but to speed up feature development cycles in large codebases.

Limitations include missing dynamic imports, runtime side-effects, or implicit import behavior.

## 🚀 Quick Start

### Installation

```bash
curl -sSL https://raw.githubusercontent.com/alexpasmantier/snob/main/install.sh | bash
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
dot -Tsvg deps.dot -Ksfdp -o graph.svg
```

_`graph.svg`_

<div align="left">
    <img width="600" alt="Screenshot From 2025-08-03 00-01-58" src="https://github.com/user-attachments/assets/35e6c73f-1968-4170-b736-7a7c979b443d" />
</div>

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## 📊 Performance

Snob is fast. On modern hardware, it should handle million line Python codebases with thousands of tests in a matter of milliseconds, making it disappear into the background of your development workflow.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**⭐ [Star us on GitHub](https://githugb.com/alexpasmantier/snob) • 🐛 [Report Issues](https://github.com/alexpasmantier/snob/issues) • 🤝 [Contribute](CONTRIBUTING.md)**

</div>
