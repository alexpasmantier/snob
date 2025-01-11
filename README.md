<div aligne="center">

![snob](./assets/snob.webp | width="600" snob, the picky test selector for python projects")

**Picky test selector for python projects**

</div>

## DISCLAIMER

`snob` is still in an early stage, we'll be adding more polish in the days/weeks to come

## About

For most python projects, running the full test suite for a given PR in a CI is the law of the land.
Because python is not the fastest language, this can sometimes make for a tedious experience (or a costly one
if you're willing to shell out for more workers / parallelization).

The whole behind `snob` is that one should only care about running _relevant_ tests for a given commit, that is
tests covering files that are **impacted** by the changes, either directly or indirectly.

The increase in granularity when selecting the tests allows for faster and less costly CI runs, saving time, money
and headaches for the whole family.

## Features

`snob` leverages the excellent **rust** language to go through your project's dependency graph and determine 
relations between your modules, packages and test files. Using those relations, it then determines for a given
git commit (or range of commits) which files are _impacted_ and which _associated test files_ should be run.

## Installation

To make things as easy as possible for developers, we've conveniently packaged all of `snob`'s goodness in a `pytest`
plugin, which you can directly install using your python packaging tool of choice (ahem... `uv`)

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

Contributions, issues and pull requests are welcome.

## Credits

This project was inspired by your Jenkins / CircleCI monthly bills, your 28 minutes test suite and your flaky failures
on tests that are absolutely irrelevant to your commit.

Also, we'd like to use the occasion to thank all open source maintainers of the excellent rayon, ruffpython_parser,
maturin and pyo3 repositories which enabled us in this project.
