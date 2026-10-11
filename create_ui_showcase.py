import os
from PIL import Image, ImageDraw, ImageFont

def generate_ui_showcase(output_path="hackathon_slides/ui_showcase.png"):
    width, height = 2400, 1400
    img = Image.new("RGBA", (width, height), (7, 16, 11, 255))
    draw = ImageDraw.Draw(img)

    font_path_bold = "C:/Windows/Fonts/segoeuib.ttf"
    font_path_reg = "C:/Windows/Fonts/segoeui.ttf"
    font_path_mono = "C:/Windows/Fonts/consola.ttf"

    def get_font(path, size):
        try:
            return ImageFont.truetype(path, size)
        except:
            return ImageFont.load_default()

    font_title = get_font(font_path_bold, 50)
    font_sub = get_font(font_path_reg, 24)
    font_h1 = get_font(font_path_bold, 26)
    font_h2 = get_font(font_path_bold, 20)
    font_body = get_font(font_path_reg, 18)
    font_mono = get_font(font_path_mono, 18)
    font_mono_sm = get_font(font_path_mono, 16)
    font_badge = get_font(font_path_bold, 15)

    NEON_GREEN   = (34, 197, 94, 255)
    TECH_CYAN    = (6, 182, 212, 255)
    AMBER        = (245, 158, 11, 255)
    BORDER_GREEN = (22, 101, 52, 255)
    BORDER_SUBTLE= (30, 58, 48, 255)
    BG_CARD      = (13, 27, 21, 255)
    BG_WINDOW    = (10, 20, 16, 255)
    TEXT_WHITE   = (248, 250, 252, 255)
    TEXT_MUTED   = (148, 163, 184, 255)
    TEXT_DIM     = (100, 116, 139, 255)

    # Grid & Border
    for x in range(0, width, 60):
        draw.line([(x, 0), (x, height)], fill=(12, 28, 20, 100), width=1)
    for y in range(0, height, 60):
        draw.line([(0, y), (width, y)], fill=(12, 28, 20, 100), width=1)
    draw.rounded_rectangle([(30, 30), (width - 30, height - 30)], radius=20, outline=BORDER_GREEN, width=2)

    # Header
    draw.text((70, 55), "VALENCE STUDIO: OPERATIONAL CONSOLE & TELEMETRY", font=font_title, fill=NEON_GREEN)
    draw.text((72, 120), "Industrial Management Interface Inspired by RedisInsight, AWS MemoryDB & Datadog APM", font=font_sub, fill=TEXT_MUTED)

    # Window Chrome Helper (Titlebar + 3 traffic light dots)
    def draw_window(x, y, w, h, title, active_tab_name):
        # Window container
        draw.rounded_rectangle([(x, y), (x + w, y + h)], radius=12, fill=BG_WINDOW, outline=BORDER_SUBTLE, width=2)
        # Title bar
        draw.rounded_rectangle([(x, y), (x + w, y + 48)], radius=12, fill=(18, 36, 28, 255))
        draw.rectangle([(x, y + 36), (x + w, y + 48)], fill=(18, 36, 28, 255))
        # Dots
        draw.ellipse([(x + 18, y + 18), (x + 30, y + 30)], fill=(239, 68, 68, 255))
        draw.ellipse([(x + 38, y + 18), (x + 50, y + 30)], fill=(245, 158, 11, 255))
        draw.ellipse([(x + 58, y + 18), (x + 70, y + 30)], fill=(34, 197, 94, 255))
        # Title text
        draw.text((x + 85, y + 14), title, font=font_h2, fill=TEXT_WHITE)
        # Tab Badge
        draw.rounded_rectangle([(x + w - 210, y + 10), (x + w - 16, y + 40)], radius=6, fill=BORDER_GREEN)
        draw.text((x + w - 198, y + 14), active_tab_name, font=font_badge, fill=TEXT_WHITE)

    # =========================================================================
    # PANEL 1 (LEFT): KEYSPACE BROWSER (CRUD, JSON, HEX)
    # =========================================================================
    p1_x, p1_y, p1_w, p1_h = 70, 180, 1100, 560
    draw_window(p1_x, p1_y, p1_w, p1_h, "VALENCE STUDIO • KEYSPACE DATA INSPECTOR", "TAB: KEYSPACE")

    # Left sub-pane: Key List
    draw.rectangle([(p1_x + 16, p1_y + 64), (p1_x + 380, p1_y + p1_h - 16)], fill=(13, 25, 20, 255), outline=BORDER_SUBTLE, width=1)
    draw.text((p1_x + 28, p1_y + 76), "SEARCH: user:*", font=font_mono_sm, fill=TEXT_MUTED)

    sample_keys = [
        ("user:session:100", "STRING", "256 B", "TTL: 3412s", True),
        ("user:profile:alex", "STRING", "184 B", "PERSISTENT", False),
        ("auth:jwt:token_90", "STRING", "512 B", "TTL: 840s", False),
        ("rate_limit:ip:10.5", "STRING", "64 B", "TTL: 42s", False),
        ("model:weights:emb", "STRING", "4096 B", "COLD (NVMe)", False),
    ]
    ky = p1_y + 115
    for k_name, k_type, k_size, k_ttl, is_active in sample_keys:
        bg = (24, 48, 36, 255) if is_active else (15, 30, 24, 255)
        draw.rounded_rectangle([(p1_x + 24, ky), (p1_x + 372, ky + 68)], radius=6, fill=bg, outline=BORDER_GREEN if is_active else BORDER_SUBTLE)
        draw.text((p1_x + 34, ky + 10), k_name, font=font_h2, fill=NEON_GREEN if is_active else TEXT_WHITE)
        draw.text((p1_x + 34, ky + 40), f"{k_type} • {k_size} • {k_ttl}", font=font_mono_sm, fill=TECH_CYAN if is_active else TEXT_DIM)
        ky += 80

    # Right sub-pane: Record Inspector & Hex Dump
    draw.rectangle([(p1_x + 396, p1_y + 64), (p1_x + p1_w - 16, p1_y + p1_h - 16)], fill=(13, 25, 20, 255), outline=BORDER_SUBTLE, width=1)
    draw.text((p1_x + 416, p1_y + 76), "KEY: user:session:100", font=font_h1, fill=TEXT_WHITE)
    draw.text((p1_x + 416, p1_y + 116), "TYPE: STRING  |  SIZE: 256 BYTES  |  SHARD: #2 (CPU CORE #2)  |  TIER: HOT DRAM", font=font_mono_sm, fill=TECH_CYAN)

    # Format Tabs
    draw.rounded_rectangle([(p1_x + 416, p1_y + 150), (p1_x + 510, p1_y + 180)], radius=4, fill=BORDER_GREEN)
    draw.text((p1_x + 434, p1_y + 155), "RAW", font=font_badge, fill=TEXT_WHITE)
    draw.rounded_rectangle([(p1_x + 520, p1_y + 150), (p1_x + 620, p1_y + 180)], radius=4, fill=(20, 40, 32, 255))
    draw.text((p1_x + 544, p1_y + 155), "JSON", font=font_badge, fill=TEXT_MUTED)
    draw.rounded_rectangle([(p1_x + 630, p1_y + 150), (p1_x + 730, p1_y + 180)], radius=4, fill=(20, 40, 32, 255))
    draw.text((p1_x + 654, p1_y + 155), "HEX", font=font_badge, fill=TEXT_MUTED)

    # Hex Dump preview box
    draw.rectangle([(p1_x + 416, p1_y + 196), (p1_x + p1_w - 30, p1_y + p1_h - 30)], fill=(8, 16, 12, 255), outline=BORDER_SUBTLE)
    hex_lines = [
        "00000000  7b 22 75 73 65 72 49 64  3a 22 31 30 30 22 2c 22  |{\"userId:\"100\",\"|",
        "00000010  65 6d 61 69 6c 22 3a 22  61 6c 65 78 40 67 6d 61  |email\":\"alex@gma|",
        "00000020  69 6c 2e 63 6f 6d 22 2c  22 72 6f 6c 65 22 3a 22  |il.com\",\"role\":\"|",
        "00000030  61 64 6d 69 6e 22 2c 22  74 6f 6b 65 6e 22 3a 22  |admin\",\"token\":\"|",
        "00000040  65 79 4a 68 62 47 63 69  4f 69 4a 49 55 7a 49 31  |eyJhbGciOiJIUzI1|",
        "00000050  4e 69 49 73 49 6e 52 35  63 43 49 36 49 6b 70 58  |NiIsInR5cCI6IkpX|",
        "00000060  56 43 4a 39 2e 65 79 4a  70 62 57 46 6e 61 57 35  |VCJ9.eyJpbWFnaW5|",
    ]
    hy = p1_y + 215
    for hline in hex_lines:
        draw.text((p1_x + 430, hy), hline, font=font_mono_sm, fill=TEXT_WHITE)
        hy += 34

    # =========================================================================
    # PANEL 2 (RIGHT): STORAGE TIERING CONSOLE (2-BIT CLOCK & NVMe DISK)
    # =========================================================================
    p2_x, p2_y, p2_w, p2_h = 1210, 180, 1120, 560
    draw_window(p2_x, p2_y, p2_w, p2_h, "VALENCE STUDIO • 2-BIT CLOCK & NVMe TIERING", "TAB: TIERING")

    # Big Metric Banner
    draw.rounded_rectangle([(p2_x + 20, p2_y + 64), (p2_x + p2_w - 20, p2_y + 180)], radius=8, fill=(16, 36, 26, 255), outline=BORDER_GREEN)
    draw.text((p2_x + 40, p2_y + 78), "DRAM MEMORY REDUCTION VIA TRANSPARENT NVMe PAGING", font=font_h2, fill=TEXT_MUTED)
    draw.text((p2_x + 40, p2_y + 110), "79.1% DRAM SAVINGS", font=font_title, fill=NEON_GREEN)
    draw.text((p2_x + 620, p2_y + 118), "DRAM Footprint: 2.1 MB\nCold NVMe Extents: 8.0 MB", font=font_mono, fill=TEXT_WHITE)

    # Tiered Records Table
    draw.rectangle([(p2_x + 20, p2_y + 200), (p2_x + p2_w - 20, p2_y + p2_h - 20)], fill=(11, 23, 18, 255), outline=BORDER_SUBTLE)
    draw.text((p2_x + 36, p2_y + 214), "KEY NAME", font=font_badge, fill=TEXT_MUTED)
    draw.text((p2_x + 400, p2_y + 214), "SIZE", font=font_badge, fill=TEXT_MUTED)
    draw.text((p2_x + 520, p2_y + 214), "TIER", font=font_badge, fill=TEXT_MUTED)
    draw.text((p2_x + 650, p2_y + 214), "CLOCK FREQ", font=font_badge, fill=TEXT_MUTED)
    draw.text((p2_x + 850, p2_y + 214), "DISK POINTER (12B)", font=font_badge, fill=TEXT_MUTED)

    tier_records = [
        ("user:session:active_402", "512 B", "HOT", "FREQ_3 (Hot)", "[In DRAM Slab Arena]", NEON_GREEN),
        ("auth:jwt:token_active", "256 B", "HOT", "FREQ_3 (Hot)", "[In DRAM Slab Arena]", NEON_GREEN),
        ("model:weights:cache", "1024 B", "HOT", "FREQ_2 (Warm)", "[In DRAM Slab Arena]", NEON_GREEN),
        ("report:pdf:archive_2025", "4096 B", "COLD", "FREQ_0 (Cold)", "extent:1, offset:0x0000", TECH_CYAN),
        ("user:session:dormant_891", "512 B", "COLD", "FREQ_0 (Cold)", "extent:1, offset:0x1000", TECH_CYAN),
        ("telemetry:raw:device_03", "2048 B", "COLD", "FREQ_1 (Cold)", "extent:1, offset:0x2000", TECH_CYAN),
    ]
    ty = p2_y + 250
    for r_key, r_sz, r_tier, r_clk, r_ptr, t_col in tier_records:
        draw.text((p2_x + 36, ty), r_key, font=font_mono_sm, fill=TEXT_WHITE)
        draw.text((p2_x + 400, ty), r_sz, font=font_mono_sm, fill=TEXT_MUTED)
        draw.text((p2_x + 520, ty), r_tier, font=font_badge, fill=t_col)
        draw.text((p2_x + 650, ty), r_clk, font=font_mono_sm, fill=t_col)
        draw.text((p2_x + 850, ty), r_ptr, font=font_mono_sm, fill=TEXT_DIM if "DRAM" in r_ptr else TECH_CYAN)
        ty += 46

    # =========================================================================
    # ROW 2: LIVE APM TELEMETRY & DIAGNOSTIC CLI TRACE
    # =========================================================================
    p3_x, p3_y, p3_w, p3_h = 70, 770, 2260, 560
    draw_window(p3_x, p3_y, p3_w, p3_h, "VALENCE TELEMETRY & QUERY PLAN TRACE (--explain)", "LIVE APM & CLI")

    # Left: APM Metrics Bar
    draw.rounded_rectangle([(p3_x + 20, p3_y + 64), (p3_x + 900, p3_y + p3_h - 20)], radius=8, fill=(12, 24, 18, 255), outline=BORDER_SUBTLE)
    draw.text((p3_x + 40, p3_y + 80), "LIVE CLUSTER PERFORMANCE & LATENCY METRICS", font=font_h2, fill=NEON_GREEN)

    # Metric Cards Grid
    apm_stats = [
        ("THROUGHPUT (QPS)", "8,500 ops/sec", "Under Stress Workload", NEON_GREEN),
        ("CACHE HIT RATE", "99.4%", "Zero Cold Misses", NEON_GREEN),
        ("P50 LATENCY", "3.4 µs", "Sub-Microsecond SLA", TECH_CYAN),
        ("P99 LATENCY", "8.2 µs", "Zero Tail Avalanche", TECH_CYAN),
        ("PINNED CORES", "4 Cores", "SetThreadAffinityMask", TEXT_WHITE),
        ("FRAGMENTATION", "RSS / Logical < 1.02", "Bitmapped Slab Arena", NEON_GREEN),
    ]
    ax = p3_x + 40
    ay = p3_y + 130
    for idx, (label, val, note, col) in enumerate(apm_stats):
        col_idx = idx % 2
        row_idx = idx // 2
        bx = ax + col_idx * 420
        by = ay + row_idx * 120
        draw.rounded_rectangle([(bx, by), (bx + 390, by + 100)], radius=6, fill=(16, 32, 24, 255), outline=BORDER_SUBTLE)
        draw.text((bx + 16, by + 10), label, font=font_badge, fill=TEXT_MUTED)
        draw.text((bx + 16, by + 34), val, font=font_h1, fill=col)
        draw.text((bx + 16, by + 70), note, font=font_mono_sm, fill=TEXT_DIM)

    # Right: Terminal Output of valence-cli --explain
    draw.rounded_rectangle([(p3_x + 930, p3_y + 64), (p3_x + p3_w - 20, p3_y + p3_h - 20)], radius=8, fill=(8, 16, 12, 255), outline=BORDER_SUBTLE)
    draw.text((p3_x + 950, p3_y + 80), "DIAGNOSTIC CLI ENGINE TRACE: valence-cli get user:session:100 --explain", font=font_mono_sm, fill=TECH_CYAN)

    cli_trace_lines = [
        ("┌── [1] Keyspace Ingress & Wire Protocol", NEON_GREEN),
        ("│   ├── Wire Protocol: RESP3 Zero-Copy Array  •  Vector: AVX2 256-bit SIMD Scanner", TEXT_WHITE),
        ("│   └── Parser Ingress Latency: ~0.08 µs (289M ops/sec)", TECH_CYAN),
        ("├── [2] Consistent Sharding & Thread-per-Core Dispatch", NEON_GREEN),
        ("│   ├── Murmur3-32 Hash: 0xa5c2c12c  •  Jump Consistent Hash: Shard #0 (CPU Core #0)", TEXT_WHITE),
        ("│   └── Inter-Core SPSC Queue: 64-Byte Cache-Padded Ring (0 False Sharing, ~0.15 µs)", TECH_CYAN),
        ("├── [3] Robin Hood Hash Table & Slot Probing", NEON_GREEN),
        ("│   ├── Index: Flat Array with DIB  •  Probing: O(1) Probing (Ideal Slot #300, Hop: 1)", TEXT_WHITE),
        ("│   └── Backward-Shift Deletion: Zero Dead Tombstones (~0.22 µs)", TECH_CYAN),
        ("├── [4] Memory Hierarchy & Transparent Tiering", NEON_GREEN),
        ("│   ├── Storage Tier: TIER-0 HOT (Slab Arena Pool & SSO Inline Buffer)", TEXT_WHITE),
        ("│   └── Frequency Tracker: 2-Bit CLOCK Status: FREQ_3 (Actively Hot, ~0.12 µs)", TECH_CYAN),
        ("└── [SUMMARY] End-to-End Query Latency: 2.76 µs  •  Server Response: $36 <Session Token>", NEON_GREEN),
    ]
    cly = p3_y + 120
    for cline, ccol in cli_trace_lines:
        draw.text((p3_x + 950, cly), cline, font=font_mono_sm, fill=ccol)
        cly += 30

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] UI showcase generated at {output_path} ({width}x{height})")

if __name__ == "__main__":
    generate_ui_showcase()
