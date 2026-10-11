import os
from PIL import Image, ImageDraw, ImageFont

def generate_architecture_diagram(output_path="hackathon_slides/architecture_diagram.png"):
    width, height = 2400, 1400
    img = Image.new("RGBA", (width, height), (7, 16, 11, 255))
    draw = ImageDraw.Draw(img)

    # Load Windows standard fonts
    font_path_bold = "C:/Windows/Fonts/segoeuib.ttf"
    font_path_reg = "C:/Windows/Fonts/segoeui.ttf"
    font_path_mono = "C:/Windows/Fonts/consola.ttf"

    def get_font(path, size):
        try:
            return ImageFont.truetype(path, size)
        except:
            return ImageFont.load_default()

    font_title = get_font(font_path_bold, 54)
    font_subtitle = get_font(font_path_reg, 26)
    font_h1 = get_font(font_path_bold, 30)
    font_h2 = get_font(font_path_bold, 24)
    font_body = get_font(font_path_reg, 21)
    font_mono = get_font(font_path_mono, 20)
    font_badge = get_font(font_path_bold, 18)

    # Color Palette matching Code Apex
    NEON_GREEN   = (34, 197, 94, 255)    # #22C55E
    TECH_CYAN    = (6, 182, 212, 255)    # #06B6D4
    BORDER_GREEN = (22, 101, 52, 255)    # #166534
    BORDER_CYAN  = (14, 116, 144, 255)   # #0E7490
    BG_CARD      = (12, 24, 18, 255)     # Deep emerald card
    BG_ELEVATED  = (18, 36, 28, 255)     # Lighter card
    TEXT_WHITE   = (248, 250, 252, 255)
    TEXT_MUTED   = (148, 163, 184, 255)
    ARROW_COLOR  = (34, 197, 94, 220)
    LINE_CYAN    = (6, 182, 212, 200)

    # 1. Subtle Background Grid
    for x in range(0, width, 60):
        draw.line([(x, 0), (x, height)], fill=(12, 28, 20, 100), width=1)
    for y in range(0, height, 60):
        draw.line([(0, y), (width, y)], fill=(12, 28, 20, 100), width=1)

    # Outer HUD Border
    draw.rounded_rectangle([(30, 30), (width - 30, height - 30)], radius=20, outline=BORDER_GREEN, width=2)

    # 2. Header
    draw.text((70, 60), "VALENCE: BARE-METAL SYSTEMS ARCHITECTURE", font=font_title, fill=NEON_GREEN)
    draw.text((72, 130), "End-to-End Query & Storage Pipeline  •  Sub-Microsecond Latency  •  Zero Mutex Contention", font=font_subtitle, fill=TEXT_MUTED)

    # Helper function for drawing rounded cards with title badge
    def draw_box(x, y, w, h, title, subtitle, bullets, badge=None, border_color=BORDER_GREEN, bg_color=BG_CARD):
        # Glow border
        draw.rounded_rectangle([(x, y), (x + w, y + h)], radius=14, fill=bg_color, outline=border_color, width=2)
        
        # Title
        draw.text((x + 24, y + 20), title, font=font_h1, fill=NEON_GREEN if border_color==BORDER_GREEN else TECH_CYAN)
        if subtitle:
            draw.text((x + 24, y + 60), subtitle, font=font_mono, fill=TEXT_MUTED)
        
        # Badge on top right
        if badge:
            bw, bh = 220, 36
            bx = x + w - bw - 20
            by = y + 20
            draw.rounded_rectangle([(bx, by), (bx + bw, by + bh)], radius=6, fill=BORDER_GREEN if border_color==BORDER_GREEN else BORDER_CYAN)
            draw.text((bx + 14, by + 6), badge, font=font_badge, fill=TEXT_WHITE)

        # Bullets
        cur_y = y + 105
        for b in bullets:
            draw.text((x + 24, cur_y), "▸ " + b[0], font=font_h2, fill=TEXT_WHITE)
            cur_y += 32
            draw.text((x + 44, cur_y), b[1], font=font_body, fill=TEXT_MUTED)
            cur_y += 38

    # Helper for horizontal pipeline arrows
    def draw_arrow(x1, y1, x2, y2, label="", color=ARROW_COLOR):
        draw.line([(x1, y1), (x2, y2)], fill=color, width=4)
        # arrowhead
        draw.polygon([(x2, y2), (x2 - 14, y2 - 8), (x2 - 14, y2 + 8)], fill=color)
        if label:
            draw.text((x1 + 10, y1 - 28), label, font=font_mono, fill=TECH_CYAN)

    def draw_v_arrow(x, y1, y2, label="", color=ARROW_COLOR):
        draw.line([(x, y1), (x, y2)], fill=color, width=4)
        draw.polygon([(x, y2), (x - 8, y2 - 14), (x + 8, y2 - 14)], fill=color)
        if label:
            draw.text((x + 12, (y1 + y2)//2 - 12), label, font=font_mono, fill=TECH_CYAN)

    # =========================================================================
    # ROW 1: INGRESS & CONCURRENCY (3 Boxes across)
    # =========================================================================
    y_row1 = 200
    h_row1 = 360

    # Box 1: Client Ingress
    draw_box(
        70, y_row1, 680, h_row1,
        "1. CLIENT & WIRE INGRESS",
        "TCP Port 6379  •  RESP3 Protocol",
        [
            ("Drop-In Redis Protocol Compatibility", "Works with existing redis-cli, Jedis, redis-py, ioredis."),
            ("Zero-Copy Memory Stream", "Tokio non-blocking TCP socket directly maps network frames."),
            ("Ingress Overhead", "Zero serialization tax • Raw TCP pipelining enabled.")
        ],
        badge="COMPATIBILITY: 100%"
    )

    draw_arrow(750, y_row1 + 180, 830, y_row1 + 180, label="Raw Wire")

    # Box 2: AVX2 SIMD Scanner
    draw_box(
        830, y_row1, 720, h_row1,
        "2. AVX2 VECTOR PARSER",
        "src/protocol/simd.rs  •  256-bit Vector Unit",
        [
            ("256-Bit Vectorized Delimiter Discovery", "_mm256_cmpeq_epi8 scans 32 bytes per clock cycle."),
            ("Bit-Manipulation Trailing Zero Count", "_tzcnt_u32 locates exact CRLF offsets in single instruction."),
            ("Benchmarked Ingress Performance", "289,477,493 ops/sec  •  Parser Latency: ~0.08 µs.")
        ],
        badge="LATENCY: 0.08 µs",
        border_color=BORDER_CYAN
    )

    draw_arrow(1550, y_row1 + 180, 1630, y_row1 + 180, label="Fast Route")

    # Box 3: Concurrency & Sharding
    draw_box(
        1630, y_row1, 700, h_row1,
        "3. THREAD-PER-CORE SHARDS",
        "src/sharding/  •  Hardware Core Pinning",
        [
            ("Jump Consistent Hash Partitioning", "Murmur3-32 maps keys to shards in O(ln N) with 0 memory."),
            ("Lock-Free 64B SPSC Ring Buffers", "Cache-line padded queues eliminate mutex locks & false sharing."),
            ("Native OS CPU Affinity", "SetThreadAffinityMask (Win) & sched_setaffinity (Linux).")
        ],
        badge="MUTEX LOCKS: 0"
    )

    # Arrow Down to Storage Layer
    draw_v_arrow(1980, y_row1 + h_row1, y_row1 + h_row1 + 70, label="Core Dispatch (~0.15 µs)")

    # =========================================================================
    # ROW 2: INDEXING & DUAL-TIER STORAGE
    # =========================================================================
    y_row2 = 640
    h_row2 = 360

    # Box 4: Flat Robin Hood Hash Map
    draw_box(
        70, y_row2, 680, h_row2,
        "4. ROBIN HOOD STORAGE INDEX",
        "src/storage/robin_hood.rs  •  Flat Array",
        [
            ("Distance-from-Ideal-Bucket (DIB)", "Bounds probe sequence lengths; lookup finishes in 1-2 hops."),
            ("Zero-Tombstone Backward-Shift Delete", "Deletions shift elements backward, eliminating dead slots."),
            ("Lookup Performance SLA", "Zero pointer chasing  •  Lookup Latency: ~0.22 µs.")
        ],
        badge="TOMBSTONES: 0"
    )

    draw_arrow(1630, y_row2 + 180, 1550, y_row2 + 180, label="Data Pointer", color=LINE_CYAN)

    # Box 5: Dual Storage Tier (HOT DRAM vs COLD NVMe)
    draw_box(
        830, y_row2, 1500, h_row2,
        "5. DUAL-TIER MEMORY HIERARCHY & TRANSPARENT NVMe PAGING",
        "memory/slab_arena.rs  &  tiering/io_uring.rs",
        [
            ("HOT TIER: 2MB Hugepage Bitmapped Slab Arenas",
             "Allocations via TZCNT in 1.19ns  •  Anti-fragmentation guarantee: RSS / Logical < 1.02."),
            ("COLD TIER: 2-Bit CLOCK NVMe Tiering (79.1% DRAM Savings)",
             "Sweeps dormant keys to 4KB aligned NVMe blocks  •  Replaced in DRAM with compact 12-byte DiskPointer."),
            ("On-Demand Cold Promotion & SSO Inlining",
             "Keys <= 23B inlined with 0B heap allocation  •  Cold keys promote back to RAM on read in microseconds.")
        ],
        badge="DRAM REDUCTION: 79.1%",
        border_color=BORDER_CYAN
    )

    # Arrow from Robin Hood to Storage Tier
    draw_arrow(750, y_row2 + 180, 830, y_row2 + 180, label="Index Fetch")

    # =========================================================================
    # ROW 3: TIMING WHEEL & DURABILITY
    # =========================================================================
    y_row3 = 1060
    h_row3 = 260

    # Box 6: 4-Tier Timing Wheel
    draw_box(
        70, y_row3, 1100, h_row3,
        "6. 4-TIER HIERARCHICAL TIMING WHEEL",
        "src/timer/timing_wheel.rs  •  O(1) TTL Expiration",
        [
            ("Hierarchical Cascaded Wheels: 10ms, 1s, 1min, 1hr",
             "Keys stored in doubly-linked circular slots  •  Atomic O(1) detach on key update."),
            ("Immune to TTL Avalanches",
             "Eliminates priority-queue O(log N) latency freezes when millions of keys expire at once.")
        ],
        badge="LATENCY: FLAT O(1)"
    )

    # Box 7: Forkless Durability & WAL
    draw_box(
        1230, y_row3, 1100, h_row3,
        "7. ZERO-FORK EPOCH SNAPSHOTS & CRC64 WAL",
        "src/snapshot/  &  src/wal/  •  ACID Crash Resilience",
        [
            ("Streaming Snapshots without Linux fork()",
             "Mutation epochs & key-level shadow buffers avoid memory doubling and OOM crashes."),
            ("64MB Segmented Binary Write-Ahead Log",
             "GroupCommit fsync batching  •  ECMA-182 CRC64  •  Torn-write tail recovery.")
        ],
        badge="MEMORY OVERHEAD: 0B",
        border_color=BORDER_CYAN
    )

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    img.save(output_path, "PNG")
    print(f"[SUCCESS] Architecture diagram generated at {output_path} ({width}x{height})")

if __name__ == "__main__":
    generate_architecture_diagram()
