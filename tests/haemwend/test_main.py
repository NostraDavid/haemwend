from pytest_structlog import StructuredLogCapture

from haemwend.main import main


def test_main_logs_start_and_exit(monkeypatch, log: StructuredLogCapture) -> None:
    """main() should emit start and exit events even when the sandbox is disabled."""

    monkeypatch.setattr("haemwend.sandbox.runtime.SandboxRunner.launch", lambda self: False, raising=False)
    monkeypatch.setattr("haemwend.sandbox.runtime.SandboxRunner.shutdown", lambda self: None, raising=False)

    main()

    assert log.has("sandbox.cli.start")
    assert log.has("sandbox.cli.exit")
