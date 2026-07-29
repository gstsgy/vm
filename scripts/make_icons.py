#!/usr/bin/env python3
"""生成 Tauri 所需图标（纯标准库，零依赖）。

画一个圆角蓝底 + 白色 "vm" 字样的图标。

输出:
  vm-gui/src-tauri/icons/icon.png  (512x512)
  vm-gui/src-tauri/icons/icon.ico  (256x256)
"""
import os
import struct
import zlib

OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "vm-gui", "src-tauri", "icons")

# 配色：主题蓝底、白字
BG = (0x2D, 0x6C, 0xDF, 0xFF)
FG = (0xFF, 0xFF, 0xFF, 0xFF)
TRANSPARENT = (0, 0, 0, 0)

# 5x7 点阵字模（'v' 与 'm'）
GLYPHS = {
    "v": [
        "X...X",
        "X...X",
        "X...X",
        "X...X",
        ".X.X.",
        ".X.X.",
        "..X..",
    ],
    "m": [
        ".....",
        ".....",
        "XX.XX",
        "X.X.X",
        "X.X.X",
        "X...X",
        "X...X",
    ],
}


def new_canvas(w, h, color):
    return [list(color) for _ in range(w * h)]


def put(canvas, w, x, y, color):
    if 0 <= x < w and 0 <= y < len(canvas) // w:
        canvas[y * w + x] = list(color)


def fill_rounded(canvas, w, h, color, radius):
    for y in range(h):
        for x in range(w):
            # 圆角判定
            cx = min(x, w - 1 - x)
            cy = min(y, h - 1 - y)
            if cx < radius and cy < radius:
                dx = radius - cx
                dy = radius - cy
                if dx * dx + dy * dy > radius * radius:
                    continue
            canvas[y * w + x] = list(color)


def draw_glyph(canvas, w, glyph, ox, oy, scale, color):
    for gy, row in enumerate(glyph):
        for gx, ch in enumerate(row):
            if ch == "X":
                for sy in range(scale):
                    for sx in range(scale):
                        put(canvas, w, ox + gx * scale + sx, oy + gy * scale + sy, color)


def draw_text(canvas, w, h):
    text = "vm"
    gw, gh = 5, 7
    scale = 40
    gap = 40  # 字间距
    total_w = len(text) * gw * scale + (len(text) - 1) * gap
    total_h = gh * scale
    ox = (w - total_w) // 2
    oy = (h - total_h) // 2
    for i, ch in enumerate(text):
        gx0 = ox + i * (gw * scale + gap)
        draw_glyph(canvas, w, GLYPHS[ch], gx0, oy, scale, FG)


def canvas_to_bytes(canvas):
    out = bytearray()
    for px in canvas:
        out += bytes(px)
    return out


def write_png(path, w, h, px):
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        raw += px[y * w * 4 : (y + 1) * w * 4]
    comp = zlib.compress(bytes(raw), 9)

    def chunk(typ, data):
        return (
            struct.pack(">I", len(data))
            + typ
            + data
            + struct.pack(">I", zlib.crc32(typ + data) & 0xFFFFFFFF)
        )

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", comp))
        f.write(chunk(b"IEND", b""))


def write_ico_png(path, w, h, px):
    """ICO 内嵌 PNG（Vista+ 支持），适合大尺寸。"""
    # 先编码为 PNG 字节
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        raw += px[y * w * 4 : (y + 1) * w * 4]
    comp = zlib.compress(bytes(raw), 9)

    def chunk(typ, data):
        return (
            struct.pack(">I", len(data))
            + typ
            + data
            + struct.pack(">I", zlib.crc32(typ + data) & 0xFFFFFFFF)
        )

    png = bytearray()
    png += b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", comp)
    png += chunk(b"IEND", b"")

    icondir = struct.pack("<HHH", 0, 1, 1)
    entry = struct.pack(
        "<BBBBHHII",
        w if w < 256 else 0,
        h if h < 256 else 0,
        0, 0, 1, 32, len(png), 6 + 16,
    )
    with open(path, "wb") as f:
        f.write(icondir)
        f.write(entry)
        f.write(png)


def scale_down(canvas, w, h, nw, nh):
    """最近邻缩放到 nw x nh。"""
    out = new_canvas(nw, nh, TRANSPARENT)
    for y in range(nh):
        sy = y * h // nh
        for x in range(nw):
            sx = x * w // nw
            out[y * nw + x] = canvas[sy * w + sx]
    return out


def build(w, h):
    canvas = new_canvas(w, h, TRANSPARENT)
    fill_rounded(canvas, w, h, BG, radius=w // 6)
    draw_text(canvas, w, h)
    return canvas


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    W = H = 512
    canvas = build(W, H)
    write_png(os.path.join(OUT_DIR, "icon.png"), W, H, canvas_to_bytes(canvas))

    ico = scale_down(canvas, W, H, 256, 256)
    write_ico_png(os.path.join(OUT_DIR, "icon.ico"), 256, 256, canvas_to_bytes(ico))
    print("icons generated at", os.path.abspath(OUT_DIR))


if __name__ == "__main__":
    main()
