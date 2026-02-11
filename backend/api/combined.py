"""Combined API that merges all sub-APIs into a single pywebview js_api object."""

from __future__ import annotations

from backend.api import ApiMeta
from backend.api.bundles import BundlesAPI
from backend.api.gallery import GalleryAPI
from backend.api.inference import InferenceAPI
from backend.api.system import SystemAPI


class CombinedAPI(SystemAPI, InferenceAPI, BundlesAPI, GalleryAPI, metaclass=ApiMeta):
    """Single js_api class that exposes every backend method to the frontend.

    pywebview accepts one ``js_api`` object per window, so we merge all
    API classes via multiple inheritance. Each parent's ``__init__`` is
    called explicitly in ``__init__`` below.
    """

    def __init__(self, service, inference, bundle_store, db) -> None:
        SystemAPI.__init__(self, service)
        InferenceAPI.__init__(self, inference)
        BundlesAPI.__init__(self, bundle_store)
        GalleryAPI.__init__(self, db)

    def health_check(self) -> dict:
        return {"status": "ok"}
