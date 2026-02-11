"""Model bundle data model, default definitions, and JSON-file persistence."""

from __future__ import annotations

import json
import logging
from dataclasses import asdict, dataclass, field, fields
from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class ModelBundle:
    id: str
    label: str
    description: str
    transformer_type: str  # "flux1_dev" | "flux2_dev" | "z_image" | "qwen_image"
    tier: str  # "performance" | "balanced" | "quality"
    transformer_model: str
    vae_model: str
    clip_tokenizer: str | None = None
    clip_encoder: str | None = None
    t5_tokenizer: str | None = None
    t5_encoder: str | None = None
    qwen3_tokenizer: str | None = None
    qwen3_encoder: str | None = None
    steps: int = 20
    cfg_scale: float = 1.0
    sampler: str = "euler"
    scheduler: str = "normal"
    vram_estimate_gb: float = 0.0
    is_default: bool = True


# ── Shared encoder repos ──

_CLIP = "openai/clip-vit-large-patch14"
_T5 = "google/t5-v1_1-xxl"
_QWEN3 = "Qwen/Qwen3-0.6B"

DEFAULT_BUNDLES: list[ModelBundle] = [
    # ── FLUX.1 Dev ──
    ModelBundle(
        id="flux1_dev_performance",
        label="FLUX.1 Dev — Fast",
        description="FLUX.1-dev Q4 quantized — lower VRAM, faster inference",
        transformer_type="flux1_dev",
        tier="performance",
        transformer_model="city96/FLUX.1-dev-gguf/flux1-dev-Q4_K_S.gguf",
        vae_model="black-forest-labs/FLUX.1-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        steps=28,
        cfg_scale=3.5,
        vram_estimate_gb=17.7,
    ),
    ModelBundle(
        id="flux1_dev_balanced",
        label="FLUX.1 Dev — Balanced",
        description="FLUX.1-dev Q8 quantized — good quality/VRAM trade-off",
        transformer_type="flux1_dev",
        tier="balanced",
        transformer_model="city96/FLUX.1-dev-gguf/flux1-dev-Q8_0.gguf",
        vae_model="black-forest-labs/FLUX.1-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        steps=30,
        cfg_scale=3.5,
        vram_estimate_gb=22.7,
    ),
    # Quality last — safest default (no GGUF dependency)
    ModelBundle(
        id="flux1_dev_quality",
        label="FLUX.1 Dev — Quality",
        description="FLUX.1-dev BF16 full precision — best quality",
        transformer_type="flux1_dev",
        tier="quality",
        transformer_model="black-forest-labs/FLUX.1-dev",
        vae_model="black-forest-labs/FLUX.1-dev",
        clip_tokenizer=_CLIP,
        clip_encoder=_CLIP,
        t5_tokenizer=_T5,
        t5_encoder=_T5,
        steps=40,
        cfg_scale=4.0,
        vram_estimate_gb=33.7,
    ),
    # ── FLUX.2 Dev ──
    ModelBundle(
        id="flux2_dev_quality",
        label="FLUX.2 Dev",
        description="FLUX.2-dev BF16 with Qwen3 text encoder",
        transformer_type="flux2_dev",
        tier="quality",
        transformer_model="black-forest-labs/FLUX.2-dev",
        vae_model="black-forest-labs/FLUX.2-dev",
        qwen3_tokenizer=_QWEN3,
        qwen3_encoder=_QWEN3,
        steps=28,
        cfg_scale=1.0,
        vram_estimate_gb=23.2,
    ),
    # ── Z-Image ──
    ModelBundle(
        id="z_image_quality",
        label="Z-Image",
        description="Z-Image BF16 with Qwen3 text encoder",
        transformer_type="z_image",
        tier="quality",
        transformer_model="rzem-ai/z-image",
        vae_model="rzem-ai/z-image",
        qwen3_tokenizer=_QWEN3,
        qwen3_encoder=_QWEN3,
        steps=28,
        cfg_scale=1.0,
        vram_estimate_gb=20.2,
    ),
    # ── Qwen-Image ──
    ModelBundle(
        id="qwen_image_quality",
        label="Qwen-Image",
        description="Qwen-Image BF16 with Qwen3 text encoder",
        transformer_type="qwen_image",
        tier="quality",
        transformer_model="rzem-ai/qwen-image",
        vae_model="rzem-ai/qwen-image",
        qwen3_tokenizer=_QWEN3,
        qwen3_encoder=_QWEN3,
        steps=28,
        cfg_scale=1.0,
        vram_estimate_gb=48.2,
    ),
]


class BundleStore:
    """JSON-file backed store for model bundles."""

    def __init__(self, data_dir: Path) -> None:
        self._path = data_dir / "bundles.json"
        self._bundles: dict[str, ModelBundle] = {}

    def load(self) -> None:
        if self._path.is_file():
            try:
                raw = json.loads(self._path.read_text())
                self._bundles = {b["id"]: ModelBundle(**b) for b in raw}
                logger.info("Loaded %d bundles from %s", len(self._bundles), self._path)
                return
            except Exception as e:
                logger.warning("Failed to read bundles file, resetting defaults: %s", e)

        self._seed_defaults()

    def _seed_defaults(self) -> None:
        self._bundles = {b.id: b for b in DEFAULT_BUNDLES}
        self._save()
        logger.info("Seeded %d default bundles to %s", len(self._bundles), self._path)

    def _save(self) -> None:
        self._path.parent.mkdir(parents=True, exist_ok=True)
        data = [asdict(b) for b in self._bundles.values()]
        self._path.write_text(json.dumps(data, indent=2))

    def get_all(self) -> list[dict]:
        return [asdict(b) for b in self._bundles.values()]

    def get_by_id(self, bundle_id: str) -> dict | None:
        b = self._bundles.get(bundle_id)
        return asdict(b) if b else None

    def get_by_type(self, transformer_type: str) -> list[dict]:
        return [asdict(b) for b in self._bundles.values() if b.transformer_type == transformer_type]

    def add(self, data: dict) -> dict:
        bundle_id = data.get("id")
        if not bundle_id:
            raise ValueError("Bundle must have an 'id'")
        if bundle_id in self._bundles:
            raise ValueError(f"Bundle '{bundle_id}' already exists")

        valid_fields = {f.name for f in fields(ModelBundle)}
        filtered = {k: v for k, v in data.items() if k in valid_fields}
        filtered["is_default"] = False
        bundle = ModelBundle(**filtered)
        self._bundles[bundle.id] = bundle
        self._save()
        return asdict(bundle)

    def update(self, bundle_id: str, updates: dict) -> dict:
        bundle = self._bundles.get(bundle_id)
        if not bundle:
            raise ValueError(f"Bundle '{bundle_id}' not found")

        valid_fields = {f.name for f in fields(ModelBundle)}
        current = asdict(bundle)
        for k, v in updates.items():
            if k in valid_fields and k != "id":
                current[k] = v

        self._bundles[bundle_id] = ModelBundle(**current)
        self._save()
        return asdict(self._bundles[bundle_id])

    def delete(self, bundle_id: str) -> None:
        if bundle_id not in self._bundles:
            raise ValueError(f"Bundle '{bundle_id}' not found")
        del self._bundles[bundle_id]
        self._save()

    def reset_defaults(self) -> list[dict]:
        self._seed_defaults()
        return self.get_all()
