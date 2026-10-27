#!/usr/bin/env bash
# Builds src/ into theme.wasm, next to themeSettings.json, so this folder is the theme as YASAFF
# and Discover read it. Then packs it into dist/double_axis_theme.zip for Settings → Theme → Import.
# The zip is byte-for-byte the same on every run when nothing changed.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$here/dist/double_axis_theme.zip"
mkdir -p "$here/dist"

cargo build --manifest-path "$here/Cargo.toml" --release --target wasm32-unknown-unknown
cp "$here/target/wasm32-unknown-unknown/release/yasaff_double_axis_theme.wasm" "$here/theme.wasm"

python3 - "$here" "$out" <<'PY'
import json
import sys
import zipfile
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
json.loads((root / "themeSettings.json").read_text(encoding="utf-8"))
files = [root / "themeSettings.json", root / "theme.wasm"]
files += sorted(p for p in (root / "images").rglob("*") if p.is_file() and p.name != ".gitkeep")
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for path in files:
        info = zipfile.ZipInfo(path.relative_to(root).as_posix(), (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o644 << 16
        z.writestr(info, path.read_bytes())
PY
echo "$out"
