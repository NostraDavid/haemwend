from haemwend.infrastructure.logging import get_logger
from haemwend.sandbox.runtime import SandboxRunner


def main() -> None:
    """Entry point for launching the Panda3D sandbox."""

    logger = get_logger(component="cli")
    runner = SandboxRunner()

    logger.info("sandbox.cli.start")
    try:
        started = runner.launch()
        if not started:
            logger.info("sandbox.cli.disabled")
    except KeyboardInterrupt:  # pragma: no cover - interactive exit
        logger.warning("sandbox.cli.interrupted")
    finally:
        runner.shutdown()
        logger.info("sandbox.cli.exit")


if __name__ == "__main__":
    main()
