from _pytest.config.argparsing import Parser
from snob.snob import get_tests


def pytest_addoption(parser: Parser) -> None:
    parser.addoption(
        "--snob",
        action="store_true",
        default=False,
        help="Run pytest-snob",
    )


def pytest_collection(session):
    """
    Override the default collection to use a custom list of tests.
    """
    session.items = get_tests(["python_code/package_1/module_1.py"])
