"""SSL verification utilities for corporate proxy environments.

When running behind a corporate SSL proxy that intercepts HTTPS with its own
certificate, SSL verification will fail for all outbound requests. This module
provides a way to globally disable SSL verification as an escape hatch.

Affects: requests, urllib3, httpx, anthropic SDK, huggingface_hub downloads,
and any library that respects the standard environment variables.
"""

from __future__ import annotations

import logging
import os
import ssl
import warnings

logger = logging.getLogger(__name__)

_ssl_disabled = False


def is_ssl_verification_disabled() -> bool:
    """Return whether SSL verification is currently disabled."""
    return _ssl_disabled


def disable_ssl_verification() -> None:
    """Globally disable SSL certificate verification.

    Sets environment variables and patches ssl defaults so that all HTTP
    libraries (requests, httpx, urllib3, huggingface_hub, anthropic SDK, etc.)
    skip certificate validation.

    WARNING: This makes all HTTPS connections vulnerable to MITM attacks.
    Only use in trusted corporate network environments with SSL-intercepting
    proxies.
    """
    global _ssl_disabled

    if _ssl_disabled:
        return

    logger.warning("Disabling SSL certificate verification (corporate proxy mode)")

    # 1. Environment variables respected by requests, pip, curl, etc.
    os.environ["CURL_CA_BUNDLE"] = ""
    os.environ["REQUESTS_CA_BUNDLE"] = ""

    # huggingface_hub: disable SSL for model downloads
    os.environ["HF_HUB_DISABLE_SSL_VERIFY"] = "1"

    # 2. Patch ssl module defaults so urllib, websocket-client, etc. skip verification
    ssl._create_default_https_context = ssl._create_unverified_context

    # 3. Suppress the InsecureRequestWarning from urllib3
    try:
        from urllib3.exceptions import InsecureRequestWarning
        warnings.filterwarnings("ignore", category=InsecureRequestWarning)
    except ImportError:
        pass

    # 4. Monkey-patch requests.Session to default verify=False
    try:
        import requests

        _orig_init = requests.Session.__init__

        def _patched_init(self, *args, **kwargs):
            _orig_init(self, *args, **kwargs)
            self.verify = False

        requests.Session.__init__ = _patched_init  # type: ignore[method-assign]
    except ImportError:
        pass

    # 5. Monkey-patch httpx.Client and httpx.AsyncClient to default verify=False
    #    This is critical for the Anthropic SDK and any other library using httpx,
    #    since httpx creates its own SSL context and ignores ssl._create_default_https_context
    try:
        import httpx

        _orig_httpx_client_init = httpx.Client.__init__
        _orig_httpx_async_client_init = httpx.AsyncClient.__init__

        def _patched_httpx_client_init(self, *args, **kwargs):
            kwargs.setdefault("verify", False)
            _orig_httpx_client_init(self, *args, **kwargs)

        def _patched_httpx_async_client_init(self, *args, **kwargs):
            kwargs.setdefault("verify", False)
            _orig_httpx_async_client_init(self, *args, **kwargs)

        httpx.Client.__init__ = _patched_httpx_client_init  # type: ignore[method-assign]
        httpx.AsyncClient.__init__ = _patched_httpx_async_client_init  # type: ignore[method-assign]
    except ImportError:
        pass

    _ssl_disabled = True
    logger.info("SSL certificate verification disabled globally")


def enable_ssl_verification() -> None:
    """Re-enable SSL certificate verification (best effort).

    Note: Some patches (like ssl._create_default_https_context) are difficult
    to fully reverse. A restart is recommended after re-enabling.
    """
    global _ssl_disabled

    if not _ssl_disabled:
        return

    logger.info("Re-enabling SSL certificate verification")

    # Remove environment overrides
    os.environ.pop("CURL_CA_BUNDLE", None)
    os.environ.pop("REQUESTS_CA_BUNDLE", None)
    os.environ.pop("HF_HUB_DISABLE_SSL_VERIFY", None)

    # Restore ssl default context
    ssl._create_default_https_context = ssl.create_default_context

    _ssl_disabled = False
    logger.info("SSL certificate verification re-enabled (restart recommended for full effect)")
