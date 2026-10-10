import os
from pptx import Presentation
from pptx.util import Inches, Pt
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.enum.shapes import MSO_SHAPE

def build_official_deck():
    prs = Presentation()
    # 16:9 Widescreen standard
    prs.slide_width = Inches(13.333)
    prs.slide_height = Inches(7.5)
    blank_layout = prs.slide_layouts[6]

    # Theme Colors matching Code Apex Cyber-Green
    NEON_GREEN = RGBColor(34, 197, 94)    # #22C55E
    TECH_CYAN  = RGBColor(6, 182, 212)    # #06B6D4
    TEXT_WHITE = RGBColor(248, 250, 252)  # #F8FAFC
    TEXT_MUTED = RGBColor(148, 163, 184)  # #94A3B8
    TEXT_DIM   = RGBColor(100, 116, 139)  # #64748B
    CARD_BG    = RGBColor(11, 23, 18)     # Dark Emerald Tint
    CARD_BORDER= RGBColor(22, 101, 52)    # Emerald Border

    def set_slide_background(slide, image_path):
        slide.shapes.add_picture(image_path, 0, 0, width=Inches(13.333), height=Inches(7.5))

    # =========================================================================
    # SLIDE 1: TITLE & TEAM DETAILS
    # =========================================================================
    s1 = prs.slides.add_slide(blank_layout)
    set_slide_background(s1, "hackathon_slides/slide_1.png")

    # Project Title & Subtitle right above or inside the frame
    tb_title = s1.shapes.add_textbox(Inches(0.8), Inches(2.2), Inches(11.733), Inches(1.2))
    tf_title = tb_title.text_frame
    tf_title.word_wrap = True
    p_t = tf_title.paragraphs[0]
    p_t.text = "VALENCE: NEXT-GEN IN-MEMORY STATE ENGINE"
    p_t.font.name = "Arial Black"
    p_t.font.size = Pt(24)
    p_t.font.bold = True
    p_t.font.color.rgb = TECH_CYAN
    p_t.alignment = PP_ALIGN.CENTER

    p_t_sub = tf_title.add_paragraph()
    p_t_sub.text = "Sub-Microsecond Pure Rust Cache with Transparent NVMe Tiering & Zero-Fork Snapshots"
    p_t_sub.font.name = "Arial"
    p_t_sub.font.size = Pt(14)
    p_t_sub.font.color.rgb = TEXT_WHITE
    p_t_sub.alignment = PP_ALIGN.CENTER
    p_t_sub.space_before = Pt(4)

    # Values for Team Details (placed to align with template labels)
    tb_team = s1.shapes.add_textbox(Inches(3.3), Inches(5.15), Inches(8.5), Inches(1.6))
    tf_team = tb_team.text_frame
    tf_team.word_wrap = True

    p_name = tf_team.paragraphs[0]
    p_name.text = "Team Valence Systems"
    p_name.font.name = "Arial"
    p_name.font.size = Pt(15)
    p_name.font.bold = True
    p_name.font.color.rgb = TEXT_WHITE

    p_lead = tf_team.add_paragraph()
    p_lead.text = "Akash Ray & Team"
    p_lead.font.name = "Arial"
    p_lead.font.size = Pt(15)
    p_lead.font.bold = True
    p_lead.font.color.rgb = TEXT_WHITE
    p_lead.space_before = Pt(9)

    p_dom = tf_team.add_paragraph()
    p_dom.text = "Cloud Infrastructure, Systems Engineering & Distributed Data Engines"
    p_dom.font.name = "Arial"
    p_dom.font.size = Pt(15)
    p_dom.font.bold = True
    p_dom.font.color.rgb = TECH_CYAN
    p_dom.space_before = Pt(9)

    # =========================================================================
    # SLIDE 2: PROBLEM STATEMENT & TARGET NEED
    # =========================================================================
    s2 = prs.slides.add_slide(blank_layout)
    set_slide_background(s2, "hackathon_slides/slide_2.png")

    # Section 1: Who is Affected (under label at y ≈ 2.7)
    tb_p1 = s2.shapes.add_textbox(Inches(0.8), Inches(2.75), Inches(11.5), Inches(0.9))
    tf_p1 = tb_p1.text_frame
    tf_p1.word_wrap = True
    p = tf_p1.paragraphs[0]
    p.text = "• Cloud-scale enterprises, fintech, real-time gaming, and LLM inference platforms running intensive caching workloads."
    p.font.name = "Arial"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_p1.add_paragraph()
    p2.text = "• Engineering teams paying thousands per month on AWS ElastiCache / Redis for mostly idle, cold memory."
    p2.font.name = "Arial"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TEXT_MUTED
    p2.space_before = Pt(3)

    # Section 2: The Current Gap (under label at y ≈ 4.1)
    tb_p2 = s2.shapes.add_textbox(Inches(0.8), Inches(4.15), Inches(11.5), Inches(1.1))
    tf_p2 = tb_p2.text_frame
    tf_p2.word_wrap = True
    p = tf_p2.paragraphs[0]
    p.text = "• Single-Thread Core Waste: Modern servers have 32–64 cores; legacy Redis utilizes only 1 CPU core for command processing."
    p.font.name = "Arial"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_p2.add_paragraph()
    p2.text = "• DRAM Cost Inefficiency: Up to 80% of cached keys are 'cold' (untouched for days), yet forced to occupy expensive RAM."
    p2.font.name = "Arial"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TEXT_MUTED
    p2.space_before = Pt(3)
    p3 = tf_p2.add_paragraph()
    p3.text = "• Dangerous fork() Snapshots: Redis BGSAVE copies memory page tables, causing latency spikes and Out-Of-Memory (OOM) crashes."
    p3.font.name = "Arial"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_WHITE
    p3.space_before = Pt(3)

    # Section 3: Why it Matters Now (under label at y ≈ 5.65)
    tb_p3 = s2.shapes.add_textbox(Inches(0.8), Inches(5.65), Inches(11.5), Inches(1.1))
    tf_p3 = tb_p3.text_frame
    tf_p3.word_wrap = True
    p = tf_p3.paragraphs[0]
    p.text = "• AI & Real-Time Wave: Vector caches, agent sessions, and streaming pipelines require sub-microsecond latency SLAs."
    p.font.name = "Arial"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_p3.add_paragraph()
    p2.text = "• Hardware Revolution: Modern NVMe SSDs deliver microsecond I/O; databases can now tier cold data without latency penalties."
    p2.font.name = "Arial"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TEXT_MUTED
    p2.space_before = Pt(3)
    p3 = tf_p3.add_paragraph()
    p3.text = "• Cloud Budget Pressure: Cutting DRAM footprint by 70–80% directly delivers massive, tangible cloud savings for companies."
    p3.font.name = "Arial"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TECH_CYAN
    p3.space_before = Pt(3)

    # =========================================================================
    # SLIDE 3: PROPOSED SOLUTION & INNOVATION
    # =========================================================================
    s3 = prs.slides.add_slide(blank_layout)
    set_slide_background(s3, "hackathon_slides/slide_3.png")

    # Section 1: What makes it different (y ≈ 2.75)
    tb_s1 = s3.shapes.add_textbox(Inches(0.8), Inches(2.75), Inches(11.5), Inches(0.95))
    tf_s1 = tb_s1.text_frame
    tf_s1.word_wrap = True
    p = tf_s1.paragraphs[0]
    p.text = "• 100% Pure Rust, Zero-C Dependencies: Eliminates memory corruption vulnerabilities without GC latency jitter."
    p.font.name = "Arial"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_s1.add_paragraph()
    p2.text = "• Drop-in RESP3 Wire Compatibility: Speaks standard Redis protocol; works instantly with existing redis-cli, Python, Java, and Node SDKs."
    p2.font.name = "Arial"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TECH_CYAN
    p2.space_before = Pt(3)
    p3 = tf_s1.add_paragraph()
    p3.text = "• AVX2 Vector Accelerator: Scans network byte delimiters at 289M ops/sec using 256-bit SIMD intrinsics."
    p3.font.name = "Arial"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_MUTED
    p3.space_before = Pt(3)

    # Section 2: Core Mechanism (y ≈ 4.15)
    tb_s2 = s3.shapes.add_textbox(Inches(0.8), Inches(4.15), Inches(11.5), Inches(1.15))
    tf_s2 = tb_s2.text_frame
    tf_s2.word_wrap = True
    p = tf_s2.paragraphs[0]
    p.text = "• Thread-per-Core Shards: Pins threads to CPU cores with 64-byte cache-padded SPSC ring buffers (zero mutex contention)."
    p.font.name = "Arial"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_s2.add_paragraph()
    p2.text = "• 2-Bit CLOCK NVMe Tiering: Automatically pages dormant keys to 4KB aligned NVMe blocks, reducing DRAM costs by 79.1%."
    p2.font.name = "Arial"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TECH_CYAN
    p2.space_before = Pt(3)
    p3 = tf_s2.add_paragraph()
    p3.text = "• Forkless Epoch Snapshots & CRC64 WAL: Point-in-time streaming persistence without Linux fork() memory doubling."
    p3.font.name = "Arial"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_MUTED
    p3.space_before = Pt(3)

    # Section 3: Why it's feasible in this timeframe (y ≈ 5.65)
    tb_s3 = s3.shapes.add_textbox(Inches(0.8), Inches(5.65), Inches(11.5), Inches(1.1))
    tf_s3 = tb_s3.text_frame
    tf_s3.word_wrap = True
    p = tf_s3.paragraphs[0]
    p.text = "• 100% Fully Implemented & Tested Today: All 8 phases are complete with 105 out of 105 passing automated test suites."
    p.font.name = "Arial"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_s3.add_paragraph()
    p2.text = "• Bitmapped Slab Allocators: Sub-nanosecond allocations (1.19ns via TZCNT) with zero OS heap fragmentation (RSS / Logical < 1.02)."
    p2.font.name = "Arial"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TECH_CYAN
    p2.space_before = Pt(3)
    p3 = tf_s3.add_paragraph()
    p3.text = "• Live Operational Console: Real-time web management studio running live right now for immediate judging verification."
    p3.font.name = "Arial"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_MUTED
    p3.space_before = Pt(3)

    # =========================================================================
    # SLIDE 4: SYSTEM ARCHITECTURE & TECH STACK
    # =========================================================================
    s4 = prs.slides.add_slide(blank_layout)
    set_slide_background(s4, "hackathon_slides/slide_4.png")

    # The template has 5 boxes on the left:
    # 1. FRONTEND: y ≈ 2.22, h ≈ 0.65, x ≈ 0.53, w ≈ 3.7
    # 2. BACKEND:  y ≈ 3.02
    # 3. DATABASE: y ≈ 3.82
    # 4. AI/APIS:  y ≈ 4.62
    # 5. HOSTING:  y ≈ 5.42
    stack_content = [
        (Inches(0.55), Inches(2.28), Inches(3.6), Inches(0.58), "React 19, TypeScript, Vite\nTailored Vanilla CSS Design System"),
        (Inches(0.55), Inches(3.08), Inches(3.6), Inches(0.58), "Pure Rust 2021, Tokio Asynchronous I/O\nLock-Free SPSC Rings, AVX2 SIMD"),
        (Inches(0.55), Inches(3.88), Inches(3.6), Inches(0.58), "Flat Robin Hood Map, Slab Arenas\nNVMe Block Paging, 2-Bit CLOCK"),
        (Inches(0.55), Inches(4.68), Inches(3.6), Inches(0.58), "RESP3 Protocol, Diagnostic CLI Engine\nCriterion Microbenchmark Harness"),
        (Inches(0.55), Inches(5.48), Inches(3.6), Inches(0.58), "Native Windows & Linux Affinity\nHardware Core Pinning, Vercel Studio"),
    ]
    for x, y, w, h, txt in stack_content:
        tb = s4.shapes.add_textbox(x, y, w, h)
        tf = tb.text_frame
        tf.word_wrap = True
        tf.margin_top = tf.margin_bottom = tf.margin_left = tf.margin_right = 0
        p = tf.paragraphs[0]
        p.text = txt
        p.font.name = "Arial"
        p.font.size = Pt(10.5)
        p.font.color.rgb = TEXT_WHITE
        p.alignment = PP_ALIGN.CENTER

    # Right side: System Architecture Pipeline Overview (x ≈ 4.8 to 12.5)
    tb_arch = s4.shapes.add_textbox(Inches(4.8), Inches(2.1), Inches(7.7), Inches(4.8))
    tf_arch = tb_arch.text_frame
    tf_arch.word_wrap = True

    p_a0 = tf_arch.paragraphs[0]
    p_a0.text = "HIGH-PERFORMANCE DATA PIPELINE (END-TO-END TRACE)"
    p_a0.font.name = "Arial Black"
    p_a0.font.size = Pt(14)
    p_a0.font.bold = True
    p_a0.font.color.rgb = TECH_CYAN

    pipeline_steps = [
        ("1. VECTOR INGRESS (protocol/simd.rs)", "AVX2 256-bit SIMD delimiter scanner inspects 32 bytes per cycle at 289M ops/sec with zero copy."),
        ("2. CONSISTENT SHARD DISPATCH (sharding/router.rs)", "Murmur3-32 + Jump Consistent Hash routes key to core-pinned worker over 64B cache-padded SPSC ring queue."),
        ("3. FLAT INDEX LOOKUP (storage/robin_hood.rs)", "Flat Robin Hood hash table with Distance-from-Ideal-Bucket (DIB) and backward-shift deletion (0 tombstones)."),
        ("4. MEMORY HIERARCHY (memory/slab_arena.rs & tiering/)", "Hot keys served from 2MB hugepage slab arena. Cold keys identified by 2-bit CLOCK swept to NVMe 4KB blocks."),
        ("5. O(1) EXPIRATION WHEEL (timer/timing_wheel.rs)", "4-tier cascaded timing wheels detach expired keys in O(1) time, completely immune to TTL avalanche spikes."),
        ("6. DURABILITY & RECOVERY (snapshot/ & wal/)", "Monotonic epoch streaming snapshots and segmented CRC64 WAL guarantee zero data loss without fork()."),
    ]
    for title, desc in pipeline_steps:
        p_t = tf_arch.add_paragraph()
        p_t.text = f"▸ {title}"
        p_t.font.name = "Arial Black"
        p_t.font.size = Pt(11)
        p_t.font.bold = True
        p_t.font.color.rgb = NEON_GREEN
        p_t.space_before = Pt(6)

        p_d = tf_arch.add_paragraph()
        p_d.text = f"   {desc}"
        p_d.font.name = "Arial"
        p_d.font.size = Pt(10.5)
        p_d.font.color.rgb = TEXT_WHITE
        p_d.space_before = Pt(1)

    # =========================================================================
    # SLIDE 5: KEY FEATURES & UI/UX
    # =========================================================================
    s5 = prs.slides.add_slide(blank_layout)
    set_slide_background(s5, "hackathon_slides/slide_5.png")

    # 4 Feature Cards arranged cleanly in 2x2 grid
    card_w = Inches(5.6)
    card_h = Inches(2.25)
    x1, x2 = Inches(0.8), Inches(6.8)
    y1, y2 = Inches(2.2), Inches(4.75)

    features = [
        (x1, y1, "1. KEYSPACE BROWSER (CRUD & HEX)", [
            "Interactive object inspector with virtual namespace folders (user:*, auth:*).",
            "3 viewing formats: Raw text, Formatted JSON, and Binary Hex Dump with offsets.",
            "Live TTL countdown and memory footprint in bytes without locking KEYS *."
        ], NEON_GREEN),
        (x2, y1, "2. CLUSTER TOPOLOGY & HARDWARE MAP", [
            "Thread-per-core hardware affinity map showing pinned worker threads.",
            "Lock-free SPSC IPC ring buffer queue depths and zero false sharing metrics.",
            "Bitmapped Slab Allocator fragmentation telemetry (RSS / Logical < 1.02)."
        ], TECH_CYAN),
        (x1, y2, "3. 2-BIT CLOCK STORAGE TIERING", [
            "Visual frequency tracker sweeping hot working sets vs. cold dormant keys.",
            "Automatic demotion to NVMe 4KB blocks with 12-byte DiskPointers (79.1% DRAM cut).",
            "Live on-demand cache promotion restoring cold keys to RAM in microseconds."
        ], TECH_CYAN),
        (x2, y2, "4. QUERY PLAN TRACE (--explain) & APM", [
            "Colored execution tree showing microsecond breakdown across all 5 engine stages.",
            "Live APM telemetry: Throughput (QPS), Cache Hit Rate (99.4%), P50/P99 latency.",
            "Embedded RESP3 Workbench terminal with live microsecond execution timer."
        ], NEON_GREEN),
    ]

    for cx, cy, c_title, c_bullets, col in features:
        card = s5.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, cx, cy, card_w, card_h)
        card.fill.solid()
        card.fill.fore_color.rgb = CARD_BG
        card.line.color.rgb = CARD_BORDER
        card.line.width = Pt(1.5)

        tb = s5.shapes.add_textbox(cx + Inches(0.2), cy + Inches(0.15), card_w - Inches(0.4), card_h - Inches(0.3))
        tf = tb.text_frame
        tf.word_wrap = True
        p = tf.paragraphs[0]
        p.text = c_title
        p.font.name = "Arial Black"
        p.font.size = Pt(13)
        p.font.bold = True
        p.font.color.rgb = col

        for b in c_bullets:
            p_b = tf.add_paragraph()
            p_b.text = f"• {b}"
            p_b.font.name = "Arial"
            p_b.font.size = Pt(11)
            p_b.font.color.rgb = TEXT_WHITE
            p_b.space_before = Pt(4)

    # =========================================================================
    # SLIDE 6: IMPLEMENTATION ROADMAP & SCOPE
    # =========================================================================
    s6 = prs.slides.add_slide(blank_layout)
    set_slide_background(s6, "hackathon_slides/slide_6.png")

    c_w = Inches(3.64)
    c_gap = Inches(0.4)
    c_y = Inches(2.2)
    c_h = Inches(4.7)

    roadmap_cols = [
        (Inches(0.8), "BUILT TODAY (100% COMPLETE)", [
            "Core Engine (All 8 Architecture Phases):",
            "  • Flat Robin Hood Map (zero tombstones)",
            "  • 4-Tier Timing Wheel (O(1) detach)",
            "  • Thread-per-Core Shards & SPSC Rings",
            "  • 64-bit Bitmapped Slab Allocators",
            "  • Forkless Snapshots & CRC64 WAL",
            "  • 2-Bit CLOCK NVMe Tiering (79.1% cut)",
            "  • AVX2 SIMD Scanner (289M ops/s)",
            "Testing & Verification:",
            "  • 105 / 105 automated tests passing",
            "Valence Studio:",
            "  • Complete 7-panel web console + CLI"
        ], NEON_GREEN),
        (Inches(0.8) + c_w + c_gap, "IN PROGRESS / UNDERWAY", [
            "Distributed Clustering:",
            "  • Multi-node Raft consensus replication",
            "  • Automatic hash-slot rebalancing via gRPC",
            "Kernel Acceleration:",
            "  • Linux io_uring submission queues for direct NVMe disk block reads",
            "Client Ecosystem:",
            "  • Pipelined native drivers for Rust, Python (asyncio), and Node.js",
            "Security & Access:",
            "  • RESP3 TLS/SSL termination & ACLs"
        ], TECH_CYAN),
        (Inches(0.8) + (c_w + c_gap)*2, "BEYOND THE HACKATHON", [
            "Serverless Elastic Valence:",
            "  • Auto-scaling cloud tier with S3 cold object storage spillover",
            "Global Active-Active Replication:",
            "  • Conflict-free replicated data types (CRDTs) across worldwide regions",
            "Commercial Managed Cloud DB:",
            "  • Fully managed cloud database SaaS undercutting AWS ElastiCache by 60%",
            "Enterprise Compliance:",
            "  • SOC2 audit logging & encryption"
        ], NEON_GREEN),
    ]

    for cx, c_title, c_items, col in roadmap_cols:
        card = s6.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, cx, c_y, c_w, c_h)
        card.fill.solid()
        card.fill.fore_color.rgb = CARD_BG
        card.line.color.rgb = CARD_BORDER
        card.line.width = Pt(1.5)

        tb = s6.shapes.add_textbox(cx + Inches(0.2), c_y + Inches(0.2), c_w - Inches(0.4), c_h - Inches(0.4))
        tf = tb.text_frame
        tf.word_wrap = True
        p = tf.paragraphs[0]
        p.text = c_title
        p.font.name = "Arial Black"
        p.font.size = Pt(13)
        p.font.bold = True
        p.font.color.rgb = col

        for item in c_items:
            p_item = tf.add_paragraph()
            p_item.text = item
            p_item.font.name = "Arial"
            p_item.font.size = Pt(11)
            p_item.font.color.rgb = TEXT_WHITE if not item.startswith("  •") else TEXT_MUTED
            p_item.space_before = Pt(3)

    # =========================================================================
    # SLIDE 7: EXPECTED IMPACT & SCALABILITY
    # =========================================================================
    s7 = prs.slides.add_slide(blank_layout)
    set_slide_background(s7, "hackathon_slides/slide_7.png")

    impact_cols = [
        (Inches(0.8), "WHO BENEFITS", [
            "Startups & Tech Enterprises:",
            "  • Slash cloud caching bills by 70–80% without rewriting existing Redis application code.",
            "High-Frequency Platforms:",
            "  • Fintech, gaming, and real-time ad tech achieve sub-microsecond latency without GC jitter.",
            "DevOps & SRE Teams:",
            "  • Zero Out-Of-Memory (OOM) snapshot crashes and robust torn-tail WAL crash recovery."
        ], NEON_GREEN),
        (Inches(0.8) + c_w + c_gap, "THE MARKET OPPORTUNITY", [
            "$15B+ In-Memory Database Market:",
            "  • Dominated by Redis, AWS MemoryDB, Aerospike, and Hazelcast.",
            "Hardware Utilization Gap:",
            "  • 64-core cloud instances are wasted by single-threaded engines; Valence scales linearly with every core.",
            "Cloud Unit Economics:",
            "  • Saving 79% on DRAM across large clusters saves enterprises tens of thousands of dollars each month."
        ], TECH_CYAN),
        (Inches(0.8) + (c_w + c_gap)*2, "GROWTH BEYOND A DEMO", [
            "Open-Source Community:",
            "  • Release Valence under Apache 2.0 / MIT to build a global systems community.",
            "Production Validation:",
            "  • Rigorous Jepsen chaos verification for partition and split-brain resilience.",
            "Commercial SaaS Business Model:",
            "  • High-margin managed database offering competing directly with AWS ElastiCache."
        ], NEON_GREEN),
    ]

    for cx, c_title, c_items, col in impact_cols:
        card = s7.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, cx, c_y, c_w, c_h)
        card.fill.solid()
        card.fill.fore_color.rgb = CARD_BG
        card.line.color.rgb = CARD_BORDER
        card.line.width = Pt(1.5)

        tb = s7.shapes.add_textbox(cx + Inches(0.2), c_y + Inches(0.2), c_w - Inches(0.4), c_h - Inches(0.4))
        tf = tb.text_frame
        tf.word_wrap = True
        p = tf.paragraphs[0]
        p.text = c_title
        p.font.name = "Arial Black"
        p.font.size = Pt(13)
        p.font.bold = True
        p.font.color.rgb = col

        for item in c_items:
            p_item = tf.add_paragraph()
            p_item.text = item
            p_item.font.name = "Arial"
            p_item.font.size = Pt(11)
            p_item.font.color.rgb = TEXT_WHITE if not item.startswith("  •") else TEXT_MUTED
            p_item.space_before = Pt(4)

    # =========================================================================
    # SLIDE 8: THANK YOU & LIVE DEMO
    # =========================================================================
    s8 = prs.slides.add_slide(blank_layout)
    set_slide_background(s8, "hackathon_slides/slide_8.png")

    # Center Card below the Thank You title (y ≈ 4.8 to 6.8)
    card_thk = s8.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, Inches(1.5), Inches(4.5), Inches(10.333), Inches(2.3))
    card_thk.fill.solid()
    card_thk.fill.fore_color.rgb = CARD_BG
    card_thk.line.color.rgb = CARD_BORDER
    card_thk.line.width = Pt(1.5)

    tb_thk = s8.shapes.add_textbox(Inches(1.8), Inches(4.6), Inches(9.733), Inches(2.1))
    tf_thk = tb_thk.text_frame
    tf_thk.word_wrap = True

    p = tf_thk.paragraphs[0]
    p.text = "LIVE SYSTEM DEMONSTRATION & VERIFICATION"
    p.font.name = "Arial Black"
    p.font.size = Pt(15)
    p.font.bold = True
    p.font.color.rgb = TECH_CYAN
    p.alignment = PP_ALIGN.CENTER

    thk_items = [
        "🌐 Live Web Management Studio: http://localhost:5173/ (Interactive 7-Panel Console)",
        "⚡ Automated Test Verification: 105 / 105 Passing Tests (cargo test --release)",
        "🔍 Diagnostic Terminal Engine: target\\release\\valence-cli.exe --explain (AVX2 SIMD Trace)",
        "🚀 1-Click Interactive Launcher: start_demo.bat (Windows) & start_demo.sh (Linux/macOS)",
        "💬 Ready for Judge Questions & Live Terminal Benchmarking!"
    ]
    for item in thk_items:
        p_i = tf_thk.add_paragraph()
        p_i.text = item
        p_i.font.name = "Arial"
        p_i.font.size = Pt(12)
        p_i.font.color.rgb = TEXT_WHITE
        p_i.alignment = PP_ALIGN.CENTER
        p_i.space_before = Pt(3)

    output_path = "Valence_CodeApex_Official.pptx"
    prs.save(output_path)
    print(f"[SUCCESS] Official presentation generated at: {output_path}")

    # Copy to Downloads
    downloads_path = r"C:\Users\akash\Downloads\Valence_CodeApex_Official.pptx"
    import shutil
    shutil.copyfile(output_path, downloads_path)
    print(f"[SUCCESS] Copied to Downloads at: {downloads_path}")

if __name__ == "__main__":
    build_official_deck()
