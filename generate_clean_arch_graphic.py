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

def create_layered_subsystems_architecture(output_path="hackathon_slides/arch_diagram_slide4.png"):
    """
    True Layered Subsystems Architecture Diagram (Component Block Architecture):
    - NOT a flowchart / NO step-by-step arrows.
    - 4 Architectural Layers:
      Layer 1: Network Ingress & Vector Protocol Subsystem
      Layer 2: Shared-Nothing Multi-Core Execution Subsystem
      Layer 3: Hybrid Dual-Tier Storage Subsystem (DRAM + NVMe)
      Layer 4: Fault Tolerance & Durability Subsystem
    Canvas: 2460 x 1580
    """
    w, h = 2460, 1580
    img = Image.new("RGBA", (w, h), (8, 18, 13, 255))
    draw = ImageDraw.Draw(img)

    # Fonts
    f_title      = get_font("segoeuib.ttf", 32)
    f_subhead    = get_font("consolab.ttf", 17)
    f_layer_hdr  = get_font("segoeuib.ttf", 23)
    f_block_t    = get_font("segoeuib.ttf", 21)
    f_block_sub  = get_font("segoeui.ttf", 17)
    f_block_dim  = get_font("segoeui.ttf", 15)
    f_badge      = get_font("consolab.ttf", 16)
    f_mono_sm    = get_font("consola.ttf", 15)
    f_caption    = get_font("segoeuii.ttf", 15)

    # Colors
    NEON_GREEN    = (34, 197, 94, 255)
    TECH_CYAN     = (6, 182, 212, 255)
    AMBER         = (245, 158, 11, 255)
    PURPLE        = (168, 85, 247, 255)
    TEXT_WHITE    = (255, 255, 255, 255)
    TEXT_MUTED    = (160, 175, 195, 255)
    TEXT_DIM      = (120, 140, 160, 255)

    BG_HEADER     = (16, 38, 28, 255)
    BG_LAYER      = (11, 25, 18, 255)
    BG_BLOCK_GRN  = (16, 38, 27, 255)
    BG_BLOCK_CYAN = (13, 35, 42, 255)
    BG_BLOCK_AMB  = (34, 30, 16, 255)
    BG_BLOCK_PURP = (28, 20, 38, 255)

    BORDER_GREEN  = (34, 197, 94, 255)
    BORDER_CYAN   = (6, 182, 212, 255)
    BORDER_AMBER  = (245, 158, 11, 255)
    BORDER_PURPLE = (168, 85, 247, 255)
    BORDER_MUTED  = (24, 52, 38, 255)
    BORDER_SUBTLE = (20, 42, 32, 255)

    # Outer Container Frame
    draw.rounded_rectangle([(10, 10), (w - 10, h - 10)], radius=14, outline=BORDER_GREEN, width=2)

    # Top Header
    draw.rounded_rectangle([(12, 12), (w - 12, 74)], radius=12, fill=BG_HEADER)
    draw.line([(12, 74), (w - 12, 74)], fill=BORDER_GREEN, width=2)
    draw.text((36, 18), "VALENCE SYSTEM ARCHITECTURE", font=f_title, fill=NEON_GREEN)
    draw.text((w - 740, 26), "MODULAR SUBSYSTEM & COMPONENT MATRIX (PURE RUST)", font=f_subhead, fill=TECH_CYAN)

    content_w = w - 80
    cx = 40

    # =========================================================================
    # LAYER 1: NETWORK INGRESS & PROTOCOL SUBSYSTEM
    # =========================================================================
    l1_y = 92
    l1_h = 300
    draw.rounded_rectangle([(cx, l1_y), (cx + content_w, l1_y + l1_h)], radius=10, fill=BG_LAYER, outline=BORDER_CYAN, width=2)

    # Layer Header Bar
    draw.text((cx + 24, l1_y + 14), "LAYER 1: NETWORK INGRESS & VECTOR PROTOCOL SUBSYSTEM", font=f_layer_hdr, fill=TECH_CYAN)
    draw_pill(draw, cx + content_w - 430, l1_y + 12, 410, 32, "289M ops/sec • Latency: 0.08 µs • Drop-In Redis", f_badge, BORDER_CYAN, TEXT_WHITE)
    draw.line([(cx + 20, l1_y + 52), (cx + content_w - 20, l1_y + 52)], fill=BORDER_MUTED, width=1)

    # 3 Component Blocks in Layer 1
    b1_w = (content_w - 48 - 32) // 3
    b1_h = l1_h - 76
    b1_y = l1_y + 64

    # Block 1.1: RESP3 Wire Listener
    bx1 = cx + 24
    draw.rounded_rectangle([(bx1, b1_y), (bx1 + b1_w, b1_y + b1_h)], radius=8, fill=BG_BLOCK_CYAN, outline=BORDER_SUBTLE, width=2)
    draw.text((bx1 + 18, b1_y + 16), "RESP3 Wire Listener (Port 6379)", font=f_block_t, fill=TEXT_WHITE)
    draw_pill(draw, bx1 + b1_w - 120, b1_y + 14, 105, 26, "TCP_NODELAY", f_badge, BORDER_CYAN, TEXT_WHITE)
    draw.line([(bx1 + 18, b1_y + 50), (bx1 + b1_w - 18, b1_y + 50)], fill=BORDER_MUTED, width=1)
    draw.text((bx1 + 18, b1_y + 64), "• Non-blocking Tokio asynchronous stream", font=f_block_sub, fill=TEXT_WHITE)
    draw.text((bx1 + 18, b1_y + 94), "• Line-rate pipelined I/O socket buffering", font=f_block_sub, fill=TEXT_MUTED)
    draw.text((bx1 + 18, b1_y + 124), "• Drop-in: redis-py, ioredis, Jedis, Go-redis", font=f_mono_sm, fill=TECH_CYAN)
    draw.text((bx1 + 18, b1_y + 154), "• Zero application code rewrites required", font=f_block_dim, fill=TEXT_DIM)

    # Block 1.2: AVX2 256-Bit SIMD Parser
    bx2 = bx1 + b1_w + 16
    draw.rounded_rectangle([(bx2, b1_y), (bx2 + b1_w, b1_y + b1_h)], radius=8, fill=BG_BLOCK_GRN, outline=BORDER_GREEN, width=2)
    draw.text((bx2 + 18, b1_y + 16), "AVX2 SIMD Vector Parser", font=f_block_t, fill=NEON_GREEN)
    draw_pill(draw, bx2 + b1_w - 120, b1_y + 14, 105, 26, "256-BIT SIMD", f_badge, BORDER_GREEN, TEXT_WHITE)
    draw.line([(bx2 + 18, b1_y + 50), (bx2 + b1_w - 18, b1_y + 50)], fill=BORDER_MUTED, width=1)
    draw.text((bx2 + 18, b1_y + 64), "• _mm256_cmpeq_epi8 scans 32 bytes per cycle", font=f_block_sub, fill=TEXT_WHITE)
    draw.text((bx2 + 18, b1_y + 94), "• _tzcnt_u32 finds CRLF offsets in 1 instruction", font=f_block_sub, fill=TEXT_MUTED)
    draw.text((bx2 + 18, b1_y + 124), "• Zero heap allocation packet-to-AST decode", font=f_mono_sm, fill=NEON_GREEN)
    draw.text((bx2 + 18, b1_y + 154), "• 289M ops/sec peak parse throughput", font=f_block_dim, fill=TEXT_DIM)

    # Block 1.3: Jump Consistent Router
    bx3 = bx2 + b1_w + 16
    draw.rounded_rectangle([(bx3, b1_y), (bx3 + b1_w, b1_y + b1_h)], radius=8, fill=BG_BLOCK_CYAN, outline=BORDER_SUBTLE, width=2)
    draw.text((bx3 + 18, b1_y + 16), "Jump Consistent Key Router", font=f_block_t, fill=TEXT_WHITE)
    draw_pill(draw, bx3 + b1_w - 110, b1_y + 14, 95, 26, "MURMUR3", f_badge, BORDER_CYAN, TEXT_WHITE)
    draw.line([(bx3 + 18, b1_y + 50), (bx3 + b1_w - 18, b1_y + 50)], fill=BORDER_MUTED, width=1)
    draw.text((bx3 + 18, b1_y + 64), "• Deterministic key-to-core partition sharding", font=f_block_sub, fill=TEXT_WHITE)
    draw.text((bx3 + 18, b1_y + 94), "• O(ln N) minimal remap on worker scale", font=f_block_sub, fill=TEXT_MUTED)
    draw.text((bx3 + 18, b1_y + 124), "• Uniform distribution (zero hot-spot cores)", font=f_mono_sm, fill=TECH_CYAN)
    draw.text((bx3 + 18, b1_y + 154), "• Dispatches commands to SPSC in ~0.05 µs", font=f_block_dim, fill=TEXT_DIM)

    # =========================================================================
    # LAYER 2: SHARED-NOTHING MULTI-CORE EXECUTION SUBSYSTEM
    # =========================================================================
    l2_y = 414
    l2_h = 320
    draw.rounded_rectangle([(cx, l2_y), (cx + content_w, l2_y + l2_h)], radius=10, fill=BG_LAYER, outline=BORDER_GREEN, width=2)

    # Layer Header Bar
    draw.text((cx + 24, l2_y + 14), "LAYER 2: SHARED-NOTHING MULTI-CORE HARDWARE WORKERS", font=f_layer_hdr, fill=NEON_GREEN)
    draw_pill(draw, cx + content_w - 410, l2_y + 12, 390, 32, "0 Mutex Contention • CPU_SET Hardware Bound", f_badge, BORDER_GREEN, TEXT_WHITE)
    draw.line([(cx + 20, l2_y + 52), (cx + content_w - 20, l2_y + 52)], fill=BORDER_MUTED, width=1)

    # 4 Pinned Worker Core Blocks
    b2_w = (content_w - 48 - 36) // 4
    b2_h = 175
    b2_y = l2_y + 64

    for i in range(4):
        bx = cx + 24 + i * (b2_w + 12)
        is_first = (i == 0)
        draw.rounded_rectangle([(bx, b2_y), (bx + b2_w, b2_y + b2_h)], radius=8, fill=BG_BLOCK_GRN if is_first else BG_BLOCK_CYAN, outline=BORDER_GREEN if is_first else BORDER_CYAN, width=2)
        draw.text((bx + 16, b2_y + 14), f"CPU Worker Core #{i}", font=f_block_t, fill=NEON_GREEN if is_first else TECH_CYAN)
        draw_pill(draw, bx + b2_w - 95, b2_y + 12, 80, 24, f"0x{1<<i:X}", f_badge, BORDER_GREEN if is_first else BORDER_CYAN, TEXT_WHITE)
        draw.line([(bx + 16, b2_y + 46), (bx + b2_w - 16, b2_y + 46)], fill=BORDER_MUTED, width=1)
        draw.text((bx + 16, b2_y + 56), "• Lock-Free 64B SPSC Ring", font=f_block_sub, fill=TEXT_WHITE)
        draw.text((bx + 16, b2_y + 84), "• Dedicated Tokio LocalSet", font=f_block_sub, fill=TEXT_MUTED)
        draw.text((bx + 16, b2_y + 112), "• Pinned to CPU Core via OS", font=f_mono_sm, fill=TECH_CYAN if is_first else NEON_GREEN)
        draw.text((bx + 16, b2_y + 140), "• Zero Cross-Core Contention", font=f_block_dim, fill=TEXT_DIM)

    # Inter-Core Communication Channel Bar at bottom of Layer 2
    ipc_y = b2_y + b2_h + 14
    ipc_h = 50
    draw.rounded_rectangle([(cx + 24, ipc_y), (cx + content_w - 24, ipc_y + ipc_h)], radius=6, fill=(16, 36, 28, 255), outline=BORDER_MUTED, width=1)
    draw.text((cx + 42, ipc_y + 14), "INTER-CORE IPC BUS:", font=f_block_t, fill=TECH_CYAN)
    draw.text((cx + 285, ipc_y + 16), "64-Byte Cache-Line Padded Ring Queues • Eliminates Multi-Core Cache False Sharing • Atomic Head/Tail Pointers", font=f_block_sub, fill=TEXT_WHITE)

    # =========================================================================
    # LAYER 3: DUAL-TIER STORAGE SUBSYSTEM (DRAM + NVMe)
    # =========================================================================
    l3_y = 756
    l3_h = 330
    draw.rounded_rectangle([(cx, l3_y), (cx + content_w, l3_y + l3_h)], radius=10, fill=BG_LAYER, outline=BORDER_AMBER, width=2)

    # Layer Header Bar
    draw.text((cx + 24, l3_y + 14), "LAYER 3: HYBRID DUAL-TIER STORAGE SUBSYSTEM", font=f_layer_hdr, fill=AMBER)
    draw_pill(draw, cx + content_w - 410, l3_y + 12, 390, 32, "< 0.2 µs SLA • 79.1% Cloud RAM Cost Reduction", f_badge, BORDER_AMBER, TEXT_WHITE)
    draw.line([(cx + 20, l3_y + 52), (cx + content_w - 20, l3_y + 52)], fill=BORDER_MUTED, width=1)

    # 2 Big Tier Blocks: Hot DRAM vs Cold NVMe
    b3_w = (content_w - 48 - 24) // 2
    b3_h = l3_h - 76
    b3_y = l3_y + 64

    # Hot Tier (DRAM)
    bx_hot = cx + 24
    draw.rounded_rectangle([(bx_hot, b3_y), (bx_hot + b3_w, b3_y + b3_h)], radius=8, fill=BG_BLOCK_GRN, outline=BORDER_GREEN, width=2)
    draw.text((bx_hot + 20, b3_y + 16), "HOT IN-MEMORY TIER (DRAM)", font=f_block_t, fill=NEON_GREEN)
    draw_pill(draw, bx_hot + b3_w - 145, b3_y + 14, 130, 26, "< 0.2 µs SLA", f_badge, BORDER_GREEN, TEXT_WHITE)
    draw.line([(bx_hot + 20, b3_y + 50), (bx_hot + b3_w - 20, b3_y + 50)], fill=BORDER_MUTED, width=1)

    hot_items = [
        ("Flat Robin Hood Hash Map", "Open addressing with Robin Hood backward shift, bounded DIB ≤ 25, 0 tombstones"),
        ("2MB Slab Arenas", "Power-of-two slab memory allocator, TZCNT instant chunk scan, zero heap fragmentation"),
        ("Small String Inlining (SSO)", "Keys ≤ 16 bytes stored directly inside pointer bits, bypassing heap allocations entirely"),
    ]
    cur_y = b3_y + 62
    for title, desc in hot_items:
        draw.text((bx_hot + 20, cur_y), f"• {title}:", font=f_block_t, fill=TEXT_WHITE)
        draw.text((bx_hot + 35, cur_y + 26), desc, font=f_block_dim, fill=TEXT_MUTED)
        cur_y += 58

    # Cold Tier (NVMe)
    bx_cold = bx_hot + b3_w + 24
    draw.rounded_rectangle([(bx_cold, b3_y), (bx_cold + b3_w, b3_y + b3_h)], radius=8, fill=BG_BLOCK_CYAN, outline=BORDER_CYAN, width=2)
    draw.text((bx_cold + 20, b3_y + 16), "COLD PERSISTENT TIER (NVMe SSD)", font=f_block_t, fill=TECH_CYAN)
    draw_pill(draw, bx_cold + b3_w - 165, b3_y + 14, 150, 26, "79.1% SAVINGS", f_badge, BORDER_CYAN, TEXT_WHITE)
    draw.line([(bx_cold + 20, b3_y + 50), (bx_cold + b3_w - 20, b3_y + 50)], fill=BORDER_MUTED, width=1)

    cold_items = [
        ("2-Bit CLOCK Frequency Hand", "Background sweeper tracks key access frequency; demotes dormant cold keys to NVMe"),
        ("4KB Block Direct I/O Paging", "Raw Linux block paging with O_DIRECT and io_uring; eliminates OS kernel page cache tax"),
        ("Compact 12B DiskPointer", "Only 12B retained in DRAM per cold key; transparent instant promotion on cache read miss"),
    ]
    cur_y = b3_y + 62
    for title, desc in cold_items:
        draw.text((bx_cold + 20, cur_y), f"• {title}:", font=f_block_t, fill=TEXT_WHITE)
        draw.text((bx_cold + 35, cur_y + 26), desc, font=f_block_dim, fill=TEXT_MUTED)
        cur_y += 58

    # =========================================================================
    # LAYER 4: FAULT TOLERANCE & DURABILITY SUBSYSTEM
    # =========================================================================
    l4_y = 1108
    l4_h = 390
    draw.rounded_rectangle([(cx, l4_y), (cx + content_w, l4_y + l4_h)], radius=10, fill=BG_LAYER, outline=BORDER_PURPLE, width=2)

    # Layer Header Bar
    draw.text((cx + 24, l4_y + 14), "LAYER 4: FAULT TOLERANCE & DURABILITY SUBSYSTEM", font=f_layer_hdr, fill=PURPLE)
    draw_pill(draw, cx + content_w - 410, l4_y + 12, 390, 32, "105/105 Passed • Zero-Fork OOM Crash Resilience", f_badge, BORDER_PURPLE, TEXT_WHITE)
    draw.line([(cx + 20, l4_y + 52), (cx + content_w - 20, l4_y + 52)], fill=BORDER_MUTED, width=1)

    # 3 Component Blocks in Layer 4
    b4_w = (content_w - 48 - 32) // 3
    b4_h = l4_h - 76
    b4_y = l4_y + 64

    # Block 4.1: Forkless Snapshotter
    bx4_1 = cx + 24
    draw.rounded_rectangle([(bx4_1, b4_y), (bx4_1 + b4_w, b4_y + b4_h)], radius=8, fill=BG_BLOCK_PURP, outline=BORDER_SUBTLE, width=2)
    draw.text((bx4_1 + 18, b4_y + 16), "Forkless Epoch Snapshotter", font=f_block_t, fill=TEXT_WHITE)
    draw_pill(draw, bx4_1 + b4_w - 120, b4_y + 14, 105, 26, "ZERO-FORK", f_badge, BORDER_PURPLE, TEXT_WHITE)
    draw.line([(bx4_1 + 18, b4_y + 50), (bx4_1 + b4_w - 18, b4_y + 50)], fill=BORDER_MUTED, width=1)
    draw.text((bx4_1 + 18, b4_y + 64), "• AtomicU64 monotonic epoch tracker", font=f_block_sub, fill=TEXT_WHITE)
    draw.text((bx4_1 + 18, b4_y + 98), "• Key-level copy-on-write shadow buffers", font=f_block_sub, fill=TEXT_MUTED)
    draw.text((bx4_1 + 18, b4_y + 132), "• Completely bypasses Linux fork() memory freeze", font=f_mono_sm, fill=PURPLE)
    draw.text((bx4_1 + 18, b4_y + 166), "• Zero OOM crash risk during heavy write spikes", font=f_block_dim, fill=TEXT_DIM)
    draw.text((bx4_1 + 18, b4_y + 196), "• Streams .val v1 snapshots continuously to disk", font=f_block_dim, fill=TEXT_MUTED)

    # Block 4.2: Segmented Binary WAL
    bx4_2 = bx4_1 + b4_w + 16
    draw.rounded_rectangle([(bx4_2, b4_y), (bx4_2 + b4_w, b4_y + b4_h)], radius=8, fill=BG_BLOCK_GRN, outline=BORDER_GREEN, width=2)
    draw.text((bx4_2 + 18, b4_y + 16), "Segmented Binary WAL", font=f_block_t, fill=NEON_GREEN)
    draw_pill(draw, bx4_2 + b4_w - 135, b4_y + 14, 120, 26, "GROUPCOMMIT", f_badge, BORDER_GREEN, TEXT_WHITE)
    draw.line([(bx4_2 + 18, b4_y + 50), (bx4_2 + b4_w - 18, b4_y + 50)], fill=BORDER_MUTED, width=1)
    draw.text((bx4_2 + 18, b4_y + 64), "• 64MB deterministic segment boundary rotation", font=f_block_sub, fill=TEXT_WHITE)
    draw.text((bx4_2 + 18, b4_y + 98), "• ECMA-182 CRC64 hardware-accelerated checksum", font=f_block_sub, fill=TEXT_MUTED)
    draw.text((bx4_2 + 18, b4_y + 132), "• GroupCommit batches concurrent client fsyncs", font=f_mono_sm, fill=NEON_GREEN)
    draw.text((bx4_2 + 18, b4_y + 166), "• Average flush latency < 48 µs to NVMe SSD", font=f_block_dim, fill=TEXT_DIM)
    draw.text((bx4_2 + 18, b4_y + 196), "• Zero-loss tail scan recovery on restart", font=f_block_dim, fill=TEXT_MUTED)

    # Block 4.3: 4-Tier Timing Wheel
    bx4_3 = bx4_2 + b4_w + 16
    draw.rounded_rectangle([(bx4_3, b4_y), (bx4_3 + b4_w, b4_y + b4_h)], radius=8, fill=BG_BLOCK_CYAN, outline=BORDER_SUBTLE, width=2)
    draw.text((bx4_3 + 18, b4_y + 16), "4-Tier Hierarchical Timing Wheel", font=f_block_t, fill=TEXT_WHITE)
    draw_pill(draw, bx4_3 + b4_w - 110, b4_y + 14, 95, 26, "< 10ns TTL", f_badge, BORDER_CYAN, TEXT_WHITE)
    draw.line([(bx4_3 + 18, b4_y + 50), (bx4_3 + b4_w - 18, b4_y + 50)], fill=BORDER_MUTED, width=1)
    draw.text((bx4_3 + 18, b4_y + 64), "• Hierarchical 4-tier timing wheel (1ms to 24h+)", font=f_block_sub, fill=TEXT_WHITE)
    draw.text((bx4_3 + 18, b4_y + 98), "• Sub-10ns O(1) atomic expired key detachment", font=f_block_sub, fill=TEXT_MUTED)
    draw.text((bx4_3 + 18, b4_y + 132), "• Eliminates Redis's blocking active-expire cycles", font=f_mono_sm, fill=TECH_CYAN)
    draw.text((bx4_3 + 18, b4_y + 166), "• Zero server stalls during mass key expiration", font=f_block_dim, fill=TEXT_DIM)
    draw.text((bx4_3 + 18, b4_y + 196), "• Deterministic sub-microsecond latency guarantee", font=f_block_dim, fill=TEXT_MUTED)

    # Bottom Caption
    draw.text((36, h - 36), "Figure 1: Valence Modular System Architecture — Layered Subsystems & Hardware Memory Layout (Pure Rust Engine).", font=f_caption, fill=TEXT_MUTED)

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"Layered Subsystems Architecture generated: {output_path}")

if __name__ == "__main__":
    create_layered_subsystems_architecture()
