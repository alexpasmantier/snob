import os
import subprocess

from _pytest.nodes import Item
from snob_lib import get_tests


def get_modified_files(commit_range: str) -> list[str]:
    """
    Get a list of files modified by the given commit using `git diff`.
    """
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", commit_range],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
            text=True,
        )
        modified_files = result.stdout.splitlines()
        return [os.path.abspath(file) for file in modified_files]
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"Failed to retrieve modified files: {e.stderr.strip()}")


def pytest_addoption(parser):
    group = parser.getgroup("snob")
    group.addoption(
        "--commit-range",
        action="store",
        dest="commit_range",
        default=None,
        help="Commit hash to get the list of modified files",
    )

def pytest_collection_modifyitems(session, config, items: list[Item]):
    commit_range = config.getoption("commit_range")
    if commit_range is not None:
        # NOTE: the test files will be retrieved using `snob_lib` here
        # based on the dependency graph of the application
        test_files = get_tests(get_modified_files(commit_range))
        # this is because a character is eaten at the beginning of next line without it
        print("")
        print(
            f"🧐 \x1b[92;3;4mSnob plugin:\x1b[m Selected \x1b[91m{len(test_files)}\x1b[m file(s)"
        )
        # compute some sort of intersection between the files picked by pytest
        # and the files picked by snob
        pytest_selected = {item for item in items if item.fspath.strpath in test_files}
        for item in pytest_selected:
            print(f"  - {item.nodeid}")

        pytest_deselected = set(items) - pytest_selected

        config.hook.pytest_deselected(items=[t for t in pytest_deselected])
        # mutation seems necessary here
        items[:] = [t for t in pytest_selected]
