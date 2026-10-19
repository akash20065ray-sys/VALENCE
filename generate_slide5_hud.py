import os
from PIL import Image, ImageDraw, ImageFont

def get_font(name, size):
    path = os.path.join("C:/Windows/Fonts", name)
    try:
        return ImageFont.truetype(path, size)
    except:
        return ImageFont.load_default()

def generate_slide5_hud_graphic(output_path="hackathon_slides/ui_hud_slide5.png"):
    w, h = 1200, 1380
    img = Image.new("RGBA", (w, h), (8, 20, 14, 255))
    draw = ImageDraw.Draw(img)

    # Fonts
    f_title     = get_font("segoeuib.ttf", 32)
    f_subtitle  = get_font("segoeui.ttf", 22)
    f_stat_val  = get_font("segoeuib.ttf", 46)
    f_stat_lbl  = get_font("segoeuib.ttf", 20)
    f_core_name = get_font("consolab.ttf", 22)
    f_core_sub  = get_font("segoeui.ttf", 18)
    f_cli_font  = get_font("consolab.ttf", 22)

    # Colors
    NEON_GREEN    = (34, 197, 94, 255)
    TECH_CYAN     = (6, 182, 212, 255)
    AMBER         = (245, 158, 11, 255)
    PURPLE        = (168, 85, 247, 255)
    TEXT_WHITE    = (248, 250, 252, 255)
    TEXT_MUTED    = (148, 163, 184, 255)
    
    BG_CARD       = (13, 30, 22, 255)
    BG_HEADER     = (16, 40, 29, 255)
    BG_INNER      = (9, 22, 16, 255)
    BG_CORE_TILE  = (20, 48, 35, 255)
    
    BORDER_GREEN  = (34, 197, 94, 255)
    BORDER_CYAN   = (6, 182, 212, 255)
    BORDER_AMBER  = (245, 158, 11, 255)
    BORDER_PURPLE = (168, 85, 247, 255)
    BORDER_MUTED  = (24, 58, 42, 255)

    # Outer Frame
    draw.rounded_rectangle([(10, 10), (w - 10, h - 10)], radius=18, fill=BG_CARD, outline=BORDER_GREEN, width=3)

    # Top Window Title Bar
    draw.rounded_rectangle([(12, 12), (w - 12, 88)], radius=16, fill=BG_HEADER)
    draw.line([(12, 88), (w - 12, 88)], fill=BORDER_GREEN, width=2)
    
    # Window traffic lights
    draw.ellipse([(32, 40), (48, 56)], fill=(239, 68, 68, 255))
    draw.ellipse([(60, 40), (76, 56)], fill=(245, 158, 11, 255))
    draw.ellipse([(88, 40), (104, 56)], fill=(34, 197, 94, 255))

    draw.text((130, 32), "VALENCE STUDIO  •  LIVE TELEMETRY HUD", font=f_title, fill=NEON_GREEN)

    # Status Pill (Right side of header)
    draw.rounded_rectangle([(w - 280, 28), (w - 32, 72)], radius=10, fill=BG_INNER, outline=BORDER_CYAN, width=1)
    draw.ellipse([(w - 264, 44), (w - 250, 58)], fill=NEON_GREEN)
    draw.text((w - 240, 36), "127.0.0.1:6379 ONLINE", font=f_subtitle, fill=TECH_CYAN)

    # =========================================================================
    # SECTION 1: 4 PINNED HARDWARE CORES
    # =========================================================================
    curr_y = 115
    draw.text((36, curr_y), "THREAD-PER-CORE SHARDS (0 MUTEX CONTENTION)", font=f_stat_lbl, fill=TEXT_MUTED)
    
    curr_y += 38
    tile_w = (w - 72 - 36) // 4
    tile_h = 135
    for i in range(4):
        tx = 36 + i * (tile_w + 12)
        draw.rounded_rectangle([(tx, curr_y), (tx + tile_w, curr_y + tile_h)], radius=12, fill=BG_CORE_TILE, outline=BORDER_GREEN, width=2)
        draw.text((tx + 20, curr_y + 16), f"CORE #{i}", font=f_core_name, fill=NEON_GREEN)
        draw.text((tx + 20, curr_y + 48), "PINNED (CPU_SET)", font=f_core_sub, fill=TEXT_MUTED)
        
        # Load progress bar
        bar_bg_y = curr_y + 82
        draw.rounded_rectangle([(tx + 20, bar_bg_y), (tx + tile_w - 20, bar_bg_y + 12)], radius=6, fill=BG_INNER)
        draw.rounded_rectangle([(tx + 20, bar_bg_y), (tx + tile_w - 35, bar_bg_y + 12)], radius=6, fill=NEON_GREEN)
        draw.text((tx + 20, curr_y + 102), "SPSC Ring: Active", font=f_core_sub, fill=TECH_CYAN)

    # =========================================================================
    # SECTION 2: STORAGE TIERING SPLIT (DRAM vs NVMe)
    # =========================================================================
    curr_y += tile_h + 38
    draw.text((36, curr_y), "AUTOMATED STORAGE TIERING (2-BIT CLOCK)", font=f_stat_lbl, fill=TEXT_MUTED)

    curr_y += 38
    # 2 Stat Cards side-by-side
    card_w = (w - 72 - 16) // 2
    card_h = 185
    
    # Card A: Hot DRAM
    draw.rounded_rectangle([(36, curr_y), (36 + card_w, curr_y + card_h)], radius=12, fill=BG_INNER, outline=BORDER_GREEN, width=2)
    draw.text((58, curr_y + 22), "HOT DRAM TIER (RAM)", font=f_stat_lbl, fill=NEON_GREEN)
    draw.text((58, curr_y + 58), "21.6% Active", font=f_stat_val, fill=TEXT_WHITE)
    draw.text((58, curr_y + 128), "Sub-microsecond latency (< 0.2 µs)", font=f_subtitle, fill=TEXT_MUTED)

    # Card B: Cold NVMe (78.4% SAVED)
    cx2 = 36 + card_w + 16
    draw.rounded_rectangle([(cx2, curr_y), (cx2 + card_w, curr_y + card_h)], radius=12, fill=BG_INNER, outline=BORDER_AMBER, width=2)
    draw.text((cx2 + 22, curr_y + 22), "COLD NVMe FLASH (SSD)", font=f_stat_lbl, fill=AMBER)
    draw.text((cx2 + 22, curr_y + 58), "78.4% RAM SAVED", font=f_stat_val, fill=AMBER)
    draw.text((cx2 + 22, curr_y + 128), "Dormant keys stored at 1/10th cost", font=f_subtitle, fill=TEXT_MUTED)

    # Visual Tiering Progress Bar
    curr_y += card_h + 24
    draw.rounded_rectangle([(36, curr_y), (w - 36, curr_y + 28)], radius=14, fill=BG_INNER, outline=BORDER_MUTED, width=1)
    split_px = int(36 + (w - 72) * 0.216)
    draw.rounded_rectangle([(36, curr_y), (split_px, curr_y + 28)], radius=14, fill=NEON_GREEN)
    draw.rounded_rectangle([(split_px, curr_y), (w - 36, curr_y + 28)], radius=14, fill=AMBER)

    # =========================================================================
    # SECTION 3: KEY PERFORMANCE STATS
    # =========================================================================
    curr_y += 50
    draw.text((36, curr_y), "ENGINE PERFORMANCE METRICS (MEASURED TODAY)", font=f_stat_lbl, fill=TEXT_MUTED)

    curr_y += 38
    stat_box_w = (w - 72 - 32) // 3
    stat_box_h = 160

    stats = [
        ("THROUGHPUT", "1.24M+", "ops / sec", TECH_CYAN, BORDER_CYAN),
        ("p99 LATENCY", "< 4.2 µs", "Deterministic", NEON_GREEN, BORDER_GREEN),
        ("MUTEX LOCKS", "0 LOCKS", "Zero Contention", PURPLE, BORDER_PURPLE),
    ]

    for i, (title, val, sub, col, border) in enumerate(stats):
        bx = 36 + i * (stat_box_w + 16)
        draw.rounded_rectangle([(bx, curr_y), (bx + stat_box_w, curr_y + stat_box_h)], radius=12, fill=BG_INNER, outline=border, width=2)
        draw.text((bx + 22, curr_y + 20), title, font=f_stat_lbl, fill=col)
        draw.text((bx + 22, curr_y + 52), val, font=f_stat_val, fill=TEXT_WHITE)
        draw.text((bx + 22, curr_y + 114), sub, font=f_subtitle, fill=TEXT_MUTED)

    # =========================================================================
    # SECTION 4: INTERACTIVE RESP3 TERMINAL (LIVE WORKBENCH)
    # =========================================================================
    curr_y += stat_box_h + 38
    draw.text((36, curr_y), "IN-BROWSER RESP3 WORKBENCH (ZERO SETUP)", font=f_stat_lbl, fill=TEXT_MUTED)

    curr_y += 38
    term_h = 240
    draw.rounded_rectangle([(36, curr_y), (w - 36, curr_y + term_h)], radius=14, fill=(5, 14, 10, 255), outline=BORDER_CYAN, width=2)
    
    # Terminal text
    draw.text((60, curr_y + 24), "valence> SET user:session:402 \"auth_ok\" EX 86400", font=f_cli_font, fill=TEXT_WHITE)
    draw.text((60, curr_y + 64), "+OK  [Route -> Core #2 | DRAM Slab Alloc: 0.18 µs]", font=f_cli_font, fill=NEON_GREEN)
    
    draw.text((60, curr_y + 116), "valence> GET user:session:402", font=f_cli_font, fill=TEXT_WHITE)
    draw.text((60, curr_y + 156), "\"auth_ok\"  [Tier: HOT DRAM | Latency: 0.14 µs | Hash: JumpConsistent]", font=f_cli_font, fill=TECH_CYAN)

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] UI HUD graphic generated at: {output_path}")

if __name__ == "__main__":
    generate_slide5_hud_graphic()
