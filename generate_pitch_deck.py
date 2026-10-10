import sys
from pptx import Presentation
from pptx.util import Inches, Pt
from pptx.enum.text import PP_ALIGN
from pptx.dml.color import RGBColor
from pptx.enum.shapes import MSO_SHAPE

def create_deck(output_path="Valence_CodeApex_Presentation.pptx"):
    prs = Presentation()
    prs.slide_width = Inches(13.333)
    prs.slide_height = Inches(7.5)

    blank_layout = prs.slide_layouts[6]

    # Colors
    BG_DARK = RGBColor(7, 16, 11)        # Cyber Dark Green/Black
    CARD_BG = RGBColor(14, 26, 20)       # Dark Emerald Card
    ACCENT_GREEN = RGBColor(34, 197, 94) # Neon Green #22C55E
    ACCENT_CYAN = RGBColor(6, 182, 212)  # Tech Cyan #06B6D4
    TEXT_WHITE = RGBColor(248, 250, 252)
    TEXT_MUTED = RGBColor(148, 163, 184)
    TEXT_DIM = RGBColor(100, 116, 139)
    BORDER_GREEN = RGBColor(22, 101, 52) # Dark Green Border

    def add_slide_background(slide):
        bg = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, 0, 0, Inches(13.333), Inches(7.5))
        bg.fill.solid()
        bg.fill.fore_color.rgb = BG_DARK
        bg.line.fill.background()
        return bg

    def add_header(slide, title_text, subtitle_text=None):
        # Badge / Top bar
        tx_box = slide.shapes.add_textbox(Inches(0.8), Inches(0.5), Inches(11.733), Inches(1.2))
        tf = tx_box.text_frame
        tf.word_wrap = True
        tf.margin_left = tf.margin_top = tf.margin_right = tf.margin_bottom = 0
        
        p = tf.paragraphs[0]
        p.text = title_text.upper()
        p.font.name = "Arial Black"
        p.font.size = Pt(28)
        p.font.bold = True
        p.font.color.rgb = ACCENT_GREEN

        if subtitle_text:
            p2 = tf.add_paragraph()
            p2.text = subtitle_text.upper()
            p2.font.name = "Arial"
            p2.font.size = Pt(13)
            p2.font.bold = True
            p2.font.color.rgb = TEXT_MUTED
            p2.space_before = Pt(6)

    def add_card(slide, left, top, width, height, title, content_list, title_color=ACCENT_GREEN):
        card = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left, top, width, height)
        card.fill.solid()
        card.fill.fore_color.rgb = CARD_BG
        card.line.color.rgb = BORDER_GREEN
        card.line.width = Pt(1.5)

        tb = slide.shapes.add_textbox(left + Inches(0.25), top + Inches(0.2), width - Inches(0.5), height - Inches(0.4))
        tf = tb.text_frame
        tf.word_wrap = True
        tf.margin_left = tf.margin_top = tf.margin_right = tf.margin_bottom = 0

        p = tf.paragraphs[0]
        p.text = title
        p.font.name = "Arial Black"
        p.font.size = Pt(16)
        p.font.bold = True
        p.font.color.rgb = title_color

        for item in content_list:
            p_item = tf.add_paragraph()
            p_item.text = f"• {item}" if not item.startswith("  ") else f"    {item.strip()}"
            p_item.font.name = "Arial"
            p_item.font.size = Pt(13)
            p_item.font.color.rgb = TEXT_WHITE if not item.startswith("  ") else TEXT_MUTED
            p_item.space_before = Pt(6)

    # ==========================================
    # SLIDE 1: TITLE SLIDE
    # ==========================================
    s1 = prs.slides.add_slide(blank_layout)
    add_slide_background(s1)

    # Event Header
    t_ev = s1.shapes.add_textbox(Inches(0.8), Inches(1.0), Inches(11.733), Inches(0.8))
    p_ev = t_ev.text_frame.paragraphs[0]
    p_ev.text = "CODE APEX  |  THE END IS COMING  •  POWERED BY verzii"
    p_ev.font.name = "Arial"
    p_ev.font.size = Pt(14)
    p_ev.font.bold = True
    p_ev.font.color.rgb = ACCENT_GREEN

    # Project Title
    t_title = s1.shapes.add_textbox(Inches(0.8), Inches(1.8), Inches(11.733), Inches(2.2))
    tf_title = t_title.text_frame
    p_main = tf_title.paragraphs[0]
    p_main.text = "VALENCE"
    p_main.font.name = "Arial Black"
    p_main.font.size = Pt(56)
    p_main.font.bold = True
    p_main.font.color.rgb = ACCENT_GREEN

    p_sub = tf_title.add_paragraph()
    p_sub.text = "Next-Generation In-Memory State Engine in Pure Rust"
    p_sub.font.name = "Arial"
    p_sub.font.size = Pt(22)
    p_sub.font.bold = True
    p_sub.font.color.rgb = TEXT_WHITE
    p_sub.space_before = Pt(8)

    # Team Box (Matching template)
    add_card(
        s1,
        Inches(0.8), Inches(4.3), Inches(11.733), Inches(2.4),
        "TEAM DETAILS",
        [
            "TEAM NAME:  Team Valence Systems",
            "TEAM LEADER NAME:  [Your Name]",
            "PROBLEM STATEMENT (DOMAIN):  Cloud Infrastructure, Systems Engineering & High-Performance Data Engines",
            "CORE VALUE PROP:  Sub-microsecond state cache solving Redis single-thread saturation, DRAM cost explosion (79% NVMe cut), and fork() crash hazards."
        ],
        ACCENT_CYAN
    )

    # ==========================================
    # SLIDE 2: PROBLEM STATEMENT & TARGET NEED
    # ==========================================
    s2 = prs.slides.add_slide(blank_layout)
    add_slide_background(s2)
    add_header(s2, "Problem Statement & Target Need", "Why modern cloud architectures outgrew legacy in-memory stores")

    # 3 Column Cards
    col_w = Inches(3.64)
    gap = Inches(0.4)
    top_pos = Inches(1.9)
    card_h = Inches(4.9)

    add_card(
        s2, Inches(0.8), top_pos, col_w, card_h,
        "WHO IS AFFECTED",
        [
            "Cloud-scale enterprises, fintech, real-time gaming, and LLM inference platforms.",
            "Companies paying massive monthly AWS ElastiCache / Redis bills for underutilized memory.",
            "Site Reliability Engineers (SREs) battling random latency spikes and Out-Of-Memory (OOM) kills during database snapshots."
        ]
    )

    add_card(
        s2, Inches(0.8) + col_w + gap, top_pos, col_w, card_h,
        "THE CURRENT GAP",
        [
            "Single-Thread CPU Waste: Modern cloud instances offer 32–64 cores; legacy Redis utilizes only 1 core for commands.",
            "DRAM Cost Explosion: 80% of cached keys are 'cold' (untouched for days), yet forced to occupy expensive RAM.",
            "Dangerous fork() Snapshots: Redis BGSAVE copies page tables via Linux fork(), causing 2x memory spikes and latency freezes."
        ],
        ACCENT_CYAN
    )

    add_card(
        s2, Inches(0.8) + (col_w + gap)*2, top_pos, col_w, card_h,
        "WHY IT MATTERS NOW",
        [
            "AI & Real-Time Data Surge: Vector search, LLM context caching, and high-frequency transactions demand sub-microsecond SLAs.",
            "Cloud Budget Scrutiny: Cloud infrastructure budgets are tightening—slashing DRAM costs by 70–80% yields millions in cloud savings.",
            "Hardware Shift: NVMe SSDs now deliver microsecond I/O; databases must exploit hardware-level NVMe tiering and SIMD vector units."
        ]
    )

    # ==========================================
    # SLIDE 3: PROPOSED SOLUTION & INNOVATION
    # ==========================================
    s3 = prs.slides.add_slide(blank_layout)
    add_slide_background(s3)
    add_header(s3, "Proposed Solution & Innovation", "Bare-metal Rust systems architecture engineered for modern hardware")

    add_card(
        s3, Inches(0.8), top_pos, col_w, card_h,
        "WHAT MAKES IT DIFFERENT",
        [
            "100% Pure Rust, Zero-C Dependencies: Complete memory safety without garbage collector latency jitter.",
            "Universal RESP3 Compatibility: Drop-in replacement for redis-cli, Jedis, redis-py, and ioredis.",
            "Zero-Fork Streaming Snapshots: Guarantees zero memory doubling and zero CPU latency freezes during persistence.",
            "AVX2 Vector Acceleration: Scans network byte delimiters at 289M ops/sec using 256-bit SIMD intrinsics."
        ]
    )

    add_card(
        s3, Inches(0.8) + col_w + gap, top_pos, col_w, card_h,
        "CORE MECHANISM",
        [
            "Thread-per-Core Shards: CPU core-pinned workers communicating via lock-free 64B cache-padded SPSC ring buffers.",
            "Robin Hood Hash Table: Flat array index with Distance-from-Ideal-Bucket (DIB) and backward-shift zero-tombstone deletion.",
            "2-Bit CLOCK NVMe Tiering: Sweeps dormant keys to 4KB aligned NVMe blocks, replacing payloads with 12B DiskPointers (79.1% DRAM savings).",
            "4-Tier Timing Wheel: O(1) atomic detach timer wheels immune to TTL expiration avalanches."
        ],
        ACCENT_CYAN
    )

    add_card(
        s3, Inches(0.8) + (col_w + gap)*2, top_pos, col_w, card_h,
        "WHY FEASIBLE NOW",
        [
            "100% Built & Verified Today: Not a mock concept—105 out of 105 automated unit and chaos test suites passing.",
            "Native Windows & Linux Affinity: Production support using SetThreadAffinityMask and sched_setaffinity.",
            "Bitmapped Slab Arenas: 64-bit integer bitmapped allocators achieve 1.19ns allocations with zero OS heap fragmentation (RSS/Logical < 1.02)."
        ]
    )

    # ==========================================
    # SLIDE 4: SYSTEM ARCHITECTURE & TECH STACK
    # ==========================================
    s4 = prs.slides.add_slide(blank_layout)
    add_slide_background(s4)
    add_header(s4, "System Architecture & Tech Stack", "Full-stack bare-metal engine paired with tier-1 enterprise telemetry")

    tech_w = Inches(11.733)
    box_h = Inches(0.85)
    gap_y = Inches(0.16)
    start_y = Inches(1.9)

    layers = [
        ("FRONTEND", "React 19, TypeScript, Vite, Vanilla CSS Design System, Lucide Icons, Canvas Visualizers", ACCENT_GREEN),
        ("BACKEND ENGINE", "Pure Rust (2021 Edition), Tokio Asynchronous Runtime, Crossbeam Ring Buffers, AVX2 SIMD Intrinsics", ACCENT_CYAN),
        ("DATABASE / STORAGE", "Custom Flat Robin Hood Map, Bitmapped Slab Allocator (2MB THP), NVMe Block Storage (4KB Pages)", ACCENT_GREEN),
        ("DIAGNOSTICS & APIS", "RESP3 Wire Protocol, Diagnostic CLI Engine (valence-cli --explain), Criterion Benchmark Suite", ACCENT_CYAN),
        ("HOSTING / DEVOPS", "Windows & Linux Native Target, Multi-threaded Core Pinning, GitHub Actions CI/CD Pipeline", ACCENT_GREEN),
    ]

    for idx, (lbl, desc, col) in enumerate(layers):
        y = start_y + idx * (box_h + gap_y)
        sh = s4.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, Inches(0.8), y, tech_w, box_h)
        sh.fill.solid()
        sh.fill.fore_color.rgb = CARD_BG
        sh.line.color.rgb = BORDER_GREEN
        sh.line.width = Pt(1.5)

        tb = s4.shapes.add_textbox(Inches(1.0), y + Inches(0.12), tech_w - Inches(0.4), box_h - Inches(0.24))
        tf = tb.text_frame
        tf.word_wrap = True
        p = tf.paragraphs[0]
        p.text = f"{lbl}  ──  "
        p.font.name = "Arial Black"
        p.font.size = Pt(14)
        p.font.bold = True
        p.font.color.rgb = col

        run = p.add_run()
        run.text = desc
        run.font.name = "Arial"
        run.font.size = Pt(13)
        run.font.color.rgb = TEXT_WHITE

    # ==========================================
    # SLIDE 5: KEY FEATURES & UI/UX
    # ==========================================
    s5 = prs.slides.add_slide(blank_layout)
    add_slide_background(s5)
    add_header(s5, "Key Features & UI/UX", "Valence Studio: Enterprise console inspired by RedisInsight and Datadog")

    card2_w = Inches(5.66)
    card2_gap = Inches(0.4)
    card2_h = Inches(2.35)

    add_card(
        s5, Inches(0.8), top_pos, card2_w, card2_h,
        "1. KEYSPACE BROWSER (CRUD & HEX)",
        [
            "Interactive object inspector with namespace grouping (user:*, auth:*).",
            "Payload viewers for Raw text, Formatted JSON, and Binary Hex Dumps.",
            "Live TTL countdown and per-key memory consumption in bytes."
        ]
    )

    add_card(
        s5, Inches(0.8) + card2_w + card2_gap, top_pos, card2_w, card2_h,
        "2. CLUSTER TOPOLOGY & MEMORY",
        [
            "Thread-per-core hardware affinity map showing pinned worker threads.",
            "Lock-free SPSC IPC ring buffer queue depths and zero false sharing.",
            "Bitmapped Slab Allocator fragmentation metrics (RSS / Logical < 1.02)."
        ],
        ACCENT_CYAN
    )

    add_card(
        s5, Inches(0.8), top_pos + card2_h + Inches(0.25), card2_w, card2_h,
        "3. NVMe TIERING & 2-BIT CLOCK",
        [
            "Visual 2-bit CLOCK frequency tracker sweeping hot vs. cold keys.",
            "Automatic demotion to NVMe 4KB blocks with 12-byte DiskPointers.",
            "Live cache promotion simulation restoring cold keys to RAM on demand."
        ],
        ACCENT_CYAN
    )

    add_card(
        s5, Inches(0.8) + card2_w + card2_gap, top_pos + card2_h + Inches(0.25), card2_w, card2_h,
        "4. QUERY PLAN TRACE & APM PROFILER",
        [
            "Colored query execution tree (--explain) tracing SIMD -> Shard -> DIB.",
            "Live APM telemetry: QPS, Cache Hit Rate (99.4%), and sub-10µs P99 latency.",
            "Embedded RESP3 Workbench terminal for live command testing."
        ]
    )

    # ==========================================
    # SLIDE 6: IMPLEMENTATION ROADMAP & SCOPE
    # ==========================================
    s6 = prs.slides.add_slide(blank_layout)
    add_slide_background(s6)
    add_header(s6, "Implementation Roadmap & Scope", "What's built today, what's in progress, and what's planned beyond the hackathon")

    add_card(
        s6, Inches(0.8), top_pos, col_w, card_h,
        "WHAT'S BUILT TODAY",
        [
            "100% of Core Engine (Phases 1-8):",
            "  • Flat Robin Hood Map & RESP3 parser",
            "  • 4-Tier Timing Wheel (O(1) detach)",
            "  • Thread-per-Core Shards & SPSC Rings",
            "  • 64-bit Bitmapped Slab Allocators",
            "  • Zero-Fork Snapshots & CRC64 WAL",
            "  • 2-Bit CLOCK NVMe Tiering (79.1% cut)",
            "  • AVX2 SIMD Scanner (289M ops/s)",
            "  • 105 / 105 automated tests passing",
            "  • Valence Studio web management console"
        ]
    )

    add_card(
        s6, Inches(0.8) + col_w + gap, top_pos, col_w, card_h,
        "WHAT'S IN PROGRESS",
        [
            "Multi-Node Distributed Clustering:",
            "  • Cross-node Raft consensus replication",
            "  • Automatic shard rebalancing over gRPC",
            "Advanced Storage Backends:",
            "  • Linux io_uring kernel submission rings for direct NVMe IOPS boost",
            "Client SDK Wrappers:",
            "  • High-throughput pipelined Rust and Python native client drivers"
        ],
        ACCENT_CYAN
    )

    add_card(
        s6, Inches(0.8) + (col_w + gap)*2, top_pos, col_w, card_h,
        "BEYOND THE HACKATHON",
        [
            "Serverless Elastic Valence:",
            "  • Auto-scaling cloud tier with S3 cold object storage spillover.",
            "Active-Active Multi-Region Replication:",
            "  • Conflict-free replicated data types (CRDTs) across global data centers.",
            "Enterprise Managed Cloud SaaS:",
            "  • Fully managed cloud DB offering undercutting AWS ElastiCache by 60%."
        ]
    )

    # ==========================================
    # SLIDE 7: EXPECTED IMPACT & SCALABILITY
    # ==========================================
    s7 = prs.slides.add_slide(blank_layout)
    add_slide_background(s7)
    add_header(s7, "Expected Impact & Scalability", "Who benefits, how big the opportunity is, and how this grows beyond a demo")

    add_card(
        s7, Inches(0.8), top_pos, col_w, card_h,
        "WHO BENEFITS",
        [
            "Engineering Teams & Startups: Drastically cut cloud caching bills without modifying existing Redis application code.",
            "High-Frequency Platforms: Fintech, e-commerce, and gaming achieve reliable sub-microsecond latency without jitter.",
            "SRE & Platform Teams: Sleep peacefully knowing forkless snapshots and torn-write WAL resilience prevent data loss."
        ]
    )

    add_card(
        s7, Inches(0.8) + col_w + gap, top_pos, col_w, card_h,
        "THE MARKET OPPORTUNITY",
        [
            "$15B+ In-Memory Database Market: Dominated by Redis, AWS MemoryDB, and Aerospike.",
            "70-80% Infrastructure Cost Reduction: Moving cold keys to NVMe drives saves enterprise clusters tens of thousands of dollars per month.",
            "Linear CPU Core Scaling: Exploits 64-core AMD EPYC & Intel Xeon servers that legacy single-threaded engines waste."
        ],
        ACCENT_CYAN
    )

    add_card(
        s7, Inches(0.8) + (col_w + gap)*2, top_pos, col_w, card_h,
        "GROWTH BEYOND A DEMO",
        [
            "Open-Source Community Adoption: Releasing Valence core under Apache 2.0 / MIT licenses to attract systems contributors.",
            "Production Hardening: Comprehensive Jepsen testing for partition tolerance and fault injection resilience.",
            "Commercial SaaS Tier: Offering automated backups, compliance encryption, and global clustering as an enterprise tier."
        ]
    )

    # ==========================================
    # SLIDE 8: THANK YOU & LIVE DEMO
    # ==========================================
    s8 = prs.slides.add_slide(blank_layout)
    add_slide_background(s8)

    t_thk = s8.shapes.add_textbox(Inches(0.8), Inches(1.2), Inches(11.733), Inches(2.2))
    p_thk = t_thk.text_frame.paragraphs[0]
    p_thk.text = "THANK YOU"
    p_thk.font.name = "Arial Black"
    p_thk.font.size = Pt(64)
    p_thk.font.bold = True
    p_thk.font.color.rgb = ACCENT_GREEN

    p_thk_sub = t_thk.text_frame.add_paragraph()
    p_thk_sub.text = "CODE APEX  |  BUILD BEFORE IT DOES.  •  POWERED BY verzii"
    p_thk_sub.font.name = "Arial"
    p_thk_sub.font.size = Pt(16)
    p_thk_sub.font.bold = True
    p_thk_sub.font.color.rgb = TEXT_WHITE
    p_thk_sub.space_before = Pt(8)

    add_card(
        s8, Inches(0.8), Inches(3.8), Inches(11.733), Inches(2.9),
        "LIVE DEMONSTRATION & PROJECT LINKS",
        [
            "Live Web Management Console:  http://localhost:5173/ (Valence Studio)",
            "Automated Verification Suite:  105 / 105 Tests Passing (cargo test --release)",
            "Diagnostic CLI Engine:  target\\release\\valence-cli.exe --explain",
            "1-Click Interactive Demo:  start_demo.bat (Windows) & start_demo.sh (Linux/macOS)",
            "GitHub Repository:  github.com/[your-handle]/valence (Pure Rust In-Memory State Engine)",
            "Ready for Judge Q&A and Live Terminal Benchmarking!"
        ],
        ACCENT_CYAN
    )

    prs.save(output_path)
    print(f"[SUCCESS] Presentation generated successfully: {output_path}")

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "Valence_CodeApex_Presentation.pptx"
    create_deck(out)
