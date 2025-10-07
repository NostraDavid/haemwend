from pytest_structlog import StructuredLogCapture
from haemwend.main import main


def test_main_logs(log: StructuredLogCapture):
    """main() should emit a structlog event with the greeting."""
    main()
    assert log.has("Hello, World!")
