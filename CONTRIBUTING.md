# 🤝 Contributing to Snob

Thanks for your interest in contributing to Snob! We welcome contributions of all kinds - from bug reports to code improvements to documentation updates.

## 🚀 Quick Start for Contributors

**New to the project?** Start here:

1. **🍴 Fork** the repository on GitHub
2. **📥 Clone** your fork: `git clone https://github.com/yourusername/snob`
3. **🔧 Set up** development environment (see below)
4. **🌟 Pick an issue** from our [good first issue](https://github.com/your-org/snob/labels/good%20first%20issue) list
5. **💻 Make changes** and add tests
6. **📝 Submit** a pull request

## 🏗️ Project Overview

Snob is a Rust-based tool that analyzes Python dependency graphs to selectively run tests. It consists of:

- **🦀 Rust CLI** (`src/`) - Core analysis engine (this is where most development happens)
- **🐍 Python Library** (`snob_lib`) - PyO3 bindings for Python integration  
- **🧪 Pytest Plugin** (`pytest-snob/`) - User-facing pytest integration

## 🛠️ Development Setup

### Prerequisites

- **Rust 1.81+** (automatically installed via `rust-toolchain.toml`)
- **Python 3.9+** 
- **Git** for version control

### First-Time Setup

```bash
# 1. Clone the repository
git clone https://github.com/your-org/snob
cd snob

# 2. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Build the project
cargo build

# 4. Run tests to make sure everything works
cargo test

# 5. (Optional) For Python development
pip install maturin
```

**That's it!** You're ready to contribute.

## ⚡ Development Commands

### 🦀 Rust Development (Most Common)

```bash
# Check if your code compiles (fast)
cargo check

# Run all tests
cargo test

# Run specific tests
cargo test test_name

# Format code (always do this before committing)
cargo fmt

# Check for common issues
cargo clippy

# Build optimized version
cargo build --release

# Run the CLI tool
cargo run -- --help
cargo run -- src/example.py
```

### 🧪 Testing Your Changes

```bash
# Run all tests (including new integration tests)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run single-threaded (useful for debugging)
cargo test -- --test-threads=1

# Run specific test file
cargo test --test integration_test_name

# Run with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture
```

### 🐍 Python Integration (Advanced)

Only needed if you're working on the Python bindings:

```bash
# Build Python library
maturin develop

# Test Python integration
python -c "import snob_lib; print('Works!')"

# Build for distribution
maturin build --release
```

## Architecture Overview

### Core Components

- `main.rs` - CLI entry point
- `lib.rs` - PyO3 library interface (`get_tests()` function)
- `config.rs` - Configuration management
- `fs.rs` - File system operations and workspace crawling
- `graph.rs` - Dependency graph construction and impact analysis
- `ast.rs` - Python AST parsing using ruff
- `results.rs` - Test result filtering and categorization

### Analysis Pipeline

1. **File Discovery** - Crawl workspace for Python files
2. **Dependency Graph** - Parse imports to build module relationships
3. **Impact Analysis** - Determine transitively affected files
4. **Test Selection** - Filter for relevant test files
5. **Result Filtering** - Apply configuration rules

## Testing

Currently tests are minimal but growing:
- Rust unit tests: `cargo test`
- Python integration tests: `cd pytest-snob && python -m pytest`

## Configuration

Snob uses a hierarchical configuration system:
1. CLI arguments (highest priority)
2. `snob.toml` 
3. `[tool.snob]` in `pyproject.toml`

## Release Process

The project uses dual CI workflows:
- `.github/workflows/maturin-ci.yml` - Builds `snob_lib` (tag: `lib@v*`)
- `.github/workflows/pytest-ci.yml` - Builds `pytest-snob` (tag: `plugin@v*`)

## Key Dependencies

- `ruff_python_parser` - Fast Python AST parsing
- `pyo3` - Python-Rust interop
- `rayon` - Parallel processing
- `globset` - File pattern matching

## Getting Help

- Check existing issues for similar problems
- Create detailed bug reports with examples
- For questions about the algorithm, see the dependency graph logic in `graph.rs`

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Pass clippy lints (`cargo clippy`)
- Write clear commit messages
- Keep changes focused and atomic