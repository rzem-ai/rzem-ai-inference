"""Utility helpers for storing style images locally with thumbnails."""

from __future__ import annotations

import logging
import uuid
from pathlib import Path

import requests
from PIL import Image

logger = logging.getLogger(__name__)

_THUMBNAIL_QUALITY = 85
_THUMBNAIL_MAX_SIDE = 512


def store_style_image(
    source: str,
    styles_dir: Path,
    style_id: str = "",
) -> tuple[str, str] | None:
    """Download or copy an image into the styles directory.

    Returns (full_path, thumbnail_path) or None on failure.
    ``source`` may be an HTTP(S) URL or a local file path.
    """
    styles_dir.mkdir(parents=True, exist_ok=True)
    prefix = f"{style_id}_" if style_id else ""
    stem = f"{prefix}{uuid.uuid4().hex[:12]}"

    try:
        if source.startswith("http://") or source.startswith("https://"):
            image = _download_image(source)
        else:
            path = Path(source)
            if not path.is_file():
                logger.warning("Source file not found: %s", source)
                return None
            image = Image.open(path)
            image.load()
    except Exception as e:
        logger.warning("Failed to load image from %s: %s", source, e)
        return None

    # Save full resolution PNG
    full_path = styles_dir / f"{stem}.png"
    image.save(str(full_path), format="PNG")

    # Save half-res WebP thumbnail (cap at _THUMBNAIL_MAX_SIDE)
    ratio = min(_THUMBNAIL_MAX_SIDE / max(image.width, image.height, 1), 1.0)
    thumb_w = max(int(image.width * ratio), 1)
    thumb_h = max(int(image.height * ratio), 1)
    thumb = image.resize((thumb_w, thumb_h), Image.LANCZOS)
    thumb_path = styles_dir / f"{stem}_thumb.webp"
    thumb.save(str(thumb_path), format="WEBP", quality=_THUMBNAIL_QUALITY)

    logger.info("Stored style image %s → %s + %s", source[:80], full_path, thumb_path)
    return str(full_path), str(thumb_path)


def _download_image(url: str) -> Image.Image:
    """Download an image from a URL and return a PIL Image."""
    resp = requests.get(url, timeout=30, stream=True)
    resp.raise_for_status()
    image = Image.open(resp.raw)
    image.load()
    return image
