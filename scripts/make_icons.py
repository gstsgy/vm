#!/usr/bin/env python3
"""生成 Tauri 所需图标（纯标准库，零依赖）。

输出:
  vm-gui/src-tauri/icons/icon.png  (512x512)
  vm-gui/src-tauri/icons/icon.ico  (32x32)
"""
import os
import struct
import zlib

OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "vm-gui", "src-tauri", "icons")
W, H = 512, 512

# 主题蓝 #2d6cdf
BG = (0x2D, 0x6C, 0xDF, 0xFF)


def pixels(w, h, color):
    px = bytearray()
    for _ in range(h):
        for _ in range(w):
            px += bytes(color)
    return px


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


def write_ico(path, w, h, px):
    stride = ((w * 4 + 3) // 4) * 4
    xor = bytearray()
    for y in range(h - 1, -1, -1):
        row = px[y * w * 4 : (y + 1) * w * 4]
        for i in range(0, len(row), 4):
            r, g, b, a = row[i], row[i + 1], row[i + 2], row[i + 3]
            xor += bytes((b, g, r, a))
        xor += b"\x00" * (stride - w * 4)
    andmask = b"\x00" * (((w + 31) // 32) * 4 * h)
    imagesize = 40 + len(xor) + len(andmask)
    bmp = struct.pack("<IiiHHIIiiII", 40, w, h * 2, 1, 32, 0, imagesize - 40, 0, 0, 0, 0)
    img = bmp + xor + andmask
    icondir = struct.pack("<HHH", 0, 1, 1)
    entry = struct.pack("<BBBBHHII", w if w < 256 else 0, h if h < 256 else 0, 0, 0, 1, 32, len(img), 6 + 16)
    with open(path, "wb") as f:
        f.write(icondir)
        f.write(entry)
        f.write(img)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    px512 = pixels(512, 512, BG)
    write_png(os.path.join(OUT_DIR, "icon.png"), 512, 512, px512)
    px32 = pixels(32, 32, BG)
    write_ico(os.path.join(OUT_DIR, "icon.ico"), 32, 32, px32)
    print("icons generated at", os.path.abspath(OUT_DIR))


if __name__ == "__main__":
    main()
