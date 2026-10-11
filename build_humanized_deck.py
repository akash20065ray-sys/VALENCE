import os
from pptx import Presentation
from pptx.util import Inches, Pt
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.enum.shapes import MSO_SHAPE

def build_humanized_deck():
    prs = Presentation()
    prs.slide_width = Inches(13.333)
    prs.slide_height = Inches(7.5)
    blank_layout = prs.slide_layouts[6]

    # Theme Colors
    NEON_GREEN   = RGBColor(34, 197, 94)    # #22C55E
    TECH_CYAN    = RGBColor(6, 182, 212)    # #06B6D4
    TEXT_WHITE   = RGBColor(248, 250, 252)  # #F8FAFC
    TEXT_MUTED   = RGBColor(148, 163, 184)  # #94A3B8
    TEXT_DIM     = RGBColor(100, 116, 139)  # #64748B
    CARD_BG      = RGBColor(12, 24, 18)     # Dark Emerald Tint
    CARD_BORDER  = RGBColor(22, 101, 52)    # Emerald Border

    def set_slide_background(slide, image_path):
        slide.shapes.add_picture(image_path, 0, 0, width=Inches(13.333), height=Inches(7.5))

    # =========================================================================
    # SLIDE 1: TITLE & TEAM DETAILS
    # =========================================================================
    s1 = prs.slides.add_slide(blank_layout)
    set_slide_background(s1, "hackathon_slides/slide_1.png")

    # Clear, Human Project Hook
    tb_title = s1.shapes.add_textbox(Inches(0.8), Inches(2.2), Inches(11.733), Inches(1.3))
    tf_title = tb_title.text_frame
    tf_title.word_wrap = True

    p_t = tf_title.paragraphs[0]
    p_t.text = "VALENCE"
    p_t.font.name = "Segoe UI"
    p_t.font.size = Pt(36)
    p_t.font.bold = True
    p_t.font.color.rgb = NEON_GREEN
    p_t.alignment = PP_ALIGN.CENTER

    p_t_sub = tf_title.add_paragraph()
    p_t_sub.text = "A Modern, Multi-Core Replacement for Redis — Built in Pure Rust"
    p_t_sub.font.name = "Segoe UI"
    p_t_sub.font.size = Pt(17)
    p_t_sub.font.bold = True
    p_t_sub.font.color.rgb = TECH_CYAN
    p_t_sub.alignment = PP_ALIGN.CENTER
    p_t_sub.space_before = Pt(4)

    p_t_tag = tf_title.add_paragraph()
    p_t_tag.text = "Uses all CPU cores • Cuts RAM bills by 79% via NVMe • Zero crash risk during backups"
    p_t_tag.font.name = "Segoe UI"
    p_t_tag.font.size = Pt(13)
    p_t_tag.font.color.rgb = TEXT_WHITE
    p_t_tag.alignment = PP_ALIGN.CENTER
    p_t_tag.space_before = Pt(3)

    # Team Box Content (mapped precisely to template lines)
    tb_team = s1.shapes.add_textbox(Inches(3.3), Inches(5.15), Inches(8.8), Inches(1.6))
    tf_team = tb_team.text_frame
    tf_team.word_wrap = True

    p_name = tf_team.paragraphs[0]
    p_name.text = "Team Valence Systems"
    p_name.font.name = "Segoe UI"
    p_name.font.size = Pt(15)
    p_name.font.bold = True
    p_name.font.color.rgb = TEXT_WHITE

    p_lead = tf_team.add_paragraph()
    p_lead.text = "Akash Ray & Team"
    p_lead.font.name = "Segoe UI"
    p_lead.font.size = Pt(15)
    p_lead.font.bold = True
    p_lead.font.color.rgb = TEXT_WHITE
    p_lead.space_before = Pt(9)

    p_dom = tf_team.add_paragraph()
    p_dom.text = "Cloud Infrastructure & High-Performance Data Systems"
    p_dom.font.name = "Segoe UI"
    p_dom.font.size = Pt(15)
    p_dom.font.bold = True
    p_dom.font.color.rgb = TECH_CYAN
    p_dom.space_before = Pt(9)

    # =========================================================================
    # SLIDE 2: PROBLEM STATEMENT & TARGET NEED (Human, Relatable Story)
    # =========================================================================
    s2 = prs.slides.add_slide(blank_layout)
    set_slide_background(s2, "hackathon_slides/slide_2.png")

    # Section 1: Who is Affected
    tb_p1 = s2.shapes.add_textbox(Inches(0.8), Inches(2.72), Inches(11.7), Inches(0.95))
    tf_p1 = tb_p1.text_frame
    tf_p1.word_wrap = True
    p = tf_p1.paragraphs[0]
    p.text = "Every modern app uses an in-memory cache (like Redis) so their websites feel instant — Twitter, Uber, Netflix, and Fintech."
    p.font.name = "Segoe UI"
    p.font.size = Pt(13.5)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_p1.add_paragraph()
    p2.text = "Engineering teams end up paying thousands of dollars every month to AWS for memory they barely touch."
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TEXT_MUTED
    p2.space_before = Pt(3)

    # Section 2: The Current Gap (The 3 Flaws of Redis)
    tb_p2 = s2.shapes.add_textbox(Inches(0.8), Inches(4.12), Inches(11.7), Inches(1.15))
    tf_p2 = tb_p2.text_frame
    tf_p2.word_wrap = True
    p = tf_p2.paragraphs[0]
    p.text = "1. The 1-Core Bottleneck: Redis was built 15 years ago for single-core servers. On a 64-core cloud instance, 63 cores sit idle while 1 core is maxed out."
    p.font.name = "Segoe UI"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_p2.add_paragraph()
    p2.text = "2. The 'Cold RAM' Waste: Over 80% of cached keys are never read after 24 hours. Yet companies keep paying expensive RAM prices to store them."
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TEXT_MUTED
    p2.space_before = Pt(3)
    p3 = tf_p2.add_paragraph()
    p3.text = "3. The Backup Freeze: When Redis saves to disk, it duplicates its memory table. If memory is full, the server freezes or crashes with an Out-of-Memory (OOM) error."
    p3.font.name = "Segoe UI"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_WHITE
    p3.space_before = Pt(3)

    # Section 3: Why it Matters Now
    tb_p3 = s2.shapes.add_textbox(Inches(0.8), Inches(5.62), Inches(11.7), Inches(1.1))
    tf_p3 = tb_p3.text_frame
    tf_p3.word_wrap = True
    p = tf_p3.paragraphs[0]
    p.text = "• AI & Real-Time Explosion: LLM sessions, agent memory, and vector caching require microsecond speeds at lower cost."
    p.font.name = "Segoe UI"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_p3.add_paragraph()
    p2.text = "• Modern SSDs are Microsecond Fast: Fast NVMe drives can now store cold keys without slowing down the app."
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TEXT_MUTED
    p2.space_before = Pt(3)
    p3 = tf_p3.add_paragraph()
    p3.text = "• Immediate Cloud Savings: Slashing database RAM by 79% saves real companies thousands of dollars on AWS every month."
    p3.font.name = "Segoe UI"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TECH_CYAN
    p3.space_before = Pt(3)

    # =========================================================================
    # SLIDE 3: PROPOSED SOLUTION & INNOVATION (Grounded & Simple)
    # =========================================================================
    s3 = prs.slides.add_slide(blank_layout)
    set_slide_background(s3, "hackathon_slides/slide_3.png")

    # Section 1: What makes it different
    tb_s1 = s3.shapes.add_textbox(Inches(0.8), Inches(2.72), Inches(11.7), Inches(0.95))
    tf_s1 = tb_s1.text_frame
    tf_s1.word_wrap = True
    p = tf_s1.paragraphs[0]
    p.text = "• Built From Scratch in Pure Rust: Zero C code, zero garbage collector pauses, zero memory safety bugs."
    p.font.name = "Segoe UI"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_s1.add_paragraph()
    p2.text = "• Drop-in Redis Compatible: Speaks the standard Redis RESP3 protocol — existing apps can switch with zero code changes."
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TECH_CYAN
    p2.space_before = Pt(3)
    p3 = tf_s1.add_paragraph()
    p3.text = "• AVX2 Vector Acceleration: Uses CPU vector instructions to read incoming commands at 289 million ops per second."
    p3.font.name = "Segoe UI"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_MUTED
    p3.space_before = Pt(3)

    # Section 2: Core Mechanism (How it works in plain English)
    tb_s2 = s3.shapes.add_textbox(Inches(0.8), Inches(4.12), Inches(11.7), Inches(1.15))
    tf_s2 = tb_s2.text_frame
    tf_s2.word_wrap = True
    p = tf_s2.paragraphs[0]
    p.text = "• Uses All CPU Cores: Gives every CPU core its own worker thread with lock-free ring queues. Threads never fight or wait in line."
    p.font.name = "Segoe UI"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_s2.add_paragraph()
    p2.text = "• Smart NVMe Tiering (2-bit CLOCK): Popular data stays in fast RAM; untouched data moves to SSD disk, cutting RAM costs by 79.1%."
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TECH_CYAN
    p2.space_before = Pt(3)
    p3 = tf_s2.add_paragraph()
    p3.text = "• Safe Forkless Snapshots: Streams backups directly to disk without duplicating memory. No server freezes, no Out-of-Memory crashes."
    p3.font.name = "Segoe UI"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_MUTED
    p3.space_before = Pt(3)

    # Section 3: Why it's feasible in this timeframe
    tb_s3 = s3.shapes.add_textbox(Inches(0.8), Inches(5.62), Inches(11.7), Inches(1.1))
    tf_s3 = tb_s3.text_frame
    tf_s3.word_wrap = True
    p = tf_s3.paragraphs[0]
    p.text = "• 100% Fully Built & Verified Today: Not a mock idea — all 8 phases are complete and passing all 105 automated test suites."
    p.font.name = "Segoe UI"
    p.font.size = Pt(13)
    p.font.color.rgb = TEXT_WHITE
    p2 = tf_s3.add_paragraph()
    p2.text = "• Bitmapped Slab Allocator: Custom memory slabs allocate in 1.19 nanoseconds with zero memory fragmentation."
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.color.rgb = TECH_CYAN
    p2.space_before = Pt(3)
    p3 = tf_s3.add_paragraph()
    p3.text = "• Live Web Console & CLI: Running live right now on localhost for judges to test hands-on."
    p3.font.name = "Segoe UI"
    p3.font.size = Pt(13)
    p3.font.color.rgb = TEXT_MUTED
    p3.space_before = Pt(3)

    # =========================================================================
    # SLIDE 4: SYSTEM ARCHITECTURE & TECH STACK (With Diagram Image!)
    # =========================================================================
    s4 = prs.slides.add_slide(blank_layout)
    set_slide_background(s4, "hackathon_slides/slide_4.png")

    # Left Tech Stack boxes (Clean & human)
    stack_content = [
        (Inches(0.55), Inches(2.28), Inches(3.6), Inches(0.58), "React 19, TypeScript, Vite\nCustom Vanilla CSS Design System"),
        (Inches(0.55), Inches(3.08), Inches(3.6), Inches(0.58), "Pure Rust 2021, Tokio Async\nLock-Free SPSC Rings, AVX2 SIMD"),
        (Inches(0.55), Inches(3.88), Inches(3.6), Inches(0.58), "Flat Robin Hood Map, Slab Arenas\nNVMe 4KB Paging, 2-Bit CLOCK"),
        (Inches(0.55), Inches(4.68), Inches(3.6), Inches(0.58), "RESP3 Protocol, Diagnostic CLI Engine\nCriterion Microbenchmark Harness"),
        (Inches(0.55), Inches(5.48), Inches(3.6), Inches(0.58), "Native CPU Core Pinning (Win/Linux)\nValence Studio Web Console"),
    ]
    for x, y, w, h, txt in stack_content:
        tb = s4.shapes.add_textbox(x, y, w, h)
        tf = tb.text_frame
        tf.word_wrap = True
        tf.margin_top = tf.margin_bottom = tf.margin_left = tf.margin_right = 0
        p = tf.paragraphs[0]
        p.text = txt
        p.font.name = "Segoe UI"
        p.font.size = Pt(10.5)
        p.font.color.rgb = TEXT_WHITE
        p.alignment = PP_ALIGN.CENTER

    # Right side: Embed the REAL generated Architecture Diagram!
    arch_img_path = "hackathon_slides/architecture_diagram.png"
    if os.path.exists(arch_img_path):
        s4.shapes.add_picture(arch_img_path, Inches(4.5), Inches(2.1), width=Inches(8.2), height=Inches(4.8))

    # =========================================================================
    # SLIDE 5: KEY FEATURES & UI/UX (With Live UI Showcase Graphic!)
    # =========================================================================
    s5 = prs.slides.add_slide(blank_layout)
    set_slide_background(s5, "hackathon_slides/slide_5.png")

    # Embed the REAL generated UI Showcase Image (Keyspace + Tiering + APM + CLI trace)!
    ui_img_path = "hackathon_slides/ui_showcase.png"
    if os.path.exists(ui_img_path):
        s5.shapes.add_picture(ui_img_path, Inches(0.8), Inches(2.1), width=Inches(11.733), height=Inches(4.8))

    # =========================================================================
    # SLIDE 6: IMPLEMENTATION ROADMAP & SCOPE (Honest, 100% Real)
    # =========================================================================
    s6 = prs.slides.add_slide(blank_layout)
    set_slide_background(s6, "hackathon_slides/slide_6.png")

    c_w = Inches(3.64)
    c_gap = Inches(0.4)
    c_y = Inches(2.2)
    c_h = Inches(4.7)

    roadmap_cols = [
        (Inches(0.8), "BUILT TODAY (100% VERIFIED)", [
            "All 8 Core Engine Phases in Rust:",
            "  • Flat Robin Hood Map (0 tombstones)",
            "  • 4-Tier Timing Wheel (O(1) detach)",
            "  • Thread-per-Core Shards & SPSC Rings",
            "  • 64-bit Bitmapped Slab Allocator",
            "  • Forkless Snapshots & CRC64 WAL",
            "  • 2-Bit CLOCK NVMe Tiering (79.1% cut)",
            "  • AVX2 SIMD Scanner (289M ops/s)",
            "Verified Test Suite:",
            "  • 105 / 105 automated tests passing",
            "Valence Studio:",
            "  • Full 7-panel web console + CLI"
        ], NEON_GREEN),
        (Inches(0.8) + c_w + c_gap, "IN PROGRESS / UNDERWAY", [
            "Distributed Clustering:",
            "  • Multi-node Raft consensus replication",
            "  • Automatic shard rebalancing via gRPC",
            "Kernel Acceleration:",
            "  • Linux io_uring submission queues for direct NVMe disk block reads",
            "Client Libraries:",
            "  • High-throughput async drivers for Python, Rust, and Node.js",
            "Security:",
            "  • RESP3 TLS encryption & ACL users"
        ], TECH_CYAN),
        (Inches(0.8) + (c_w + c_gap)*2, "BEYOND THE HACKATHON", [
            "Serverless Elastic Cache:",
            "  • Auto-scaling cloud tier with S3 cold object storage spillover",
            "Global Multi-Region Replication:",
            "  • Conflict-free replicated data types (CRDTs) across worldwide cloud regions",
            "Commercial Managed Cloud DB:",
            "  • Managed cloud database offering undercutting AWS ElastiCache by 60%",
            "Enterprise Compliance:",
            "  • SOC2 audit trails & backup sync"
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
        p.font.name = "Segoe UI"
        p.font.size = Pt(13.5)
        p.font.bold = True
        p.font.color.rgb = col

        for item in c_items:
            p_item = tf.add_paragraph()
            p_item.text = item
            p_item.font.name = "Segoe UI"
            p_item.font.size = Pt(11)
            p_item.font.color.rgb = TEXT_WHITE if not item.startswith("  •") else TEXT_MUTED
            p_item.space_before = Pt(3)

    # =========================================================================
    # SLIDE 7: EXPECTED IMPACT & SCALABILITY (Clear Business Value)
    # =========================================================================
    s7 = prs.slides.add_slide(blank_layout)
    set_slide_background(s7, "hackathon_slides/slide_7.png")

    impact_cols = [
        (Inches(0.8), "WHO BENEFITS", [
            "Startups & Growing Apps:",
            "  • Cut cloud caching bills by 70–80% without rewriting existing Redis application code.",
            "High-Throughput Systems:",
            "  • Fintech, gaming, and real-time ad tech get predictable sub-microsecond latency without GC pauses.",
            "Platform & SRE Teams:",
            "  • Zero Out-of-Memory crashes during backups and automatic crash recovery via WAL."
        ], NEON_GREEN),
        (Inches(0.8) + c_w + c_gap, "THE MARKET OPPORTUNITY", [
            "$15B+ In-Memory Database Market:",
            "  • Dominated by Redis, AWS MemoryDB, and Aerospike.",
            "Fixing Cloud Hardware Waste:",
            "  • 64-core cloud servers are wasted by legacy single-core engines; Valence scales linearly across all cores.",
            "Huge Dollar Savings:",
            "  • Slashing 79% of RAM across large enterprise clusters saves tens of thousands of dollars every month."
        ], TECH_CYAN),
        (Inches(0.8) + (c_w + c_gap)*2, "GROWTH BEYOND A DEMO", [
            "Open-Source Adoption:",
            "  • Releasing Valence core under Apache 2.0 / MIT to build a global community of systems engineers.",
            "Battle-Testing & Reliability:",
            "  • Jepsen chaos testing for network partition tolerance and crash resilience.",
            "Commercial SaaS Tier:",
            "  • Fully managed cloud offering competing directly with AWS ElastiCache."
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
        p.font.name = "Segoe UI"
        p.font.size = Pt(13.5)
        p.font.bold = True
        p.font.color.rgb = col

        for item in c_items:
            p_item = tf.add_paragraph()
            p_item.text = item
            p_item.font.name = "Segoe UI"
            p_item.font.size = Pt(11)
            p_item.font.color.rgb = TEXT_WHITE if not item.startswith("  •") else TEXT_MUTED
            p_item.space_before = Pt(4)

    # =========================================================================
    # SLIDE 8: THANK YOU & LIVE DEMO
    # =========================================================================
    s8 = prs.slides.add_slide(blank_layout)
    set_slide_background(s8, "hackathon_slides/slide_8.png")

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
    p.font.name = "Segoe UI"
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
        p_i.font.name = "Segoe UI"
        p_i.font.size = Pt(12)
        p_i.font.color.rgb = TEXT_WHITE
        p_i.alignment = PP_ALIGN.CENTER
        p_i.space_before = Pt(3)

    output_path = "Valence_CodeApex_Official.pptx"
    prs.save(output_path)
    print(f"[SUCCESS] Humanized presentation generated at: {output_path}")

    downloads_path = r"C:\Users\akash\Downloads\Valence_CodeApex_Official.pptx"
    import shutil
    shutil.copyfile(output_path, downloads_path)
    print(f"[SUCCESS] Copied to Downloads at: {downloads_path}")

if __name__ == "__main__":
    build_humanized_deck()
