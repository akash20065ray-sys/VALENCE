import os
from PIL import Image, ImageDraw, ImageFont

def get_font(name, size):
    path = os.path.join("C:/Windows/Fonts", name)
    try:
        return ImageFont.truetype(path, size)
    except:
        return ImageFont.load_default()

def draw_arrow_h(draw, x1, y, x2, color, width=5, arrow_size=20):
    draw.line([(x1, y), (x2, y)], fill=color, width=width)
    draw.polygon([(x2, y), (x2 - arrow_size, y - arrow_size // 2), (x2 - arrow_size, y + arrow_size // 2)], fill=color)

def draw_arrow_v(draw, x, y1, y2, color, width=5, arrow_size=20):
    draw.line([(x, y1), (x, y2)], fill=color, width=width)
    draw.polygon([(x, y2), (x - arrow_size // 2, y2 - arrow_size), (x + arrow_size // 2, y2 - arrow_size)], fill=color)

def draw_arrow_bidirectional_v(draw, x, y1, y2, color, width=5, arrow_size=20):
    draw.line([(x, y1), (x, y2)], fill=color, width=width)
    draw.polygon([(x, y1), (x - arrow_size // 2, y1 + arrow_size), (x + arrow_size // 2, y1 + arrow_size)], fill=color)
    draw.polygon([(x, y2), (x - arrow_size // 2, y2 - arrow_size), (x + arrow_size // 2, y2 - arrow_size)], fill=color)

def draw_pill(draw, x, y, text, font, text_color, border_color, bg_color):
    bbox = font.getbbox(text)
    pw = bbox[2] - bbox[0] + 36
    ph = bbox[3] - bbox[1] + 24
    draw.rounded_rectangle([(x - pw // 2, y - ph // 2), (x + pw // 2, y + ph // 2)], radius=10, fill=bg_color, outline=border_color, width=2)
    draw.text((x - (bbox[2] - bbox[0]) // 2, y - (bbox[3] - bbox[1]) // 2 - 2), text, font=font, fill=text_color)

def generate_compact_architecture_graphic(output_path="hackathon_slides/arch_diagram_slide4.png"):
    w, h = 2600, 1500
    
    # Ultra-deep dark background for maximum readability
    img = Image.new("RGBA", (w, h), (6, 16, 12, 255))
    draw = ImageDraw.Draw(img)

    # High-Readability Large Typography
    f_main_title = get_font("segoeuib.ttf", 44)
    f_col_hdr    = get_font("segoeuib.ttf", 32)
    f_box_title  = get_font("segoeuib.ttf", 34)
    f_box_desc   = get_font("segoeuib.ttf", 26)
    f_chip       = get_font("segoeuib.ttf", 26)
    f_pill       = get_font("segoeuib.ttf", 24)

    NEON_GREEN    = (34, 197, 94, 255)
    TECH_CYAN     = (6, 182, 212, 255)
    AMBER         = (245, 158, 11, 255)
    PURPLE        = (168, 85, 247, 255)
    TEXT_WHITE    = (255, 255, 255, 255)
    TEXT_MUTED    = (160, 174, 192, 255)

    BG_CARD       = (11, 26, 19, 255)
    BG_HEADER     = (14, 38, 27, 255)
    BG_PILL       = (8, 20, 15, 255)
    BG_CHIP       = (18, 48, 34, 255)

    BORDER_GREEN  = (34, 197, 94, 255)
    BORDER_CYAN   = (6, 182, 212, 255)
    BORDER_AMBER  = (245, 158, 11, 255)
    BORDER_PURPLE = (168, 85, 247, 255)
    BORDER_MUTED  = (28, 68, 49, 255)

    # Outer Frame
    draw.rounded_rectangle([(14, 14), (w - 14, h - 14)], radius=20, outline=BORDER_GREEN, width=3)

    # Top Title Banner
    draw.rounded_rectangle([(16, 16), (w - 16, 102)], radius=18, fill=BG_HEADER)
    draw.line([(16, 102), (w - 16, 102)], fill=BORDER_GREEN, width=2)
    t_bbox = f_main_title.getbbox("VALENCE SYSTEM ARCHITECTURE")
    draw.text(((w - (t_bbox[2] - t_bbox[0])) // 2, 24), "VALENCE SYSTEM ARCHITECTURE", font=f_main_title, fill=NEON_GREEN)

    # 3 Columns with 260px Spacing
    col_w   = 640
    col_gap = 260
    c1_x = 80
    c2_x = c1_x + col_w + col_gap  # 980
    c3_x = c2_x + col_w + col_gap  # 1880

    hdr_y = 125
    draw.text((c1_x + 10, hdr_y), "1. INGRESS (Network)", font=f_col_hdr, fill=TECH_CYAN)
    draw.text((c2_x + 10, hdr_y), "2. ENGINE (Compute)", font=f_col_hdr, fill=NEON_GREEN)
    draw.text((c3_x + 10, hdr_y), "3. STORAGE & PERSISTENCE", font=f_col_hdr, fill=AMBER)

    draw.line([(c1_x + 10, hdr_y + 48), (c1_x + col_w - 10, hdr_y + 48)], fill=BORDER_CYAN, width=3)
    draw.line([(c2_x + 10, hdr_y + 48), (c2_x + col_w - 10, hdr_y + 48)], fill=BORDER_GREEN, width=3)
    draw.line([(c3_x + 10, hdr_y + 48), (c3_x + col_w - 10, hdr_y + 48)], fill=BORDER_AMBER, width=3)

    box_h = 200
    r1_y = 230
    r2_y = 670
    r3_y = 1110

    def draw_card(x, y, title, desc, border_col, title_col):
        draw.rounded_rectangle([(x, y), (x + col_w, y + box_h)], radius=16, fill=BG_CARD, outline=border_col, width=3)
        draw.text((x + 28, y + 24), title, font=f_box_title, fill=title_col)
        draw.line([(x + 28, y + 80), (x + col_w - 28, y + 80)], fill=BORDER_MUTED, width=2)
        draw.text((x + 28, y + 112), desc, font=f_box_desc, fill=TEXT_WHITE)

    # Row 1 Cards
    draw_card(c1_x, r1_y, "CLIENT REQUESTS", "Python / Go / Node RESP3", BORDER_CYAN, TECH_CYAN)
    draw_card(c2_x, r1_y, "LOCK-FREE ROUTER", "Key-hash dispatch across CPU shards", BORDER_GREEN, NEON_GREEN)
    draw_card(c3_x, r1_y, "HOT DRAM TIER (RAM)", "Active keys (< 1µs latency)", BORDER_GREEN, NEON_GREEN)

    # Row 1 Horizontal Arrows
    a1_y = r1_y + box_h // 2
    draw_arrow_h(draw, c1_x + col_w, a1_y, c2_x, TECH_CYAN, width=5, arrow_size=20)
    draw_arrow_h(draw, c2_x + col_w, a1_y, c3_x, NEON_GREEN, width=5, arrow_size=20)

    # Connectors R1 -> R2
    c1_mid_x = c1_x + col_w // 2
    c2_mid_x = c2_x + col_w // 2
    c3_mid_x = c3_x + col_w // 2

    # Col 1: Zero-copy
    draw_arrow_v(draw, c1_mid_x, r1_y + box_h, r2_y, TECH_CYAN, width=5, arrow_size=20)
    draw_pill(draw, c1_mid_x, (r1_y + box_h + r2_y) // 2, "Zero-copy", f_pill, TECH_CYAN, BORDER_CYAN, BG_PILL)

    # Col 2: Core dispatch
    draw_arrow_v(draw, c2_mid_x, r1_y + box_h, r2_y, NEON_GREEN, width=5, arrow_size=20)
    draw_pill(draw, c2_mid_x, (r1_y + box_h + r2_y) // 2, "Core dispatch", f_pill, NEON_GREEN, BORDER_GREEN, BG_PILL)

    # Col 3: 2-Bit CLOCK Auto-Tiering
    draw_arrow_bidirectional_v(draw, c3_mid_x, r1_y + box_h, r2_y, AMBER, width=5, arrow_size=20)
    draw_pill(draw, c3_mid_x, (r1_y + box_h + r2_y) // 2, "2-Bit CLOCK Auto-Tiering", f_pill, AMBER, BORDER_AMBER, BG_PILL)

    # Row 2 Cards
    draw_card(c1_x, r2_y, "AVX2 SIMD PARSER", "289M ops/s frame decode", BORDER_CYAN, TECH_CYAN)

    # Box 2.2: 4 PINNED CPU CORES
    draw.rounded_rectangle([(c2_x, r2_y), (c2_x + col_w, r2_y + box_h)], radius=16, fill=BG_CARD, outline=BORDER_GREEN, width=3)
    draw.text((c2_x + 28, r2_y + 18), "4 PINNED CPU CORES", font=f_box_title, fill=NEON_GREEN)
    draw.line([(c2_x + 28, r2_y + 72), (c2_x + col_w - 28, r2_y + 72)], fill=BORDER_MUTED, width=2)

    # 4 Core Chips: [C0] [C1] [C2] [C3]
    chip_w = (col_w - 56 - 36) // 4
    chip_h = 46
    chip_y = r2_y + 82
    for i in range(4):
        cx = c2_x + 28 + i * (chip_w + 12)
        draw.rounded_rectangle([(cx, chip_y), (cx + chip_w, chip_y + chip_h)], radius=8, fill=BG_CHIP, outline=BORDER_GREEN, width=2)
        tag = f"[C{i}]"
        t_bb = f_chip.getbbox(tag)
        draw.text((cx + (chip_w - (t_bb[2] - t_bb[0])) // 2, chip_y + 8), tag, font=f_chip, fill=NEON_GREEN)

    draw.text((c2_x + 28, r2_y + 144), "0 Mutex Contention  •  L1/L2 Local", font=f_box_desc, fill=TEXT_WHITE)

    # Row 2 Horizontal Arrow: 4 PINNED CPU CORES -> COLD NVMe TIER
    a2_y = r2_y + box_h // 2
    draw_arrow_h(draw, c2_x + col_w, a2_y, c3_x, AMBER, width=5, arrow_size=20)
    draw_pill(draw, (c2_x + col_w + c3_x) // 2, a2_y - 40, "Cold Extents", f_pill, AMBER, BORDER_AMBER, BG_PILL)

    draw_card(c3_x, r2_y, "COLD NVMe TIER (SSD)", "70%+ RAM saved (16µs NVMe)", BORDER_AMBER, AMBER)

    # Connectors R2 -> R3
    draw_arrow_v(draw, c2_mid_x, r2_y + box_h, r3_y, NEON_GREEN, width=5, arrow_size=20)
    draw_pill(draw, c2_mid_x, (r2_y + box_h + r3_y) // 2, "Non-blocking SPSC Ring", f_pill, NEON_GREEN, BORDER_GREEN, BG_PILL)

    draw_arrow_v(draw, c3_mid_x, r2_y + box_h, r3_y, PURPLE, width=5, arrow_size=20)
    draw_pill(draw, c3_mid_x, (r2_y + box_h + r3_y) // 2, "Async log flush", f_pill, PURPLE, BORDER_PURPLE, BG_PILL)

    # Row 3 Cards
    draw_card(c2_x, r3_y, "LOCK-FREE IPC", "Inter-core message channels", BORDER_GREEN, NEON_GREEN)
    draw_card(c3_x, r3_y, "FORKLESS WAL & BACKUP", "Zero fork() OOM spikes  •  Append-only", BORDER_PURPLE, PURPLE)

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] High-readability architecture graphic generated at: {output_path}")

if __name__ == "__main__":
    generate_compact_architecture_graphic()
