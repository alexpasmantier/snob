import os
import subprocess
from pathlib import Path

import pytest
from snob_lib import get_tests


def get_modified_files(main_branch_name: str = "master") -> list[str]:
    """
    Get a list of files modified by the given commit using `git diff`.
    """
    try:
        result = subprocess.run(
            # git diff main --name-only && git ls-files --modified
            [
                "git",
                "diff",
                "--name-only",
                main_branch_name,
                "&&",
                "git",
                "ls-files",
                "--modified",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
            text=True,
        )
        modified_files = result.stdout.splitlines()
        return [os.path.abspath(file) for file in modified_files]
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"Failed to retrieve modified files: {e.stderr.strip()}")


# TODO: handle errors nicely
# TODO: sensible default
def pytest_addoption(parser):
    group = parser.getgroup("snob")
    group.addoption(
        "--snob",
        action="store",
        dest="use_snob",
        default=False,
        help="Enable the snob plugin",
    )


@pytest.hookimpl(tryfirst=True)
def pytest_collection(session: pytest.Session):
    if session.config.getoption("use_snob"):
        modified_files = get_modified_files()
        print(
            f"🧐 \x1b[92;3;4mSnob plugin:\x1b[m Found \x1b[91m{len(modified_files)}\x1b[m modified file(s)"
        )
        test_files = get_tests(modified_files)
        print("")
        print(
            f"🧐 \x1b[92;3;4mSnob plugin:\x1b[m Selected \x1b[91m{len(test_files)}\x1b[m test file(s)"
        )
        session.perform_collect((Path(f) for f in test_files))

        return True
