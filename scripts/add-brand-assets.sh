#!/usr/bin/env bash
# add-brand-assets.sh — writes the Aspen brand SVGs into resources/brand/
# Run from the repository root: bash scripts/add-brand-assets.sh
# Safe to re-run: overwrites only the files it owns.

set -euo pipefail

OUT="resources/brand"
mkdir -p "$OUT"

# ---------------------------------------------------------------- primary mark
# Two blades: the pine blade, and its silver echo offset up-left. One leaf, two
# moments. Curved off-axis petioles stand for the flattened aspen petiole.
cat > "$OUT/aspen-icon.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="120" height="120" role="img" aria-label="Aspen">
  <title>Aspen</title>
  <g>
    <ellipse cx="44" cy="44" rx="17" ry="14" transform="rotate(-12 44 44)" fill="#B3BCC2"/>
    <path d="M44 58 Q43 70 36 80" fill="none" stroke="#B3BCC2" stroke-width="4" stroke-linecap="round"/>
    <ellipse cx="74" cy="40" rx="20" ry="16" transform="rotate(-26 74 40)" fill="#17453A"/>
    <path d="M74 56 Q74 70 66 84" fill="none" stroke="#17453A" stroke-width="5" stroke-linecap="round"/>
  </g>
</svg>
SVG

# ------------------------------------------------------------- single colour cut
# One ink, for stamps, embroidery and mono print.
sed \
  -e 's/#B3BCC2/#17453A/g' \
  -e 's/#17453A/#17453A/g' \
  "$OUT/aspen-icon.svg" > "$OUT/aspen-icon-mono.svg"

# --------------------------------------------------------------- small-size cut
# Below 24 px two blades clog, so the favicon is a single blade.
cat > "$OUT/aspen-icon-small.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="120" height="120" role="img" aria-label="Aspen">
  <title>Aspen</title>
  <g>
    <ellipse cx="62" cy="44" rx="24" ry="19" transform="rotate(-20 62 44)" fill="#17453A"/>
    <path d="M62 63 Q62 80 54 96" fill="none" stroke="#17453A" stroke-width="7" stroke-linecap="round"/>
  </g>
</svg>
SVG

# ------------------------------------------------------------- social preview
# 1280 x 640 GitHub social preview. Export to PNG before uploading: GitHub does
# not accept SVG for social previews.
cat > "$OUT/aspen-social.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 640" width="1280" height="640" role="img" aria-label="Aspen — GitHub Actions impacts estimator">
  <title>Aspen — GitHub Actions impacts estimator</title>
  <rect width="1280" height="640" fill="#F6F4EF"/>
  <g transform="translate(168 232) scale(1.5)">
    <ellipse cx="44" cy="44" rx="17" ry="14" transform="rotate(-12 44 44)" fill="#B3BCC2"/>
    <path d="M44 58 Q43 70 36 80" fill="none" stroke="#B3BCC2" stroke-width="4" stroke-linecap="round"/>
    <ellipse cx="74" cy="40" rx="20" ry="16" transform="rotate(-26 74 40)" fill="#17453A"/>
    <path d="M74 56 Q74 70 66 84" fill="none" stroke="#17453A" stroke-width="5" stroke-linecap="round"/>
  </g>
  <text x="430" y="352" fill="#17453A"
        font-family="Inter, 'IBM Plex Sans', 'Helvetica Neue', Arial, sans-serif"
        font-size="132" font-weight="500" letter-spacing="-4">aspen</text>
  <text x="434" y="420" fill="#4A5560"
        font-family="Inter, 'IBM Plex Sans', 'Helvetica Neue', Arial, sans-serif"
        font-size="40" font-weight="400" letter-spacing="0">GitHub Actions impacts estimator</text>
</svg>
SVG

echo "Written to $OUT:"
ls -1 "$OUT"
