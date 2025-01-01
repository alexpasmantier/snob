# snob 🍵

`snob` is a tool that allows developers to select tests to execute for a given commit. The idea behind it is that most tests are executed for no reason most of the time.



the rust repo defines two things:

- a lib called `snob_lib`
- a binary called `snob`

a python package named `snob_lib` is produced using `Maturin` (it's the python version of the rust lib)

we then define a `pytest` plugin that leverages `snob_lib` (especially the exposed `get_tests` function)
