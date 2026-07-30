"""生成 SlayMuManager 统一图标（靛蓝渐变 "S" 图标）

从 1024x1024 渲染，然后缩放到所有 Tauri 所需尺寸。
使用 Pillow + numpy 实现对角渐变。
"""

import numpy as np
from PIL import Image, ImageDraw
import os

# --- 颜色定义（靛蓝色板）---
C_300 = np.array([165, 180, 252])   # primary-300 亮色
C_500 = np.array([99, 102, 241])    # primary-color 基调
C_900 = np.array([49, 46, 129])     # primary-900 深色

# 输出目录
OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(OUT_DIR, exist_ok=True)

# 所需尺寸（Tauri 标准）
SIZES_PNG = [32, 64, 128, 256, 512]
SIZES_SPECIAL = {
    "128x128@2x": 256,
    "Square310x310Logo": 310,
    "Square284x284Logo": 284,
    "Square150x150Logo": 150,
    "Square142x142Logo": 142,
    "Square107x107Logo": 107,
    "Square89x89Logo": 89,
    "Square71x71Logo": 71,
    "Square44x44Logo": 44,
    "Square30x30Logo": 30,
    "StoreLogo": 50,
}


def lerp(c1: np.ndarray, c2: np.ndarray, t: float) -> np.ndarray:
    return (c1 + (c2 - c1) * t).astype(np.uint8)


def render_gradient_icon(size: int) -> Image.Image:
    """渲染靛蓝对角渐变圆角矩形 + S 字母"""
    arr = np.zeros((size, size, 3), dtype=np.uint8)
    diag_max = 2 * size

    # --- 对角渐变：左上 (0,0)→t=0, 右下 (size,size)→t=1 ---
    for y in range(size):
        # 统一计算整行每个像素的 t
        xs = np.arange(size, dtype=np.float32)
        t = (xs + y) / diag_max

        # 线性插值三阶渐变
        mask1 = t < 0.5
        mask2 = ~mask1
        t1 = t[mask1] / 0.5
        t2 = (t[mask2] - 0.5) / 0.5

        row = np.zeros((size, 3), dtype=np.uint8)
        if mask1.any():
            row[mask1] = lerp(C_300, C_500, t1[:, np.newaxis])
        if mask2.any():
            row[mask2] = lerp(C_500, C_900, t2[:, np.newaxis])

        arr[y] = row

    img = Image.fromarray(arr, "RGB")

    # --- 圆角蒙版 ---
    radius = int(size * 28 / 128)
    mask = Image.new("L", (size, size), 0)
    md = ImageDraw.Draw(mask)
    md.rounded_rectangle(
        (0, 0, size - 1, size - 1),
        radius=radius,
        fill=255,
    )

    result = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    result.paste(img, (0, 0), mask)

    # --- 绘制 "S" 字母 ---
    draw = ImageDraw.Draw(result)
    # 使用默认字体 + 手动缩放字号
    font_size = max(12, int(size * 58 / 128))
    try:
        from PIL import ImageFont
        font = ImageFont.truetype("segoeui.ttf", font_size)
    except (OSError, IOError):
        try:
            font = ImageFont.truetype("arial.ttf", font_size)
        except (OSError, IOError):
            font = ImageFont.load_default()
    # Fallback: if default font is tiny, draw with scalable approach
    bbox = draw.textbbox((0, 0), "S", font=font)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    x = (size - tw) // 2 - bbox[0]
    y = (size - th) // 2 - bbox[1] - int(size * 0.02)  # 微调垂直居中
    draw.text((x, y), "S", fill="white", font=font)

    return result


def save_png(img: Image.Image, path: str):
    img.save(path, "PNG")


def render_foreground_only(size: int) -> Image.Image:
    """仅渲染白色 S 字母（透明背景），用于 Android adaptive icon 前景"""
    result = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(result)
    font_size = max(12, int(size * 58 / 128))
    try:
        from PIL import ImageFont
        font = ImageFont.truetype("segoeui.ttf", font_size)
    except (OSError, IOError):
        try:
            font = ImageFont.truetype("arial.ttf", font_size)
        except (OSError, IOError):
            font = ImageFont.load_default()
    bbox = draw.textbbox((0, 0), "S", font=font)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    x = (size - tw) // 2 - bbox[0]
    y = (size - th) // 2 - bbox[1] - int(size * 0.02)
    draw.text((x, y), "S", fill="white", font=font)
    return result


def build_ico():
    """从各尺寸 PNG 合成 ICO（手动构造，避免 Pillow 多帧 ICO 保存 bug）"""
    import struct
    import io

    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    images = [render_gradient_icon(s) for s in ico_sizes]

    # 先把每帧保存为 PNG 到内存
    png_data_list = []
    for img in images:
        buf = io.BytesIO()
        img.save(buf, "PNG")
        png_data_list.append(buf.getvalue())

    # ICO 文件头
    count = len(ico_sizes)
    header = struct.pack("<HHH", 0, 1, count)  # reserved=0, type=1(ICO), count

    # 目录项（16 bytes each）
    offset = 6 + count * 16
    directories = b""
    for i, (s, png_data) in enumerate(zip(ico_sizes, png_data_list)):
        w = s if s < 256 else 0
        h = s if s < 256 else 0
        directories += struct.pack(
            "<BBBBHHII",
            w, h,           # width, height (0=256)
            0,              # color palette (unused)
            0,              # reserved
            1,              # color planes
            32,             # bits per pixel
            len(png_data),  # size
            offset,         # offset in file
        )
        offset += len(png_data)

    # 写入文件
    ico_path = os.path.join(OUT_DIR, "icon.ico")
    with open(ico_path, "wb") as f:
        f.write(header)
        f.write(directories)
        for png_data in png_data_list:
            f.write(png_data)

    print(f"  ✓ icon.ico ({count} frames, {os.path.getsize(ico_path)} bytes)")


def main():
    print("生成 SlayMuManager 统一图标 ...\n")

    # 1. 标准 PNG 尺寸
    for s in SIZES_PNG:
        img = render_gradient_icon(s)
        name = f"{s}x{s}.png"
        save_png(img, os.path.join(OUT_DIR, name))
        print(f"  ✓ {name}")

    # 2. 特殊命名尺寸
    for name, s in SIZES_SPECIAL.items():
        img = render_gradient_icon(s)
        save_png(img, os.path.join(OUT_DIR, f"{name}.png"))
        print(f"  ✓ {name}.png")

    # 3. icon.png (主图标，512x512)
    img = render_gradient_icon(512)
    save_png(img, os.path.join(OUT_DIR, "icon.png"))
    print("  ✓ icon.png (512x512)")

    # 4. icon.ico
    build_ico()

    # 5. Android 图标
    build_android_icons()

    # 6. iOS 图标
    build_ios_icons()

    # 7. icon.icns (macOS — 尽可能输出)
    print("  (保留 icon.icns — 需 macOS 原生工具构建)")

    print("\n✓ 全部图标生成完成!")


def build_android_icons():
    ANDROID_DIR = os.path.join(OUT_DIR, "android")
    if not os.path.isdir(ANDROID_DIR):
        return
    print("\n--- Android ---")
    # density → 图标尺寸
    densities = {
        "mdpi": 48,
        "hdpi": 72,
        "xhdpi": 96,
        "xxhdpi": 144,
        "xxxhdpi": 192,
    }
    for density, base_sz in densities.items():
        launcher = render_gradient_icon(base_sz)
        launcher_round = render_gradient_icon(base_sz)
        foreground = render_foreground_only(base_sz)
        dir_path = os.path.join(ANDROID_DIR, "mipmap-" + density)
        launcher.save(os.path.join(dir_path, "ic_launcher.png"), "PNG")
        launcher_round.save(os.path.join(dir_path, "ic_launcher_round.png"), "PNG")
        foreground.save(os.path.join(dir_path, "ic_launcher_foreground.png"), "PNG")
        print(f"  ✓ {density}")


def build_ios_icons():
    IOS_DIR = os.path.join(OUT_DIR, "ios")
    if not os.path.isdir(IOS_DIR):
        return
    print("\n--- iOS ---")
    ios_sizes = [
        ("AppIcon-20x20@1x", 20),
        ("AppIcon-20x20@2x", 40),
        ("AppIcon-20x20@3x", 60),
        ("AppIcon-29x29@1x", 29),
        ("AppIcon-29x29@2x", 58),
        ("AppIcon-29x29@3x", 87),
        ("AppIcon-40x40@1x", 40),
        ("AppIcon-40x40@2x", 80),
        ("AppIcon-40x40@3x", 120),
        ("AppIcon-60x60@2x", 120),
        ("AppIcon-60x60@3x", 180),
        ("AppIcon-76x76@1x", 76),
        ("AppIcon-76x76@2x", 152),
        ("AppIcon-83.5x83.5@2x", 167),
        ("AppIcon-512@2x", 1024),
    ]
    # Handle the -1 variant for iTunes
    for name_root, sz in ios_sizes:
        img = render_gradient_icon(sz)
        path = os.path.join(IOS_DIR, f"{name_root}.png")
        img.save(path, "PNG")
    # The -1 variants (same size, different name for iTunes)
    for variant_suffix in ["20x20@2x-1", "29x29@2x-1", "40x40@2x-1"]:
        src = f"AppIcon-{variant_suffix.replace('-1', '')}"
        dst = f"AppIcon-{variant_suffix}"
        sz_match = None
        for name_root, sz in ios_sizes:
            if name_root == src:
                sz_match = sz
                break
        if sz_match:
            img = render_gradient_icon(sz_match)
            img.save(os.path.join(IOS_DIR, f"{dst}.png"), "PNG")
    print(f"  ✓ {len(ios_sizes)} sizes")


if __name__ == "__main__":
    main()
