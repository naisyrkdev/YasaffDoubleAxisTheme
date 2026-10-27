#!/usr/bin/env bash
# Packs theme/ into dist/double_axis_theme.zip. The zip is byte-for-byte the same on every run.
# With --bundle, also copies it to ../YASAFF/initial_content/custom_themes/, which YASAFF
# installs in every new profile.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$here/dist/double_axis_theme.zip"
mkdir -p "$here/dist"

python3 - "$here/theme" "$out" <<'PY'
import json
import sys
import zipfile
from pathlib import Path

src, out = Path(sys.argv[1]), Path(sys.argv[2])
json.loads((src / "themeSettings.json").read_text(encoding="utf-8"))
files = sorted(p for p in src.rglob("*") if p.is_file() and p.name != ".gitkeep")
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for path in files:
        info = zipfile.ZipInfo(path.relative_to(src).as_posix(), (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o644 << 16
        z.writestr(info, path.read_bytes())
PY
echo "$out"

if [[ "${1:-}" == "--bundle" ]]; then
    dest="$here/../YASAFF/initial_content/custom_themes"
    mkdir -p "$dest"
    cp "$out" "$dest/"
    echo "$dest/double_axis_theme.zip"
fi
