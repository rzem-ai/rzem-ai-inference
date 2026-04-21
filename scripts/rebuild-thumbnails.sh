#!/usr/bin/env bash
# Rebuild freedesktop thumbnails for every image in a directory so
# Nautilus / GNOME Files shows previews without invoking its own
# thumbnailer. Writes to ~/.cache/thumbnails/large/{md5(uri)}.png with
# the required Thumb::URI and Thumb::MTime tEXt chunks, and clears any
# stale fail entry.
#
# Usage: rebuild-thumbnails.sh [DIR]
#   DIR defaults to "$HOME/.config/RZEM AI Inference/output"

set -euo pipefail

DIR="${1:-$HOME/.config/Inference/output}"

if [[ ! -d "$DIR" ]]; then
	echo "error: not a directory: $DIR" >&2
	exit 1
fi

for bin in python3 md5sum; do
	command -v "$bin" >/dev/null || { echo "error: $bin not found" >&2; exit 1; }
done

python3 - "$DIR" <<'PY'
import hashlib
import os
import sys
import urllib.parse
from pathlib import Path

try:
	from PIL import Image, PngImagePlugin
except ImportError:
	sys.exit("error: python3-pil not installed (sudo apt install python3-pil)")

SIZE = 256
CACHE_DIR = Path(os.environ.get("XDG_CACHE_HOME") or Path.home() / ".cache") / "thumbnails"
LARGE_DIR = CACHE_DIR / "large"
FAIL_DIR = CACHE_DIR / "fail" / "gnome-thumbnail-factory"
LARGE_DIR.mkdir(parents=True, exist_ok=True)

src_dir = Path(sys.argv[1])
exts = {".png", ".jpg", ".jpeg", ".webp"}

made = skipped = failed = 0
for path in sorted(src_dir.iterdir()):
	if path.suffix.lower() not in exts or not path.is_file():
		continue
	uri = "file://" + urllib.parse.quote(str(path.resolve()))
	digest = hashlib.md5(uri.encode()).hexdigest()
	out = LARGE_DIR / f"{digest}.png"
	mtime = int(path.stat().st_mtime)

	if out.exists() and out.stat().st_mtime >= path.stat().st_mtime:
		skipped += 1
		continue

	try:
		with Image.open(path) as im:
			im.thumbnail((SIZE, SIZE))
			meta = PngImagePlugin.PngInfo()
			meta.add_text("Thumb::URI", uri)
			meta.add_text("Thumb::MTime", str(mtime))
			tmp = out.with_suffix(".png.tmp")
			im.save(tmp, "PNG", pnginfo=meta, optimize=True)
			os.chmod(tmp, 0o600)
			os.replace(tmp, out)
		(FAIL_DIR / f"{digest}.png").unlink(missing_ok=True)
		made += 1
	except Exception as e:
		print(f"  ! {path.name}: {e}", file=sys.stderr)
		failed += 1

print(f"done: {made} written, {skipped} already current, {failed} failed")
PY
