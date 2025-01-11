<div align="center">

<img src="https://github.com/alexpasmantier/snob/raw/main/assets/snob.png" width="320" alt="snob, the picky test selector for python projects">

**Picky test selector for python projects**

</div>

## DISCLAIMER

`snob` is still in an early stage, we'll be adding more polish in the days/weeks to come

## About

For most python projects, running the full test suite for a given PR in a CI is the law of the land.
Because python is not the fastest language, this can sometimes make for a tedious experience (or a costly one
if you're willing to shell out for more workers / parallelization).

The whole idea behind `snob` is that one should only care about running _relevant_ tests for a given commit, that is
tests covering files that are **impacted** by the changes, either directly or indirectly.

The increase in granularity when selecting the tests allows for faster and less costly CI runs, saving time, money
and headaches for the whole family.

## Features

`snob` leverages the rust language to go through your project's dependency graph and determine 
relations between your modules, packages and test files. Using those relations, it then determines for a given
git commit (or range of commits) which files are _impacted_ and which _associated test files_ should be run.

## Installation

To make things as easy as possible for developers, `snob`'s goodness is available through a `pytest`
plugin, which you can directly install using your python packaging tool of choice (ahem... `uv`).

This pytest plugin leverage a python package built using the excellent [pyo3](https://github.com/PyO3/pyo3) / [maturin](https://github.com/PyO3/maturin) toolchain.

## Usage

first, install `snob`

```bash
# create a ven and source that venv, then
uv pip install pytest-snob
```
then run pytest with the appropriate arguments

```bash
pytest --commit-range d68ae21..af8acc9

# that's it 🔥
```

## Configuration

`snob` can be configured through a configuration file called `snob.toml` or through a section of
your pre-existing project configuration file `pyproject.toml`

here's an example what your `$GIT_ROOT/snob.toml` configuration file might look like

```toml
[general]
# whether you want to get logs from the rust code hacking away behind the scene
verbosity_level = 2
quiet = false

[files]
# the files listed here won't be considered by snob for import statements (glob format)
# EXAMPLE: suppose you have some configuration files that carry information
# updated on commit or in the CI but this file has no relation to the rest of your codebase
ignores = ["files_to_ignore/**/*.py"]

# the files listed here will trigger all tests on change
# this is for files that are extremely important to you and you want covered
# always all the time for some reason
run-all-tests-on-change = []

[tests]
# the tests listed here will never run (glob format)
ignores = ["some_expensive_test_that_runs_elsewhere.py"]
# the tests listed here will always run (has higher priority than ignores)
always-run = ["tests/mandatory_tests/**/*.py"]
````
## Contributions

Contributions and pull requests are welcome.

So are issues and ideas, but just like when streaming in 8K, mind the bandwidth.

## Credits

This project was inspired by outrageous recurring Jenkins / CircleCI monthly bills and those 38 minutes test suite runs, gnawing at your soul
one irrelevant flaky test failure at a time.

Also, we'd like to use the occasion to thank all the dedicated, passionate and hard-working open source maintainers of the excellent [rayon](https://github.com/rayon-rs/rayon), 
[ruffpython_parser](https://github.com/astral-sh/ruff), [maturin](https://github.com/PyO3/maturin), [pyo3](https://github.com/PyO3/pyo3) amongst all the awesome projects which enabled us in this project. Shoulders, giants, true gratitude to all of you.
