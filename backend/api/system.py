from backend.services.app_service import AppService


class SystemAPI:
    """js_api bridge class exposed to the frontend via pywebview.

    Every public method becomes callable as window.pywebview.api.<method>().
    Keep this as a thin facade — delegate logic to services.
    """

    def __init__(self, service: AppService) -> None:
        self._service = service

    def get_system_info(self) -> dict:
        return self._service.get_system_info()

    def greet(self, name: str) -> str:
        return self._service.greet(name)

    def increment_counter(self) -> int:
        return self._service.increment_counter()

    def get_counter(self) -> int:
        return self._service.get_counter()
