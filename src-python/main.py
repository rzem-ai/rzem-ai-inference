#!/usr/bin/env python3
"""
Main entry point for rzem-ai-inference with pywebview.

This replaces the Tauri desktop wrapper with pywebview.
"""

import argparse
import asyncio
import sys
import threading
from pathlib import Path
from loguru import logger
import webview

from app_state import AppState
from api import Api
from shared import RuntimeConfig, RuntimeMode


# Configure logging
logger.remove()  # Remove default handler
logger.add(
    sys.stderr,
    format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
    level="INFO",
)


def parse_args():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(
        prog="rzem-ai-inference",
        description="Local AI Image Generation with GPU Sharing",
    )

    parser.add_argument(
        "--server",
        action="store_true",
        help="Enable server mode (expose REST/WebSocket API)",
    )

    parser.add_argument(
        "--headless",
        action="store_true",
        help="Run in headless mode (no desktop UI, server mode only)",
    )

    parser.add_argument(
        "--client",
        action="store_true",
        help="Connect to remote server (client mode)",
    )

    parser.add_argument(
        "--server-url",
        type=str,
        help="Server URL for client mode (e.g., http://192.168.1.100:8080)",
    )

    parser.add_argument(
        "-p", "--port",
        type=int,
        default=8080,
        help="Port for server mode (default: 8080)",
    )

    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode",
    )

    return parser.parse_args()


async def initialize_app_async(app_state: AppState) -> None:
    """Initialize app state asynchronously"""
    await app_state.initialize()


def run_local_mode(runtime_config: RuntimeConfig, debug: bool = False) -> None:
    """
    Run in local mode with desktop UI.

    This is the default mode - desktop app with local GPU inference.
    """
    logger.info("Starting in local mode")

    # Create app state
    app_state = AppState(runtime_config)

    # Create API instance
    api = Api(app_state)

    # Setup event loop in a separate thread
    loop = asyncio.new_event_loop()

    def run_event_loop():
        asyncio.set_event_loop(loop)
        loop.run_forever()

    event_thread = threading.Thread(target=run_event_loop, daemon=True)
    event_thread.start()

    # Set event loop for API
    api.set_event_loop(loop)

    # Initialize app state
    future = asyncio.run_coroutine_threadsafe(initialize_app_async(app_state), loop)
    try:
        future.result(timeout=30)  # Wait up to 30 seconds for initialization
    except Exception as e:
        logger.error(f"Failed to initialize app state: {e}")
        sys.exit(1)

    # Find the frontend files
    # Try to locate the dist folder (built frontend)
    script_dir = Path(__file__).parent.parent
    frontend_dirs = [
        script_dir / "dist",  # Production build
        script_dir / "src",   # Development
    ]

    frontend_path = None
    for dir_path in frontend_dirs:
        if dir_path.exists():
            frontend_path = dir_path
            break

    if not frontend_path:
        logger.error("Frontend files not found. Please build the frontend first.")
        logger.error(f"Searched in: {[str(d) for d in frontend_dirs]}")
        sys.exit(1)

    logger.info(f"Loading frontend from: {frontend_path}")

    # Determine entry point
    if (frontend_path / "index.html").exists():
        entry_point = str(frontend_path / "index.html")
    else:
        logger.error("index.html not found in frontend directory")
        sys.exit(1)

    # Create window
    window = webview.create_window(
        title="RZEM AI Inference",
        url=entry_point,
        js_api=api,
        width=1280,
        height=800,
        resizable=True,
    )

    # Start webview
    webview.start(debug=debug)

    # Cleanup
    logger.info("Shutting down...")
    future = asyncio.run_coroutine_threadsafe(app_state.shutdown(), loop)
    try:
        future.result(timeout=10)
    except Exception as e:
        logger.error(f"Error during shutdown: {e}")

    loop.call_soon_threadsafe(loop.stop)
    event_thread.join(timeout=5)


def run_server_mode(runtime_config: RuntimeConfig, port: int, headless: bool = False) -> None:
    """
    Run in server mode (REST API + WebSocket).

    Args:
        runtime_config: Runtime configuration
        port: Port to listen on
        headless: If True, run without UI
    """
    logger.info(f"Starting in server mode on port {port}")
    logger.info(f"API will be available at: http://localhost:{port}/api/v1")
    logger.info(f"WebSocket at: ws://localhost:{port}/api/v1/ws")
    logger.warning("WARNING: No authentication enabled. Use on trusted networks only!")

    if headless:
        logger.warning("Headless mode not yet fully implemented")

    # TODO: Implement server mode with FastAPI/aiohttp
    logger.error("Server mode not yet implemented in Python backend")
    sys.exit(1)


def run_client_mode(runtime_config: RuntimeConfig) -> None:
    """
    Run in client mode (connect to remote server).

    Args:
        runtime_config: Runtime configuration with server URL
    """
    logger.info(f"Starting in client mode, connecting to: {runtime_config.server_url}")

    # TODO: Implement client mode
    logger.error("Client mode not yet implemented in Python backend")
    sys.exit(1)


def main():
    """Main entry point"""
    args = parse_args()

    # Configure debug logging
    if args.debug:
        logger.remove()
        logger.add(
            sys.stderr,
            format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
            level="DEBUG",
        )

    # Determine runtime mode
    if args.client:
        if not args.server_url:
            logger.error("--server-url is required when using --client")
            sys.exit(1)

        runtime_config = RuntimeConfig.client(args.server_url)
        run_client_mode(runtime_config)

    elif args.server:
        runtime_config = RuntimeConfig.server(args.port)
        run_server_mode(runtime_config, args.port, args.headless)

    else:
        # Local mode (default)
        runtime_config = RuntimeConfig.local()
        run_local_mode(runtime_config, debug=args.debug)


if __name__ == "__main__":
    main()
