#!/usr/bin/env python3
"""Generates the Swarf icon: a helical steel chip in temper colours on a face-milled tile.

Writes assets/icon.svg, assets/icon-<size>.png, assets/icon.icns (macOS) and assets/icon.ico.
Needs rsvg-convert (librsvg) and Pillow; iconutil is used on macOS only.

    python3 scripts/make_icon.py
"""

import math
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"

# Temper colours of steel, from straw to blue, along the chip.
TEMPER = [(0.00, (236, 205, 120)), (0.25, (214, 150, 70)), (0.48, (168, 92, 120)), (0.70, (96, 92, 200)), (1.00, (120, 180, 235))]


def lerp_color(u):
    for (u0, c0), (u1, c1) in zip(TEMPER, TEMPER[1:]):
        if u <= u1:
            k = (u - u0) / (u1 - u0)
            return tuple(round(a + (b - a) * k) for a, b in zip(c0, c1))
    return TEMPER[-1][1]


def shade(c, f):
    return tuple(max(0, min(255, round(v * f))) for v in c)


def hexc(c):
    return "#%02x%02x%02x" % c


def chip_segments(a, b, r0, r1, turns, w0, w1, tilt=0.62, steps=600):
    """Helix from a to b, radius r0→r1, projected with a tilt; returns (back, front) SVG paths."""
    ax, ay = b[0] - a[0], b[1] - a[1]
    length = math.hypot(ax, ay)
    dx, dy = ax / length, ay / length  # axis direction
    nx, ny = -dy, dx  # perpendicular on screen
    pts = []
    for i in range(steps + 1):
        u = i / steps
        t = u * turns * 2 * math.pi
        r = r0 + (r1 - r0) * u
        cx, cy = a[0] + ax * u, a[1] + ay * u
        z = math.cos(t)
        sx = cx + nx * r * math.sin(t) + dx * r * tilt * z
        sy = cy + ny * r * math.sin(t) + dy * r * tilt * z
        width = (w0 + (w1 - w0) * u) * (0.55 + 0.45 * abs(math.cos(t + 0.6)))
        pts.append((sx, sy, z, u, width))
    # One short segment per sample keeps the colour continuous; each segment is
    # stretched slightly into its neighbours so anti-aliased seams don't show.
    back, front, shine = [], [], []
    run = []
    for p, q in zip(pts, pts[1:]):
        z = (p[2] + q[2]) / 2
        u = (p[3] + q[3]) / 2
        w = (p[4] + q[4]) / 2
        ddx, ddy = q[0] - p[0], q[1] - p[1]
        n = math.hypot(ddx, ddy) or 1.0
        ox, oy = ddx / n * 1.5, ddy / n * 1.5
        line = f"M{p[0] - ox:.1f},{p[1] - oy:.1f} L{q[0] + ox:.1f},{q[1] + oy:.1f}"
        base = lerp_color(u)
        if z < 0:
            back.append(f'<path d="{line}" stroke="{hexc(shade(base, 0.62))}" stroke-width="{w:.1f}"/>')
        else:
            front.append(f'<path d="{line}" stroke="{hexc(shade(base, 0.85 + 0.3 * z))}" stroke-width="{w:.1f}"/>')
        # Specular streak along the front-most part of each turn.
        if z > 0.45:
            run.append(p)
        elif run:
            shine.append(run)
            run = []
    if run:
        shine.append(run)
    for r in shine:
        if len(r) > 3:
            d = "M" + " L".join(f"{p[0]:.1f},{p[1]:.1f}" for p in r)
            front.append(f'<path d="{d}" stroke="#ffffff" stroke-opacity="0.26" stroke-width="{r[len(r) // 2][4] * 0.2:.1f}" stroke-linecap="round"/>')
    return back, front


# Look of the icon; change these and rerun the script.
SHANK = "#aab3bb"  # tool shank
BODY = "#8b949c"  # fluted section
GROOVE = "#1b2026"  # flute grooves
TILE = ("#3b444d", "#12171c")  # background gradient, top-left to bottom-right
MARKS = "#ffffff"  # face-milling marks on the tile
CHIP_END = (800.0, 470.0)  # where the chip curls to
CHIP_SIZE = (50.0, 112.0)  # coil radius at the tool and at the far end
CHIP_TURNS = 2.4
CHIP_WIDTH = (28.0, 42.0)

TOOL_ANGLE = 0.0  # degrees; 0 = vertical, negative leans the shank to the upper left
TIP = (400.0, 790.0)  # screen position of the tool tip centre
R = 112.0  # tool radius
FLUTE = 430.0  # flute length


def to_screen(x, y):
    """Tool coordinates (x across, y up from the tip, negative) to screen."""
    a = math.radians(TOOL_ANGLE)
    return (TIP[0] + x * math.cos(a) - y * math.sin(a), TIP[1] + x * math.sin(a) + y * math.cos(a))


def end_mill():
    """A carbide end mill drawn upright in tool coordinates; the caller rotates it."""
    slope = 2 * R * math.tan(math.radians(32))  # 32° helix rise across the diameter
    bands = []
    for k in range(-1, 9):
        y0 = -FLUTE + 40 + k * 84
        pts = [(-R, y0), (R, y0 - slope), (R, y0 - slope + 34), (-R, y0 + 34)]
        poly = " ".join(f"{x:.1f},{y:.1f}" for x, y in pts)
        bands.append(f'<polygon points="{poly}" fill="{GROOVE}"/>')
        bands.append(f'<line x1="{-R}" y1="{y0:.1f}" x2="{R}" y2="{y0 - slope:.1f}" stroke="#f2f5f8" stroke-opacity="0.55" stroke-width="3"/>')
    neck = f"{-R},{-FLUTE} {R},{-FLUTE} {R - 8},{-FLUTE - 18} {-R + 8},{-FLUTE - 18}"
    return f'''
    <clipPath id="flutes"><rect x="{-R}" y="{-FLUTE}" width="{2 * R}" height="{FLUTE}" rx="6"/></clipPath>
    <rect x="{-R}" y="-1200" width="{2 * R}" height="{1200 - FLUTE - 16}" fill="{SHANK}"/>
    <polygon points="{neck}" fill="{BODY}"/>
    <rect x="{-R}" y="{-FLUTE}" width="{2 * R}" height="{FLUTE}" rx="6" fill="{BODY}"/>
    <g clip-path="url(#flutes)">{"".join(bands)}</g>
    <rect x="{-R}" y="-1200" width="{2 * R}" height="1200" rx="6" fill="url(#cyl)"/>
    <rect x="{-R}" y="-6" width="{2 * R}" height="6" rx="3" fill="#e6ebef" fill-opacity="0.7"/>
'''


def build_svg():
    edge = to_screen(R - 6, -40)
    back, front = chip_segments(edge, CHIP_END, CHIP_SIZE[0], CHIP_SIZE[1], CHIP_TURNS, CHIP_WIDTH[0], CHIP_WIDTH[1], tilt=1.0)
    marks = "\n".join(
        f'<circle cx="-260" cy="1320" r="{r}" fill="none" stroke="{MARKS}" stroke-opacity="{0.025 + 0.02 * (i % 2):.3f}" stroke-width="7"/>'
        for i, r in enumerate(range(620, 1700, 26))
    )
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs>
    <linearGradient id="tile" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="{TILE[0]}"/>
      <stop offset="1" stop-color="{TILE[1]}"/>
    </linearGradient>
    <clipPath id="clip"><rect x="100" y="100" width="824" height="824" rx="186"/></clipPath>
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="14"/>
      <feOffset dy="12"/>
      <feComponentTransfer><feFuncA type="linear" slope="0.45"/></feComponentTransfer>
      <feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <linearGradient id="cyl" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="#000" stop-opacity="0.55"/>
      <stop offset="0.28" stop-color="#fff" stop-opacity="0.38"/>
      <stop offset="0.45" stop-color="#fff" stop-opacity="0.05"/>
      <stop offset="0.85" stop-color="#000" stop-opacity="0.35"/>
      <stop offset="1" stop-color="#000" stop-opacity="0.6"/>
    </linearGradient>
    <filter id="chipshadow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="10"/>
      <feOffset dx="6" dy="16"/>
      <feComponentTransfer><feFuncA type="linear" slope="0.55"/></feComponentTransfer>
      <feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <g filter="url(#shadow)">
    <rect x="100" y="100" width="824" height="824" rx="186" fill="url(#tile)"/>
  </g>
  <g clip-path="url(#clip)">
    {marks}
    <rect x="100" y="100" width="824" height="824" rx="186" fill="none" stroke="#ffffff" stroke-opacity="0.10" stroke-width="3"/>
  </g>
  <g clip-path="url(#clip)">
    <g filter="url(#chipshadow)" transform="translate({TIP[0]},{TIP[1]}) rotate({TOOL_ANGLE})">{end_mill()}</g>
  </g>
  <g filter="url(#chipshadow)" fill="none" stroke-linejoin="round">
    {chr(10).join(back)}
    {chr(10).join(front)}
  </g>
</svg>
'''


def main():
    ASSETS.mkdir(exist_ok=True)
    svg = ASSETS / "icon.svg"
    svg.write_text(build_svg())
    if not shutil.which("rsvg-convert"):
        sys.exit("rsvg-convert not found (install librsvg)")
    sizes = [16, 32, 48, 64, 128, 256, 512, 1024]
    for s in sizes:
        subprocess.run(["rsvg-convert", "-w", str(s), "-h", str(s), str(svg), "-o", str(ASSETS / f"icon-{s}.png")], check=True)

    from PIL import Image

    Image.open(ASSETS / "icon-256.png").save(ASSETS / "icon.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])

    if shutil.which("iconutil"):
        iconset = ASSETS / "icon.iconset"
        iconset.mkdir(exist_ok=True)
        for s in (16, 32, 128, 256, 512):
            shutil.copy(ASSETS / f"icon-{s}.png", iconset / f"icon_{s}x{s}.png")
            shutil.copy(ASSETS / f"icon-{s * 2}.png", iconset / f"icon_{s}x{s}@2x.png")
        subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(ASSETS / "icon.icns")], check=True)
        shutil.rmtree(iconset)
    print("icons written to", ASSETS)


if __name__ == "__main__":
    main()
