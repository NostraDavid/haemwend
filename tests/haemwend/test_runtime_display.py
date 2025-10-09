import pytest

import haemwend.sandbox.runtime as runtime
from haemwend.sandbox.runtime import SandboxRunner


def test_create_application_falls_back_to_tiny_display(monkeypatch):
    runner = SandboxRunner()
    call_count: dict[str, int] = {"count": 0}

    class FakeApp:
        def __init__(self) -> None:
            call_count["count"] += 1
            if call_count["count"] == 1:
                raise Exception("No graphics pipe is available! missing libGL")

    fallback_calls: dict[str, int] = {"count": 0}

    def fake_configure(self: SandboxRunner) -> str:
        fallback_calls["count"] += 1
        return "mock-display"

    monkeypatch.setattr(runtime, "SandboxApp", FakeApp)
    monkeypatch.setattr(SandboxRunner, "_configure_software_display", fake_configure, raising=False)

    app, error = runner._create_application()

    assert isinstance(app, FakeApp)
    assert error is None
    assert call_count["count"] == 2
    assert fallback_calls["count"] == 1
    assert runner._display_backend == "mock-display"


def test_create_application_returns_error_when_no_fallback_available(monkeypatch):
    runner = SandboxRunner()

    class FailingApp:
        def __init__(self) -> None:
            raise Exception("No graphics pipe is available! no backend")

    monkeypatch.setattr(runtime, "SandboxApp", FailingApp)
    monkeypatch.setattr(SandboxRunner, "_configure_software_display", lambda self: None, raising=False)

    app, error = runner._create_application()

    assert app is None
    assert error is not None
    assert "No graphics pipe is available" in error
    assert runner._display_backend is None


def test_create_application_re_raises_for_non_display_errors(monkeypatch):
    runner = SandboxRunner()

    class OtherErrorApp:
        def __init__(self) -> None:
            raise RuntimeError("application configuration failed")

    monkeypatch.setattr(runtime, "SandboxApp", OtherErrorApp)
    monkeypatch.setattr(SandboxRunner, "_configure_software_display", lambda self: "mock-display", raising=False)

    with pytest.raises(RuntimeError):
        runner._create_application()


def test_is_graphics_pipe_error_handles_window_open_failure():
    runner = SandboxRunner()

    assert runner._is_graphics_pipe_error(Exception("Could not open window."))
    assert runner._is_graphics_pipe_error(Exception("Unable to open 'onscreen' window."))


def test_configure_window_skips_when_properties_unavailable():
    app = runtime.SandboxApp.__new__(runtime.SandboxApp)
    app._title = "Test"
    app.win = object()

    runtime.SandboxApp._configure_window(app)
