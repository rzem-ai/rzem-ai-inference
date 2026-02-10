import webview

from backend.api.system import SystemAPI
from backend.config import AppConfig
from backend.services.app_service import AppService


def main() -> None:
    config = AppConfig()
    service = AppService()
    api = SystemAPI(service)

    webview.create_window(
        config.window_title,
        url=config.entry_url,
        js_api=api,
        width=config.window_width,
        height=config.window_height,
        min_size=(config.min_width, config.min_height),
    )
    webview.start(debug=config.dev_mode)


if __name__ == "__main__":
    main()
