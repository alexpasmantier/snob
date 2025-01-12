# pytest-snob

`pytest-snob` is a `pytest` plugin that leverages the `snob` library (see https://github.com/alexpasmantier/snob) to filter
tests to execute in a given application based on the contents of a range of commits

## Rationale

Most of the time, running your full test suite is a waste of time and resources, since only a portion of the files has changed
since your last CI run / deploy.

By leveraging `snob`, this pytest plugin will accurately determine which tests are relevant for a given changeset and only run those.

## How does it work?

`snob` is a rust library (which we produce a python package from using [Maturin](https://github.com/PyO3/maturin) / [Py03](https://github.com/PyO3))
that efficiently parses the dependency graph of your python codebase and as such, can accurately determine which files are _impacted_ by your changes.

For more information, see [the snob github repository](https://github.com/alexpasmantier/snob)
