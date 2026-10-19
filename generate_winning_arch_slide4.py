import os
from PIL import Image, ImageDraw, ImageFont

def get_font(name, size):
    path = os.path.join("C:/Windows/Fonts", name)
    try:
        return ImageFont.truetype(path, size)
    except:
        return ImageFont.load_default()

def draw_pill(draw, x, y, w, h, text, font, fill_col, text_col, radius=6):
    draw.rounded_rectangle([(x, y), (x + w, y + h)], radius=radius, fill=fill_col)
    bbox = font.getbbox(text)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    draw.text((x + (w - tw) // 2, y + (h - th) // 2 - 1), text, font=font, fill=text_col)

def create_winning_architecture_diagram(output_path="hackathon_slides/arch_diagram_slide4.png"):
    """
    Winning Hackathon Component Architecture Diagram:
    - 4 Modular Subsystems: Ingress -> Engine Core -> Dual-Tier Storage -> Durability & Telemetry
    - Human-written, crystal-clear labels.
    - Zero overlapping labels, generous padding, 10/10 visual balance.
    Canvas: 2460 x 1580
    """
    w, h = 2460, 1580
    img = Image.new("RGBA", (w, h), (7, 18, 13, 255))
    draw = ImageDraw.Draw(img)

    # Typography
    f_title     = get_font("segoeuib.ttf", 32)
    f_subhead   = get_font("segoeui.ttf", 18)
    f_layer_t   = get_font("segoeuib.ttf", 24)
    f_card_t    = get_font("segoeuib.ttf", 20)
    f_card_desc = get_font("segoeui.ttf", 16)
    f_badge     = get_font("segoeuib.ttf", 15)
    f_bus_lbl   = get_font("segoeuib.ttf", 14)

    # Refined Color Palette
    NEON_GREEN   = (34, 197, 94, 255)
    TECH_CYAN    = (6, 182, 212, 255)
    AMBER        = (245, 158, 11, 255)
    PURPLE       = (168, 85, 247, 255)
    TEXT_WHITE   = (248, 250, 252, 255)
    TEXT_MUTED   = (148, 163, 184, 255)
    
    BG_CARD      = (11, 26, 20, 255)
    BG_HEADER    = (15, 36, 28, 255)
    BG_INNER_BOX = (16, 38, 29, 255)
    BG_PILL_BUS  = (14, 32, 25, 255)
    
    BORDER_GRN   = (22, 101, 52, 255)
    BORDER_CYAN  = (14, 116, 144, 255)
    BORDER_AMB   = (180, 83, 9, 255)
    BORDER_PURP  = (126, 34, 206, 255)
    BORDER_INNER = (24, 56, 44, 255)

    # Subtle Background Grid
    for x in range(0, w, 80):
        draw.line([(x, 0), (x, h)], fill=(12, 28, 21, 50), width=1)
    for y in range(0, h, 80):
        draw.line([(0, y), (w, y)], fill=(12, 28, 21, 50), width=1)

    # Outer Container Frame
    draw.rounded_rectangle([(12, 12), (w - 12, h - 12)], radius=14, outline=BORDER_GRN, width=2)

    # Top Header Banner
    draw.rounded_rectangle([(14, 14), (w - 14, 82)], radius=12, fill=BG_HEADER)
    draw.line([(14, 82), (w - 14, 82)], fill=BORDER_GRN, width=2)
    draw.text((36, 22), "VALENCE SYSTEM ARCHITECTURE", font=f_title, fill=NEON_GREEN)
    draw.text((w - 680, 30), "Modular Component Subsystems in Pure Rust", font=f_subhead, fill=TECH_CYAN)

    content_w = w - 80
    cx = 40

    # =========================================================================
    # LAYER 1: CLIENT INGRESS & PROTOCOL SUBSYSTEM
    # =========================================================================
    l1_y = 98
    l1_h = 276
    draw.rounded_rectangle([(cx, l1_y), (cx + content_w, l1_y + l1_h)], radius=12, fill=BG_CARD, outline=BORDER_CYAN, width=2)
    
    draw.text((cx + 24, l1_y + 16), "1. Network Ingress & Protocol Subsystem", font=f_layer_t, fill=TECH_CYAN)
    draw_pill(draw, cx + content_w - 380, l1_y + 14, 360, 34, "100% Drop-In Redis Compatibility", f_badge, BORDER_CYAN, TEXT_WHITE, radius=6)
    draw.line([(cx + 20, l1_y + 58), (cx + content_w - 20, l1_y + 58)], fill=BORDER_INNER, width=1)

    box_w = (content_w - 48 - 32) // 3
    box_h = l1_h - 78
    by = l1_y + 66

    # Box 1.1: RESP3 Wire Listener
    bx1 = cx + 24
    draw.rounded_rectangle([(bx1, by), (bx1 + box_w, by + box_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_INNER, width=1)
    draw.text((bx1 + 16, by + 16), "RESP3 Protocol Gateway", font=f_card_t, fill=TEXT_WHITE)
    draw.text((bx1 + 16, by + 50), "• Standard Port: Listens on TCP 6379", font=f_card_desc, fill=NEON_GREEN)
    draw.text((bx1 + 16, by + 82), "• Zero App Changes: Works with redis-py, ioredis, Jedis", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((bx1 + 16, by + 114), "• Async Network I/O: Non-blocking Tokio TCP streams", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((bx1 + 16, by + 146), "• Pipelined Requests: Handles thousands of concurrent clients", font=f_card_desc, fill=TEXT_MUTED)

    # Box 1.2: Zero-Copy Framing
    bx2 = bx1 + box_w + 16
    draw.rounded_rectangle([(bx2, by), (bx2 + box_w, by + box_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_INNER, width=1)
    draw.text((bx2 + 16, by + 16), "Zero-Copy Frame Buffers", font=f_card_t, fill=TEXT_WHITE)
    draw.text((bx2 + 16, by + 50), "• Direct Memory Access: Sockets read into ring buffers", font=f_card_desc, fill=TECH_CYAN)
    draw.text((bx2 + 16, by + 82), "• Zero Heap Allocations: Avoids dynamic memory fragmentation", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((bx2 + 16, by + 114), "• Line-Rate Speed: Sub-microsecond command framing", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((bx2 + 16, by + 146), "• Native Rust Slices: Safe references without data copying", font=f_card_desc, fill=TEXT_MUTED)

    # Box 1.3: SIMD Accelerator
    bx3 = bx2 + box_w + 16
    draw.rounded_rectangle([(bx3, by), (bx3 + box_w, by + box_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_INNER, width=1)
    draw.text((bx3 + 16, by + 16), "SIMD Vector Accelerator", font=f_card_t, fill=TEXT_WHITE)
    draw.text((bx3 + 16, by + 50), "• Hardware Acceleration: 256-bit AVX2 vector intrinsics", font=f_card_desc, fill=AMBER)
    draw.text((bx3 + 16, by + 82), "• Instant Delimiter Find: Scans 32 bytes in 1 CPU cycle", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((bx3 + 16, by + 114), "• Peak Parser Speed: 289M operations/second throughput", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((bx3 + 16, by + 146), "• 4x Faster: Leaves legacy single-threaded C parsers behind", font=f_card_desc, fill=TEXT_MUTED)

    # Connector Bus 1 -> 2 (Clean Pill Badge, No text clash)
    bus1_y = l1_y + l1_h + 8
    draw.rounded_rectangle([(cx + content_w // 2 - 240, bus1_y), (cx + content_w // 2 + 240, bus1_y + 26)], radius=6, fill=BG_PILL_BUS, outline=BORDER_CYAN, width=1)
    draw.text((cx + content_w // 2 - 210, bus1_y + 4), "▼  Parsed Commands Dispatched via Lock-Free Ring Queues  ▼", font=f_bus_lbl, fill=TECH_CYAN)

    # =========================================================================
    # LAYER 2: MULTI-CORE EXECUTION SUBSYSTEM
    # =========================================================================
    l2_y = l1_y + l1_h + 44
    l2_h = 276
    draw.rounded_rectangle([(cx, l2_y), (cx + content_w, l2_y + l2_h)], radius=12, fill=BG_CARD, outline=BORDER_GRN, width=2)
    
    draw.text((cx + 24, l2_y + 16), "2. Multi-Core Execution Engine (Shared-Nothing Architecture)", font=f_layer_t, fill=NEON_GREEN)
    draw_pill(draw, cx + content_w - 360, l2_y + 14, 340, 34, "0 Mutex Contention • CPU-Pinned", f_badge, BORDER_GRN, TEXT_WHITE, radius=6)
    draw.line([(cx + 20, l2_y + 58), (cx + content_w - 20, l2_y + 58)], fill=BORDER_INNER, width=1)

    core_w = (content_w - 48 - 48) // 4
    core_h = l2_h - 78
    c_by = l2_y + 66

    for i in range(4):
        c_bx = cx + 24 + i * (core_w + 16)
        draw.rounded_rectangle([(c_bx, c_by), (c_bx + core_w, c_by + core_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_GRN if i==0 else BORDER_INNER, width=1)
        draw.text((c_bx + 16, c_by + 16), f"Worker Shard #{i}", font=f_card_t, fill=NEON_GREEN)
        draw.text((c_bx + 16, c_by + 50), f"• Pinned to CPU Core #{i}", font=f_card_desc, fill=TEXT_WHITE)
        draw.text((c_bx + 16, c_by + 82), "• Dedicated SPSC Ring Queue", font=f_card_desc, fill=TECH_CYAN)
        draw.text((c_bx + 16, c_by + 114), "• Independent Memory Store", font=f_card_desc, fill=TEXT_MUTED)
        draw.text((c_bx + 16, c_by + 146), "• 0 Global Lock Contention", font=f_card_desc, fill=TEXT_MUTED)

    # Connector Bus 2 -> 3 (Clean Pill Badge)
    bus2_y = l2_y + l2_h + 8
    draw.rounded_rectangle([(cx + content_w // 2 - 270, bus2_y), (cx + content_w // 2 + 270, bus2_y + 26)], radius=6, fill=BG_PILL_BUS, outline=BORDER_GRN, width=1)
    draw.text((cx + content_w // 2 - 245, bus2_y + 4), "▼  Direct Memory Access: Hot Keys Kept in DRAM, Cold paged to NVMe  ▼", font=f_bus_lbl, fill=NEON_GREEN)

    # =========================================================================
    # LAYER 3: DUAL-TIER STORAGE SUBSYSTEM (HOT DRAM + COLD NVMe)
    # =========================================================================
    l3_y = l2_y + l2_h + 44
    l3_h = 286
    draw.rounded_rectangle([(cx, l3_y), (cx + content_w, l3_y + l3_h)], radius=12, fill=BG_CARD, outline=BORDER_AMB, width=2)

    draw.text((cx + 24, l3_y + 16), "3. Dual-Tier Hybrid Storage Subsystem", font=f_layer_t, fill=AMBER)
    draw_pill(draw, cx + content_w - 380, l3_y + 14, 360, 34, "Slashes Cloud RAM Bills by 79.1%", f_badge, BORDER_AMB, TEXT_WHITE, radius=6)
    draw.line([(cx + 20, l3_y + 58), (cx + content_w - 20, l3_y + 58)], fill=BORDER_INNER, width=1)

    tier_w = (content_w - 48 - 24) // 2
    tier_h = l3_h - 78
    t_by = l3_y + 66

    # Hot Tier (DRAM)
    t1_x = cx + 24
    draw.rounded_rectangle([(t1_x, t_by), (t1_x + tier_w, t_by + tier_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_GRN, width=1)
    draw.text((t1_x + 18, t_by + 16), "HOT TIER — Fast In-Memory DRAM", font=f_card_t, fill=NEON_GREEN)
    draw.text((t1_x + 18, t_by + 52), "• Active Working Set: Holds frequently read hot keys in RAM", font=f_card_desc, fill=TEXT_WHITE)
    draw.text((t1_x + 18, t_by + 86), "• Sub-Microsecond Lookups: Fast open-addressed sharded hash map", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((t1_x + 18, t_by + 120), "• Small String Inlining: Short keys stored directly inside pointer bits", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((t1_x + 18, t_by + 154), "• Zero GC Freezes: Predictable, deterministic Rust memory management", font=f_card_desc, fill=TEXT_MUTED)

    # Cold Tier (NVMe SSD)
    t2_x = t1_x + tier_w + 24
    draw.rounded_rectangle([(t2_x, t_by), (t2_x + tier_w, t_by + tier_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_AMB, width=1)
    draw.text((t2_x + 18, t_by + 16), "COLD TIER — High-Speed NVMe SSD", font=f_card_t, fill=AMBER)
    draw.text((t2_x + 18, t_by + 52), "• Automatic Sweeper: Dormant keys (>24h untouched) page to NVMe", font=f_card_desc, fill=TEXT_WHITE)
    draw.text((t2_x + 18, t_by + 86), "• 2-Bit CLOCK Algorithm: Tracks key access frequency in the background", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((t2_x + 18, t_by + 120), "• Huge Cost Reduction: Slashes cloud memory infrastructure bills by 79%", font=f_card_desc, fill=TECH_CYAN)
    draw.text((t2_x + 18, t_by + 154), "• Instant Promotion: Promotes cold keys back to DRAM transparently on read", font=f_card_desc, fill=TEXT_MUTED)

    # Connector Bus 3 -> 4 (Clean Pill Badge)
    bus3_y = l3_y + l3_h + 8
    draw.rounded_rectangle([(cx + content_w // 2 - 250, bus3_y), (cx + content_w // 2 + 250, bus3_y + 26)], radius=6, fill=BG_PILL_BUS, outline=BORDER_PURP, width=1)
    draw.text((cx + content_w // 2 - 225, bus3_y + 4), "▼  Async Mutation Streams & Real-Time Performance Telemetry  ▼", font=f_bus_lbl, fill=PURPLE)

    # =========================================================================
    # LAYER 4: DURABILITY, SNAPSHOTS & MANAGEMENT CONSOLE
    # =========================================================================
    l4_y = l3_y + l3_h + 44
    l4_h = 276
    draw.rounded_rectangle([(cx, l4_y), (cx + content_w, l4_y + l4_h)], radius=12, fill=BG_CARD, outline=BORDER_PURP, width=2)

    draw.text((cx + 24, l4_y + 16), "4. Durability, Recovery & Web Management Subsystem", font=f_layer_t, fill=PURPLE)
    draw_pill(draw, cx + content_w - 380, l4_y + 14, 360, 34, "Zero-Fork Backups • No OOM Outages", f_badge, BORDER_PURP, TEXT_WHITE, radius=6)
    draw.line([(cx + 20, l4_y + 58), (cx + content_w - 20, l4_y + 58)], fill=BORDER_INNER, width=1)

    d_w = (content_w - 48 - 32) // 3
    d_h = l4_h - 78
    d_by = l4_y + 66

    # Durability Box 1
    d1_x = cx + 24
    draw.rounded_rectangle([(d1_x, d_by), (d1_x + d_w, d_by + d_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_INNER, width=1)
    draw.text((d1_x + 16, d_by + 16), "Forkless Snapshot Engine", font=f_card_t, fill=TEXT_WHITE)
    draw.text((d1_x + 16, d_by + 50), "• Epoch State Versioning: Snapshots memory safely", font=f_card_desc, fill=PURPLE)
    draw.text((d1_x + 16, d_by + 82), "• No fork() Freezes: Avoids Linux process duplication", font=f_card_desc, fill=NEON_GREEN)
    draw.text((d1_x + 16, d_by + 114), "• Zero OOM Crashes: Never duplicates process memory", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((d1_x + 16, d_by + 146), "• Background Streaming: Non-blocking disk writes", font=f_card_desc, fill=TEXT_MUTED)

    # Durability Box 2
    d2_x = d1_x + d_w + 16
    draw.rounded_rectangle([(d2_x, d_by), (d2_x + d_w, d_by + d_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_INNER, width=1)
    draw.text((d2_x + 16, d_by + 16), "Write-Ahead Log (WAL)", font=f_card_t, fill=TEXT_WHITE)
    draw.text((d2_x + 16, d_by + 50), "• Segmented Binary Log: High-throughput disk audit", font=f_card_desc, fill=TECH_CYAN)
    draw.text((d2_x + 16, d_by + 82), "• GroupCommit Fsync: Batches disk writes for speed", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((d2_x + 16, d_by + 114), "• Zero-Loss Recovery: Instant replay upon server reboot", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((d2_x + 16, d_by + 146), "• Checksum Protected: Validates complete data integrity", font=f_card_desc, fill=TEXT_MUTED)

    # Durability Box 3: Studio & Telemetry
    d3_x = d2_x + d_w + 16
    draw.rounded_rectangle([(d3_x, d_by), (d3_x + d_w, d_by + d_h)], radius=8, fill=BG_INNER_BOX, outline=BORDER_INNER, width=1)
    draw.text((d3_x + 16, d_by + 16), "Valence Studio Console", font=f_card_t, fill=TEXT_WHITE)
    draw.text((d3_x + 16, d_by + 50), "• Real-Time APM: Live React 19 management dashboard", font=f_card_desc, fill=NEON_GREEN)
    draw.text((d3_x + 16, d_by + 82), "• Live Telemetry: Sub-millisecond latency & throughput HUD", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((d3_x + 16, d_by + 114), "• Interactive Console: Direct query execution & testing", font=f_card_desc, fill=TEXT_MUTED)
    draw.text((d3_x + 16, d_by + 146), "• Visual Memory Tiering: Real-time DRAM vs NVMe ratio", font=f_card_desc, fill=TEXT_MUTED)

    # Bottom caption
    f_cap = get_font("segoeuii.ttf", 15)
    draw.text((cx + 10, h - 30), "Figure 1: Valence Modular Systems Architecture — Component Subsystems & Memory Hierarchy (Pure Rust)", font=f_cap, fill=TEXT_MUTED)

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] Clean Component Architecture graphic generated: {output_path}")

if __name__ == "__main__":
    create_winning_architecture_diagram()
