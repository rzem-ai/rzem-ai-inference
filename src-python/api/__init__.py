"""
API package for pywebview bridge.

Composes the Api class from domain-specific mixins. Each mixin adds methods
that pywebview exposes to the JavaScript frontend as window.pywebview.api.*.

ApiBase provides __init__, _run_async, health_check, init_database, poll_events.
All mixins access self._app_state and self._run_async through the MRO.
"""

from api.base import ApiBase
from api.queue import QueueApiMixin
from api.gallery import GalleryApiMixin
from api.folders import FolderApiMixin
from api.tags import TagApiMixin
from api.settings import SettingsApiMixin
from api.styles import StyleApiMixin
from api.models import ModelApiMixin
from api.system import SystemApiMixin
from api.batch import BatchApiMixin


class Api(
    QueueApiMixin,
    GalleryApiMixin,
    FolderApiMixin,
    TagApiMixin,
    SettingsApiMixin,
    StyleApiMixin,
    ModelApiMixin,
    SystemApiMixin,
    BatchApiMixin,
):
    """
    Unified API class exposed to JavaScript frontend via pywebview.

    All public methods are automatically available as:
        await window.pywebview.api.method_name(args)

    Methods are organized into domain-specific mixins for maintainability.
    """
    pass
