import os
import sys
from dataclasses import dataclass, field
from pathlib import Path


def _get_base_dir() -> Path:
    """Return base directory, handling PyInstaller bundled mode."""
    if getattr(sys, "frozen", False):
        return Path(sys._MEIPASS)
    return Path(__file__).resolve().parent.parent


@dataclass(frozen=True)
class AppConfig:
    dev_mode: bool = field(
        default_factory=lambda: os.environ.get("DEV_MODE", "").lower() in ("1", "true", "yes")
    )
    base_dir: Path = field(default_factory=_get_base_dir)
    window_title: str = "rzem.ai.inference"
    window_width: int = 1800
    window_height: int = 1200
    window_x: int = 10
    window_y: int = 100
    min_width: int = 800
    min_height: int = 600
    vite_dev_url: str = "http://localhost:1978"
    data_dir: Path = field(
        default_factory=lambda: Path.home() / ".rzem-ai"
    )
    output_dir: Path = field(
        default_factory=lambda: Path.home() / ".rzem-ai" / "output"
    )
    styles_dir: Path = field(
        default_factory=lambda: Path.home() / ".rzem-ai" / "styles"
    )

    @property
    def entry_url(self) -> str:
        if self.dev_mode:
            return self.vite_dev_url
        return str(self.base_dir / "frontend" / "dist" / "index.html")
