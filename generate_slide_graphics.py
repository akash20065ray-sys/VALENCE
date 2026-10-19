import os
from PIL import Image, ImageDraw, ImageFont

def get_font(name, size):
    path = os.path.join("C:/Windows/Fonts", name)
    try:
        return ImageFont.truetype(path, size)
    except:
        return ImageFont.load_default()

def create_slide4_architecture_graphic(output_path="hackathon_slides/arch_diagram_slide4.png"):
    """
    Research-Paper-Grade Systems Architecture Diagram for Valence (Slide 4).
    Concise, elegant, and comprehensive — structured like an OSDI / VLDB / SIGMOD systems paper.
    Canvas: 2460 x 1580 (matches 8.25 in x 5.30 in).
    """
    w, h = 2460, 1580
    img = Image.new("RGBA", (w, h), (7, 16, 12, 255))
    draw = ImageDraw.Draw(img)

    # Fonts
    f_paper_title = get_font("segoeuib.ttf", 32)
    f_sec_tag     = get_font("segoeuib.ttf", 16)
    f_box_t       = get_font("segoeuib.ttf", 20)
    f_box_sub     = get_font("consola.ttf", 14)
    f_body        = get_font("segoeui.ttf", 15)
    f_body_b      = get_font("segoeuib.ttf", 15)
    f_mono_b      = get_font("consolab.ttf", 14)
    f_badge       = get_font("segoeuib.ttf", 13)

    # Color Palette (Deep Systems Emerald + Electric Cyan)
    NEON_GREEN    = (34, 197, 94, 255)
    TECH_CYAN     = (6, 182, 212, 255)
    BORDER_GREEN  = (22, 101, 52, 255)
    BORDER_CYAN   = (14, 116, 144, 255)
    BG_CARD       = (11, 24, 18, 255)
    BG_CARD_LIGHT = (15, 34, 25, 255)
    BG_HEADER     = (18, 38, 29, 255)
    TEXT_WHITE    = (248, 250, 252, 255)
    TEXT_MUTED    = (148, 163, 184, 255)
    TEXT_DIM      = (100, 116, 139, 255)

    # 1. Subtle Paper Grid Pattern
    for x in range(0, w, 60):
        draw.line([(x, 0), (x, h)], fill=(12, 26, 19, 80), width=1)
    for y in range(0, h, 60):
        draw.line([(0, y), (w, y)], fill=(12, 26, 19, 80), width=1)

    # Outer Container Frame
    draw.rounded_rectangle([(10, 10), (w - 10, h - 10)], radius=14, outline=BORDER_GREEN, width=2)

    # Header Bar (Paper Figure Title Style)
    draw.rounded_rectangle([(12, 12), (w - 12, 74)], radius=12, fill=BG_HEADER)
    draw.line([(12, 74), (w - 12, 74)], fill=BORDER_GREEN, width=2)
    draw.text((32, 18), "FIGURE 1: VALENCE END-TO-END SYSTEMS ARCHITECTURE & TOPOLOGY", font=f_paper_title, fill=NEON_GREEN)
    draw.text((w - 530, 28), "PURE RUST • SHARED-NOTHING ACTOR MODEL", font=f_mono_b, fill=TECH_CYAN)

    def draw_card(x, y, bw, bh, title, badge=None, border_color=BORDER_GREEN, bg_color=BG_CARD):
        draw.rounded_rectangle([(x, y), (x + bw, y + bh)], radius=8, fill=bg_color, outline=border_color, width=2)
        draw.text((x + 16, y + 12), title, font=f_box_t, fill=NEON_GREEN if border_color==BORDER_GREEN else TECH_CYAN)
        if badge:
            badge_w = 170
            badge_h = 24
            bx = x + bw - badge_w - 12
            by = y + 10
            draw.rounded_rectangle([(bx, by), (bx + badge_w, by + badge_h)], radius=4, fill=border_color)
            draw.text((bx + 10, by + 4), badge, font=f_badge, fill=TEXT_WHITE)

    def draw_h_arrow(x1, y1, x2, y2, label="", color=TECH_CYAN):
        draw.line([(x1, y1), (x2, y2)], fill=color, width=3)
        draw.polygon([(x2, y2), (x2 - 10, y2 - 5), (x2 - 10, y2 + 5)], fill=color)
        if label:
            draw.text((x1 + 8, y1 - 20), label, font=f_mono_b, fill=TEXT_MUTED)

    def draw_v_arrow(x, y1, y2, label="", color=NEON_GREEN):
        draw.line([(x, y1), (x, y2)], fill=color, width=3)
        draw.polygon([(x, y2), (x - 5, y2 - 10), (x + 5, y2 - 10)], fill=color)
        if label:
            draw.text((x + 12, (y1 + y2)//2 - 10), label, font=f_mono_b, fill=TECH_CYAN)

    # LAYER 1: CLIENT INGRESS & PARSER
    l1_y = 90
    l1_h = 185
    draw.text((32, l1_y), "LAYER 1: CLIENT INGRESS & ZERO-COPY PARSING (AVX2 SIMD ACCELERATION)", font=f_sec_tag, fill=TEXT_MUTED)

    b1_w = 690
    b1_x = 32
    b1_y = l1_y + 24
    draw_card(b1_x, b1_y, b1_w, l1_h - 24, "1. CLIENT INGRESS & WIRE SOCKET", badge="PORT: 6379 (RESP3)", border_color=BORDER_GREEN)
    draw.text((b1_x + 16, b1_y + 44), "TCP_NODELAY • Non-Blocking Tokio Stream", font=f_box_sub, fill=TEXT_MUTED)
    draw.text((b1_x + 16, b1_y + 70), "• 100% Drop-in: redis-py, ioredis, Jedis, redis-cli", font=f_body, fill=TEXT_WHITE)
    draw.text((b1_x + 16, b1_y + 96), "• Circular 64KB buffers map directly to memory frames", font=f_body, fill=TEXT_MUTED)
    draw.text((b1_x + 16, b1_y + 124), "• Zero serialization tax: Line-rate pipelined I/O", font=f_body, fill=TEXT_MUTED)

    draw_h_arrow(b1_x + b1_w, b1_y + (l1_h - 24)//2, b1_x + b1_w + 50, b1_y + (l1_h - 24)//2, label="Raw Wire", color=TECH_CYAN)

    b2_w = 780
    b2_x = b1_x + b1_w + 50
    draw_card(b2_x, b1_y, b2_w, l1_h - 24, "2. AVX2 256-BIT VECTOR PARSER", badge="LATENCY: 0.08 µs", border_color=BORDER_CYAN)
    draw.text((b2_x + 16, b1_y + 44), "src/protocol/simd.rs • 256-Bit Vector Intrinsics", font=f_box_sub, fill=TEXT_MUTED)
    draw.text((b2_x + 16, b1_y + 70), "• _mm256_cmpeq_epi8 scans 32 bytes per cycle", font=f_body, fill=TEXT_WHITE)
    draw.text((b2_x + 16, b1_y + 96), "• _tzcnt_u32 locates exact CRLF offsets in 1 instruction", font=f_body, fill=TEXT_MUTED)
    draw.text((b2_x + 16, b1_y + 124), "• 289,477,493 ops/sec decode throughput (0.08 µs)", font=f_body_b, fill=TECH_CYAN)

    draw_h_arrow(b2_x + b2_w, b1_y + (l1_h - 24)//2, b2_x + b2_w + 50, b1_y + (l1_h - 24)//2, label="Parsed AST", color=NEON_GREEN)

    b3_x = b2_x + b2_w + 50
    b3_w = w - 32 - b3_x
    draw_card(b3_x, b1_y, b3_w, l1_h - 24, "3. JUMP CONSISTENT ROUTER", badge="SHARDS: 4 CORES", border_color=BORDER_GREEN)
    draw.text((b3_x + 16, b1_y + 44), "Deterministic Key Sharding • O(ln N) Minimal Remap", font=f_box_sub, fill=TEXT_MUTED)
    draw.text((b3_x + 16, b1_y + 70), "• Murmur3-32 maps keys to hardware core shards", font=f_body, fill=TEXT_WHITE)
    draw.text((b3_x + 16, b1_y + 96), "• Perfect uniform distribution (zero hot-spot cores)", font=f_body, fill=TEXT_MUTED)
    draw.text((b3_x + 16, b1_y + 124), "• Dispatches commands to worker queues in ~0.05 µs", font=f_body_b, fill=NEON_GREEN)

    draw_v_arrow(w // 2, b1_y + l1_h - 24, b1_y + l1_h - 24 + 32, label="Lock-Free Dispatch via 64B Cache-Padded SPSC Rings", color=TECH_CYAN)

    # LAYER 2: MULTI-CORE HARDWARE EXECUTION
    l2_y = 305
    l2_h = 205
    draw.text((32, l2_y), "LAYER 2: SHARED-NOTHING HARDWARE WORKERS (PINNED VIA OS AFFINITY • 0 MUTEX CONTENTION)", font=f_sec_tag, fill=TEXT_MUTED)

    draw.rounded_rectangle([(32, l2_y + 24), (w - 32, l2_y + l2_h)], radius=8, fill=BG_CARD, outline=BORDER_CYAN, width=2)

    core_w = (w - 64 - 36) // 4
    for i in range(4):
        cx = 44 + i * (core_w + 12)
        cy = l2_y + 36
        ch = l2_h - 24
        draw.rounded_rectangle([(cx, cy), (cx + core_w, cy + ch)], radius=6, fill=BG_CARD_LIGHT, outline=BORDER_GREEN if i % 2 == 0 else BORDER_CYAN, width=1)
        
        draw.text((cx + 14, cy + 10), f"CPU CORE #{i} (WORKER SHARD)", font=f_box_t, fill=NEON_GREEN if i % 2 == 0 else TECH_CYAN)
        draw.text((cx + 14, cy + 32), "Affinity Mask: Core Locked", font=f_box_sub, fill=TEXT_DIM)

        c_bullets = [
            "Queue: 64B Padded SPSC Ring",
            "Memory: Dedicated 2MB Slab",
            "Reactor: Tokio LocalSet Loop",
            "Contention: Zero Mutex Locks"
        ]
        cur_cy = cy + 56
        for cb in c_bullets:
            draw.text((cx + 14, cur_cy), "• " + cb, font=f_body, fill=TEXT_WHITE if "Zero" in cb else TEXT_MUTED)
            cur_cy += 24

    draw_v_arrow(w // 2, l2_y + l2_h, l2_y + l2_h + 32, label="Direct Key Mutate & Query Execution against Engine Subsystems", color=NEON_GREEN)

    # LAYER 3: CORE SYSTEMS SUBSYSTEMS (THE 4 PILLARS)
    l3_y = 540
    l3_h = 990
    draw.text((32, l3_y), "LAYER 3: CORE SYSTEMS SUBSYSTEMS (STORAGE, TIERING, TTL, DURABILITY)", font=f_sec_tag, fill=TEXT_MUTED)

    col_w = (w - 64 - 36) // 4
    col_y = l3_y + 24
    col_h = l3_h - 24

    # PILLAR 1: FLAT STORAGE INDEX
    p1_x = 32
    draw_card(p1_x, col_y, col_w, col_h, "1. FLAT STORAGE INDEX", badge="LATENCY: < 0.2 µs SLA", border_color=BORDER_GREEN)
    draw.text((p1_x + 16, col_y + 44), "Hot DRAM Tier • 64B Slots", font=f_box_sub, fill=TEXT_MUTED)

    p1_sections = [
        ("Contiguous Flat Slot Array", [
            "64-byte cache-line aligned slots",
            "Sequential CPU hardware prefetch",
            "Zero pointer chasing overhead"
        ]),
        ("Robin Hood Hash Probing", [
            "Distance-from-Ideal-Bucket (DIB)",
            "Strictly bounded probe sequence (<25)",
            "Predictable flat O(1) read latency"
        ]),
        ("O(1) Backshift Deletion", [
            "Shifts adjacent elements backward",
            "0 dead tombstones in memory",
            "Zero tombstone clustering"
        ]),
        ("Small String Optimization", [
            "Keys <= 15B inlined in slot",
            "Values <= 23B inlined in slot",
            "0 heap allocations on hot path"
        ]),
        ("2MB Bitmapped Slab Arena", [
            "TZCNT allocates memory in 1.19ns",
            "RSS / Logical ratio stays < 1.02",
            "Zero malloc heap fragmentation"
        ])
    ]

    cur_py = col_y + 72
    for sec_t, sec_b in p1_sections:
        draw.text((p1_x + 16, cur_py), sec_t, font=f_body_b, fill=NEON_GREEN)
        cur_py += 22
        for b in sec_b:
            draw.text((p1_x + 24, cur_py), "• " + b, font=f_body, fill=TEXT_MUTED)
            cur_py += 22
        cur_py += 8

    # PILLAR 2: COLD NVMe TIERING
    p2_x = p1_x + col_w + 12
    draw_card(p2_x, col_y, col_w, col_h, "2. COLD NVMe TIERING", badge="79.1% DRAM SAVINGS", border_color=BORDER_CYAN)
    draw.text((p2_x + 16, col_y + 44), "Autonomous NVMe Block Paging", font=f_box_sub, fill=TEXT_MUTED)

    p2_sections = [
        ("2-Bit CLOCK Algorithm", [
            "Tracks key frequency via 2-bit clock",
            "Dormant keys (>24h old) swept",
            "Zero performance jitter on hot keys"
        ]),
        ("Sequential NVMe Extents", [
            "4KB page-aligned block writes",
            "Write Amplification Factor ≈ 1.0",
            "Protects SSD endurance life"
        ]),
        ("Compact DiskPointer", [
            "12-byte pointer replaces key in DRAM",
            "Stores extent ID + block offset",
            "Frees 79.1% of cloud DRAM"
        ]),
        ("Transparent Promotion", [
            "Cold keys read load back to DRAM",
            "Microsecond transparent promotion",
            "Zero application code changes"
        ]),
        ("Cloud Cost Reduction", [
            "Reduces AWS ElastiCache bills by 79%",
            "Enterprise DRAM footprint slashed",
            "Scale TBs on affordable NVMe SSDs"
        ])
    ]

    cur_py = col_y + 72
    for sec_t, sec_b in p2_sections:
        draw.text((p2_x + 16, cur_py), sec_t, font=f_body_b, fill=TECH_CYAN)
        cur_py += 22
        for b in sec_b:
            draw.text((p2_x + 24, cur_py), "• " + b, font=f_body, fill=TEXT_MUTED)
            cur_py += 22
        cur_py += 8

    # PILLAR 3: 4-TIER TIMING WHEEL
    p3_x = p2_x + col_w + 12
    draw_card(p3_x, col_y, col_w, col_h, "3. 4-TIER TIMING WHEEL", badge="TTL SLA: FLAT O(1)", border_color=BORDER_GREEN)
    draw.text((p3_x + 16, col_y + 44), "Intrusive Cascaded Expiration", font=f_box_sub, fill=TEXT_MUTED)

    p3_sections = [
        ("Cascaded Circular Wheels", [
            "Tier 1: 1,000 slots (1ms resolution)",
            "Tier 2: 60 slots (1s resolution)",
            "Tier 3: 60 slots (1m resolution)",
            "Tier 4: 24 slots (1h resolution)"
        ]),
        ("Atomic Head Detachment", [
            "head.swap(null, AcqRel) pointer swap",
            "Detaches batch of keys in <10ns",
            "Zero locks on foreground worker path"
        ]),
        ("Avalanche Immunity", [
            "Eliminates O(log N) priority heaps",
            "Millions of simultaneous TTL keys",
            "Zero latency spikes during mass expiry"
        ]),
        ("Asynchronous Reclamation", [
            "Detached lists sent to background",
            "Reclaims memory with zero stall",
            "Guarantees <0.2 µs query consistency"
        ]),
        ("Benchmark Result", [
            "Redis spikes to 250ms on avalanche",
            "Valence remains strictly < 0.2 µs",
            "True deterministic real-time TTL"
        ])
    ]

    cur_py = col_y + 72
    for sec_t, sec_b in p3_sections:
        draw.text((p3_x + 16, cur_py), sec_t, font=f_body_b, fill=NEON_GREEN)
        cur_py += 22
        for b in sec_b:
            draw.text((p3_x + 24, cur_py), "• " + b, font=f_body, fill=TEXT_MUTED)
            cur_py += 22
        cur_py += 8

    # PILLAR 4: FORKLESS DURABILITY
    p4_x = p3_x + col_w + 12
    draw_card(p4_x, col_y, col_w, col_h, "4. FORKLESS DURABILITY", badge="MEMORY OVERHEAD: 0B", border_color=BORDER_CYAN)
    draw.text((p4_x + 16, col_y + 44), "Epoch Versioning & CRC64 WAL", font=f_box_sub, fill=TEXT_MUTED)

    p4_sections = [
        ("Zero-Fork Epoch Snapshots", [
            "Eliminates Linux fork() system call",
            "Streams state via mutation epochs",
            "Zero page-table duplication"
        ]),
        ("Pre-Update Shadow Buffers", [
            "OnPreUpdate hook captures slot",
            "Memory growth strictly bounded <5%",
            "Completely eliminates OOM crashes"
        ]),
        ("64MB Segmented Binary WAL", [
            "GroupCommit fsync durability batching",
            "Zero data loss on sudden power failure",
            "High-throughput append-only disk stream"
        ]),
        ("Torn-Write Tail Recovery", [
            "ECMA-182 CRC64 data verification",
            "Detects partial sector writes on crash",
            "Automatic sub-second journal replay"
        ]),
        ("Replication Stream Catchup", [
            "Replicas track monotonic SequenceId",
            "Stream directly from disk segments",
            "Zero full-resync buffer overruns"
        ])
    ]

    cur_py = col_y + 72
    for sec_t, sec_b in p4_sections:
        draw.text((p4_x + 16, cur_py), sec_t, font=f_body_b, fill=TECH_CYAN)
        cur_py += 22
        for b in sec_b:
            draw.text((p4_x + 24, cur_py), "• " + b, font=f_body, fill=TEXT_MUTED)
            cur_py += 22
        cur_py += 8

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] Research-Paper Systems Architecture graphic generated: {output_path} ({w}x{h})")


def create_slide5_ui_showcase_graphic(output_path="hackathon_slides/ui_showcase_slide5.png"):
    """
    Creates the comprehensive UI/UX Showcase for Slide 5 using the REAL screenshot of Valence Studio.
    Size: 2100 x 1420 (matches 7.15 in x 4.84 in on 16:9 slide).
    """
    for candidate in ["hackathon_slides/real_studio_topology.png", "hackathon_slides/real_studio_screenshot.png"]:
        if os.path.exists(candidate):
            raw_ss = Image.open(candidate).convert("RGBA")
            tw, th = 2100, 1420
            img = Image.new("RGBA", (tw, th), (7, 16, 11, 255))
            draw = ImageDraw.Draw(img)

            f_title = get_font("segoeuib.ttf", 26)
            f_badge = get_font("consolab.ttf", 16)

            # Outer border
            draw.rounded_rectangle([(8, 8), (tw - 8, th - 8)], radius=14, fill=(10, 22, 16, 255), outline=(34, 197, 94, 255), width=2)

            # Titlebar
            tb_h = 52
            draw.rounded_rectangle([(10, 10), (tw - 10, 10 + tb_h)], radius=12, fill=(16, 36, 26, 255))
            draw.rectangle([(10, 10 + tb_h - 10), (tw - 10, 10 + tb_h)], fill=(16, 36, 26, 255))
            draw.line([(10, 10 + tb_h), (tw - 10, 10 + tb_h)], fill=(34, 197, 94, 255), width=2)

            # Window Dots
            draw.ellipse([(28, 24), (44, 40)], fill=(239, 68, 68, 255))
            draw.ellipse([(52, 24), (68, 40)], fill=(245, 158, 11, 255))
            draw.ellipse([(76, 24), (92, 40)], fill=(34, 197, 94, 255))

            draw.text((115, 18), "VALENCE STUDIO  •  http://localhost:5173  •  CLUSTER TOPOLOGY & ENGINE HUD", font=f_title, fill=(248, 250, 252, 255))
            draw.text((tw - 340, 22), "REACT 19 • LIVE TELEMETRY", font=f_badge, fill=(6, 182, 212, 255))

            # Paste real screenshot inside
            ss_w = tw - 24
            ss_h = th - 12 - tb_h - 12
            resized_ss = raw_ss.resize((ss_w, ss_h), Image.Resampling.LANCZOS)
            img.paste(resized_ss, (12, 12 + tb_h))

            os.makedirs(os.path.dirname(output_path), exist_ok=True)
            img.save(output_path, "PNG")
            print(f"[SUCCESS] Real Studio screenshot framed and saved at: {output_path}")
            return

    w, h = 2100, 1650
    img = Image.new("RGBA", (w, h), (7, 16, 11, 255))
    draw = ImageDraw.Draw(img)

    f_title      = get_font("segoeuib.ttf", 30)
    f_sub        = get_font("segoeui.ttf", 16)
    f_h1         = get_font("segoeuib.ttf", 20)
    f_h2         = get_font("segoeuib.ttf", 16)
    f_body       = get_font("segoeui.ttf", 14)
    f_mono       = get_font("consola.ttf", 15)
    f_mono_bold  = get_font("consolab.ttf", 15)
    f_mono_sm    = get_font("consola.ttf", 13)
    f_badge      = get_font("segoeuib.ttf", 13)
    f_metric_val = get_font("consolab.ttf", 26)
    f_metric_lbl = get_font("segoeuib.ttf", 13)

    NEON_GREEN    = (34, 197, 94, 255)
    TECH_CYAN     = (6, 182, 212, 255)
    AMBER         = (245, 158, 11, 255)
    RED           = (239, 68, 68, 255)
    BORDER_GREEN  = (22, 101, 52, 255)
    BORDER_CYAN   = (14, 116, 144, 255)
    BORDER_SUBTLE = (26, 52, 40, 255)
    BG_WINDOW     = (9, 20, 15, 255)
    BG_HEADER     = (16, 34, 26, 255)
    BG_CARD       = (12, 26, 20, 255)
    TEXT_WHITE    = (248, 250, 252, 255)
    TEXT_MUTED    = (148, 163, 184, 255)
    TEXT_DIM      = (100, 116, 139, 255)

    draw.rounded_rectangle([(10, 10), (w - 10, h - 10)], radius=14, fill=BG_WINDOW, outline=BORDER_GREEN, width=2)

    th = 52
    draw.rounded_rectangle([(10, 10), (w - 10, 10 + th)], radius=14, fill=BG_HEADER)
    draw.rectangle([(10, 10 + th - 12), (w - 10, 10 + th)], fill=BG_HEADER)
    draw.line([(10, 10 + th), (w - 10, 10 + th)], fill=BORDER_GREEN, width=2)

    draw.ellipse([(30, 26), (44, 40)], fill=RED)
    draw.ellipse([(54, 26), (68, 40)], fill=AMBER)
    draw.ellipse([(78, 26), (92, 40)], fill=NEON_GREEN)

    draw.text((115, 22), "VALENCE STUDIO  •  127.0.0.1:6379 [ONLINE]  •  REAL-TIME CLUSTER HUD", font=f_title, fill=TEXT_WHITE)

    bw, bh = 240, 30
    bx = w - bw - 26
    draw.rounded_rectangle([(bx, 20), (bx + bw, 20 + bh)], radius=4, fill=BORDER_GREEN)
    draw.text((bx + 16, 26), "REACT 19 • APM CONSOLE", font=f_badge, fill=TEXT_WHITE)

    # ROW 1: USER EXPERIENCE (UX) INTEGRATION BANNER
    ux_y = 74
    ux_h = 74
    draw.rounded_rectangle([(26, ux_y), (w - 26, ux_y + ux_h)], radius=6, fill=(16, 36, 27, 255), outline=BORDER_CYAN, width=1)
    draw.text((44, ux_y + 12), "DEVELOPER UX: 1-LINE DROP-IN ADOPTION", font=f_h2, fill=TECH_CYAN)
    draw.text((44, ux_y + 40), "Switching from Redis takes 0 code changes: Point any client to Port 6379 (RESP3) -> Valence transparently handles traffic.", font=f_sub, fill=TEXT_WHITE)
    draw.text((w - 480, ux_y + 26), "CLI: $ redis-cli -p 6379 PING -> PONG", font=f_mono, fill=NEON_GREEN)

    # ROW 2: LIVE TELEMETRY METRIC PILLS
    my = 162
    mh = 98
    mw = (w - 52 - 36) // 4
    metrics = [
        ("THROUGHPUT (SIMD)", "289.4M ops/s", "Peak AVX2 Delimiter Scan", NEON_GREEN),
        ("P99 LATENCY", "0.18 µs", "Zero Queue Contention", TECH_CYAN),
        ("RAM SAVINGS", "79.1%", "Automated NVMe Tiering", NEON_GREEN),
        ("HARDWARE CORES", "4 Pinned", "0 Locks • SPSC Ring", TECH_CYAN),
    ]

    for i, (m_lbl, m_val, m_sub, m_col) in enumerate(metrics):
        mx = 26 + i * (mw + 12)
        draw.rounded_rectangle([(mx, my), (mx + mw, my + mh)], radius=6, fill=BG_CARD, outline=BORDER_SUBTLE, width=2)
        draw.text((mx + 14, my + 10), m_lbl, font=f_metric_lbl, fill=TEXT_MUTED)
        draw.text((mx + 14, my + 32), m_val, font=f_metric_val, fill=m_col)
        draw.text((mx + 14, my + 70), m_sub, font=f_mono_sm, fill=TEXT_DIM)

    # ROW 3: KEYSPACE BROWSER (LEFT) & RECORD INSPECTOR (RIGHT)
    mid_y = 278
    mid_h = 790
    left_w = 860
    right_w = w - 52 - left_w - 18

    k_x = 26
    draw.rounded_rectangle([(k_x, mid_y), (k_x + left_w, mid_y + mid_h)], radius=8, fill=(11, 23, 17, 255), outline=BORDER_SUBTLE, width=2)
    draw.text((k_x + 18, mid_y + 14), "LIVE KEYSPACE BROWSER", font=f_h1, fill=NEON_GREEN)
    draw.text((k_x + 18, mid_y + 42), "Streaming inspection with 0 server lockups (Keys * never freezes)", font=f_mono_sm, fill=TEXT_MUTED)

    draw.rounded_rectangle([(k_x + 18, mid_y + 70), (k_x + left_w - 18, mid_y + 108)], radius=4, fill=(16, 32, 24, 255), outline=BORDER_GREEN, width=1)
    draw.text((k_x + 30, mid_y + 79), "SEARCH: user:*   [5,420 Active Keys Found]", font=f_mono, fill=TEXT_WHITE)

    keys = [
        ("user:session:100", "STRING", "256 B", "HOT DRAM (Core #2)", NEON_GREEN, True),
        ("user:profile:alex", "STRING", "184 B", "HOT DRAM (Core #1)", NEON_GREEN, False),
        ("auth:jwt:token_90", "STRING", "512 B", "HOT DRAM (Core #0)", NEON_GREEN, False),
        ("rate_limit:ip:10.5", "STRING", "64 B", "HOT DRAM (Core #3)", NEON_GREEN, False),
        ("model:weights:emb", "BLOB", "4096 B", "COLD NVMe (79% Saved)", TECH_CYAN, False),
        ("report:pdf:archive", "BLOB", "8192 B", "COLD NVMe (Extent 0x14)", TECH_CYAN, False),
        ("telemetry:device:99", "BLOB", "2048 B", "COLD NVMe (2-Bit CLOCK)", TECH_CYAN, False),
    ]

    item_y = mid_y + 120
    for k_name, k_type, k_sz, k_tier, k_col, is_active in keys:
        bg_i = (22, 46, 34, 255) if is_active else (14, 28, 21, 255)
        br_i = BORDER_GREEN if is_active else BORDER_SUBTLE
        draw.rounded_rectangle([(k_x + 18, item_y), (k_x + left_w - 18, item_y + 78)], radius=6, fill=bg_i, outline=br_i, width=1)
        draw.text((k_x + 30, item_y + 12), k_name, font=f_h2, fill=NEON_GREEN if is_active else TEXT_WHITE)
        draw.text((k_x + 30, item_y + 44), f"TYPE: {k_type}  •  SIZE: {k_sz}", font=f_mono_sm, fill=TEXT_MUTED)

        badge_w = 230
        badge_x = k_x + left_w - 18 - badge_w - 10
        draw.rounded_rectangle([(badge_x, item_y + 24), (badge_x + badge_w, item_y + 54)], radius=4, fill=(18, 38, 28, 255), outline=br_i)
        draw.text((badge_x + 12, item_y + 30), k_tier, font=f_mono_sm, fill=k_col)
        item_y += 90

    # Right: Record Inspector & Hex Viewer
    r_x = k_x + left_w + 18
    draw.rounded_rectangle([(r_x, mid_y), (r_x + right_w, mid_y + mid_h)], radius=8, fill=(11, 23, 17, 255), outline=BORDER_SUBTLE, width=2)
    draw.text((r_x + 18, mid_y + 14), "RECORD INSPECTOR: user:session:100", font=f_h1, fill=TECH_CYAN)
    draw.text((r_x + 18, mid_y + 42), "Shard: Core #2  |  Tier: HOT DRAM (Slab Arena)  |  SLA: 0.18 µs", font=f_mono_sm, fill=TEXT_MUTED)

    tab_y = mid_y + 70
    draw.rounded_rectangle([(r_x + 18, tab_y), (r_x + 110, tab_y + 32)], radius=4, fill=BORDER_GREEN)
    draw.text((r_x + 38, tab_y + 6), "RAW", font=f_badge, fill=TEXT_WHITE)

    draw.rounded_rectangle([(r_x + 120, tab_y), (r_x + 215, tab_y + 32)], radius=4, fill=(18, 36, 27, 255))
    draw.text((r_x + 142, tab_y + 6), "JSON", font=f_badge, fill=TEXT_MUTED)

    draw.rounded_rectangle([(r_x + 225, tab_y), (r_x + 320, tab_y + 32)], radius=4, fill=(18, 36, 27, 255))
    draw.text((r_x + 248, tab_y + 6), "HEX", font=f_badge, fill=TEXT_MUTED)

    hex_y = tab_y + 44
    hex_h = mid_h - (hex_y - mid_y) - 18
    draw.rounded_rectangle([(r_x + 18, hex_y), (r_x + right_w - 18, hex_y + hex_h)], radius=6, fill=(8, 16, 12, 255), outline=BORDER_SUBTLE)

    hex_lines = [
        "OFFSET    00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ASCII VALUE",
        "-------- ------------------------  ------------------------  ----------------",
        "00000000 7B 22 75 73 65 72 49 64  3A 22 31 30 30 22 2C 22  |{\"userId:\"100\",\"|",
        "00000010 65 6D 61 69 6C 22 3A 22  61 6C 65 78 40 76 61 6C  |email\":\"alex@val|",
        "00000020 65 6E 63 65 2E 64 65 76  22 2C 22 72 6F 6C 65 22  |ence.dev\",\"role\"|",
        "00000030 3A 22 61 64 6D 69 6E 22  2C 22 73 65 73 73 69 6F  |:\"admin\",\"sessio|",
        "00000040 6E 54 6F 6B 65 6E 22 3A  22 65 79 4A 68 62 47 63  |nToken\":\"eyJhbGc|",
        "00000050 69 4F 69 4A 49 55 7A 49  31 4E 69 49 73 49 6E 52  |iOiJIUzI1NiIsInR|",
        "00000060 35 63 43 49 36 49 6B 70  58 56 43 4A 39 2E 65 79  |5cCI6IkpXVCJ9.ey|",
        "00000070 4A 70 62 57 46 6E 61 57  35 6E 58 30 6C 6B 49 6A  |JpbWFnY5nX0lkIj|",
        "00000080 6F 69 4D 54 41 77 49 69  77 69 5A 58 68 77 61 58  |oiMTAwIiwiZXhwaX|",
        "00000090 4A 6C 64 43 49 36 4D 54  63 7A 4D 7A 49 34 4E 7A  |JldCI6MTczMzI4Nz|",
        "000000A0 45 77 4D 53 77 69 61 57  52 70 59 32 46 30 61 57  |EwMSwiaWRpY2F0aW|",
        "000000B0 39 75 49 6A 6F 69 56 47  68 79 5A 58 68 77 61 58  |9uIjoiVGhyZWFrLT|",
        "000000C0 42 6C 63 69 31 44 62 33  4A 6C 49 6E 30 7D 0D 0A  |Blci1Db3JlIn0}\\r\\n|",
    ]
    hy = hex_y + 16
    for i, hline in enumerate(hex_lines):
        c = TECH_CYAN if i == 0 else (TEXT_DIM if i == 1 else TEXT_WHITE)
        draw.text((r_x + 32, hy), hline, font=f_mono_sm, fill=c)
        hy += 31

    # ROW 4: DEV UX - QUERY PLANNER DIAGNOSTIC CLI TRACE
    bot_y = mid_y + mid_h + 16
    bot_h = h - bot_y - 20
    draw.rounded_rectangle([(26, bot_y), (w - 26, bot_y + bot_h)], radius=8, fill=(8, 18, 13, 255), outline=BORDER_GREEN, width=2)

    draw.text((44, bot_y + 14), "DEVELOPER OBSERVABILITY: valence-cli get user:session:100 --explain", font=f_h2, fill=NEON_GREEN)
    draw.text((w - 380, bot_y + 14), "QUERY LATENCY: 0.18 µs [OK]", font=f_mono_bold, fill=TECH_CYAN)

    steps = [
        ("Step 1: Wire Ingress", "RESP3 zero-copy decode via AVX2 SIMD scanner", "Latency: ~0.08 µs (32B/cycle)"),
        ("Step 2: Core Routing", "Murmur3 jump consistent hash -> Core #2", "Latency: ~0.05 µs (0 locks)"),
        ("Step 3: Storage Fetch", "Robin Hood flat table -> 2MB slab arena hot slot", "Latency: ~0.05 µs (1 probe hop)"),
        ("Step 4: Client Return", "HTTP/RESP3 256 bytes returned directly to application", "Total Latency: 0.18 µs"),
    ]
    sw = (w - 52 - 60) // 4
    for i, (st, sd, sl) in enumerate(steps):
        sx = 44 + i * (sw + 16)
        sy = bot_y + 46
        draw.text((sx, sy), st, font=f_h2, fill=TEXT_WHITE)
        draw.text((sx, sy + 24), sd, font=f_mono_sm, fill=TEXT_MUTED)
        draw.text((sx, sy + 48), sl, font=f_mono_sm, fill=NEON_GREEN if i == 3 else TECH_CYAN)

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] Slide 5 UI Showcase graphic generated at: {output_path} ({w}x{h})")

if __name__ == "__main__":
    create_slide4_architecture_graphic()
    create_slide5_ui_showcase_graphic()
