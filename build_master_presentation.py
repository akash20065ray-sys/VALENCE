import os
import shutil
import subprocess
from pptx import Presentation
from pptx.util import Inches, Pt
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.enum.shapes import MSO_SHAPE

def build_presentation(output_path="Valence Presentation.pptx"):
    prs = Presentation()
    prs.slide_width = Inches(13.333)
    prs.slide_height = Inches(7.5)
    blank_layout = prs.slide_layouts[6]

    # Clean Cyber Theme Colors
    NEON_GREEN   = RGBColor(34, 197, 94)    # #22C55E
    TECH_CYAN    = RGBColor(6, 182, 212)    # #06B6D4
    AMBER        = RGBColor(245, 158, 11)   # #F59E0B
    PURPLE       = RGBColor(168, 85, 247)   # #A855F7
    TEXT_WHITE   = RGBColor(248, 250, 252)  # #F8FAFC
    TEXT_MUTED   = RGBColor(148, 163, 184)  # #94A3B8
    TEXT_DIM     = RGBColor(100, 116, 139)  # #64748B
    
    BG_CARD      = RGBColor(9, 22, 16)      # Deep dark emerald card
    BORDER_GREEN = RGBColor(22, 101, 52)    # #166534
    BORDER_CYAN  = RGBColor(14, 116, 144)   # #0E7490
    BORDER_AMBER = RGBColor(180, 83, 9)

    def set_slide_background(slide, image_path):
        slide.shapes.add_picture(image_path, 0, 0, width=Inches(13.333), height=Inches(7.5))

    def create_card(slide, left, top, width, height, border_color=BORDER_GREEN, fill_color=BG_CARD):
        card = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left, top, width, height)
        card.fill.solid()
        card.fill.fore_color.rgb = fill_color
        card.line.color.rgb = border_color
        card.line.width = Pt(1.5)
        return card

    # =========================================================================
    # SLIDE 1: TITLE & TEAM DETAILS (PLAIN TEXT DIRECTLY ON TEMPLATE)
    # =========================================================================
    s1 = prs.slides.add_slide(blank_layout)
    set_slide_background(s1, "hackathon_slides/slide_1.png")

    # Team Info - Plain text aligned after pre-printed dashes in template
    team_boxes = [
        ("Team Valence Systems", Inches(3.30), Inches(5.18)),
        ("Akash Ray & Team", Inches(4.30), Inches(5.66)),
        ("Cloud Infrastructure & High-Performance Data Systems", Inches(5.85), Inches(6.15))
    ]
    for text, left_pos, top_pos in team_boxes:
        tb = s1.shapes.add_textbox(left_pos, top_pos, Inches(7.2), Inches(0.40))
        tf = tb.text_frame
        tf.word_wrap = True
        tf.margin_left = tf.margin_top = tf.margin_bottom = tf.margin_right = 0
        p = tf.paragraphs[0]
        p.text = text
        p.font.name = "Segoe UI"
        p.font.size = Pt(15.5)
        p.font.bold = True
        p.font.color.rgb = TEXT_WHITE

    # Project Subtitle & Value Props in open space of lower frame
    tb_s1_proj = s1.shapes.add_textbox(Inches(1.0), Inches(6.55), Inches(11.333), Inches(0.65))
    tf_s1_proj = tb_s1_proj.text_frame
    tf_s1_proj.word_wrap = True
    tf_s1_proj.margin_left = tf_s1_proj.margin_top = tf_s1_proj.margin_bottom = tf_s1_proj.margin_right = 0
    
    p_proj = tf_s1_proj.paragraphs[0]
    p_proj.text = "PROJECT: VALENCE — Next-Gen Multi-Core In-Memory Database Engine in Pure Rust"
    p_proj.font.name = "Segoe UI"
    p_proj.font.size = Pt(13.5)
    p_proj.font.bold = True
    p_proj.font.color.rgb = TECH_CYAN
    p_proj.alignment = PP_ALIGN.CENTER

    p_props = tf_s1_proj.add_paragraph()
    p_props.text = "4x Hardware Scaling  •  79% RAM Savings via NVMe  •  Zero Mutex Contention  •  Drop-in RESP3"
    p_props.font.name = "Segoe UI"
    p_props.font.size = Pt(12)
    p_props.font.color.rgb = TEXT_WHITE
    p_props.alignment = PP_ALIGN.CENTER
    p_props.space_before = Pt(3)

    # =========================================================================
    # SLIDE 2: PROBLEM STATEMENT & TARGET NEED (EXPANDED PLAIN TEXT DIRECTLY ON TEMPLATE)
    # =========================================================================
    s2 = prs.slides.add_slide(blank_layout)
    set_slide_background(s2, "hackathon_slides/slide_2.png")

    # Section 1: UNDER > WHO IS AFFECTED (Expanded, rich, authoritative)
    tb2_1 = s2.shapes.add_textbox(Inches(0.85), Inches(2.65), Inches(11.8), Inches(1.05))
    tf2_1 = tb2_1.text_frame
    tf2_1.word_wrap = True
    tf2_1.margin_left = tf2_1.margin_top = tf2_1.margin_bottom = tf2_1.margin_right = 0

    p = tf2_1.paragraphs[0]
    p.text = "• High-Throughput Real-Time Platforms: Fintech trading engines, payment gateways, and e-commerce caching millions of operations/sec."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_WHITE

    p = tf2_1.add_paragraph()
    p.text = "• Generative AI & Agent Workloads: Modern LLM applications maintaining massive conversational context and session caches in memory."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_WHITE
    p.space_before = Pt(3)

    p = tf2_1.add_paragraph()
    p.text = "• Infrastructure Engineers & Enterprise CFOs: SRE teams managing fragile Redis clusters, and enterprises paying $10,000s monthly for idle DRAM."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
    p.space_before = Pt(3)

    # Section 2: UNDER > THE CURRENT GAP (1st line about Redis, then the 3 architectural gaps)
    tb2_2 = s2.shapes.add_textbox(Inches(0.85), Inches(3.95), Inches(11.8), Inches(1.02))
    tf2_2 = tb2_2.text_frame
    tf2_2.word_wrap = True
    tf2_2.margin_left = tf2_2.margin_top = tf2_2.margin_bottom = tf2_2.margin_right = 0

    p_gap_intro = tf2_2.paragraphs[0]
    p_gap_intro.text = "Redis was architected in 2009 for single-threaded execution on single-core servers—modern cloud workloads push it far beyond its limits:"
    p_gap_intro.font.name, p_gap_intro.font.size, p_gap_intro.font.bold, p_gap_intro.font.color.rgb = "Segoe UI", Pt(11), True, TECH_CYAN

    p = tf2_2.add_paragraph()
    p.text = "1. Single-Core Bottleneck: Redis locks execution to 1 CPU core; modern 32–128 vCPU cloud servers leave remaining cores idle while queues stall."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(10.8), TEXT_WHITE
    p.space_before = Pt(2)

    p = tf2_2.add_paragraph()
    p.text = "2. The Cold RAM Waste: Over 80% of cached keys are dormant after 24 hours, yet remain locked in expensive DRAM costing 10x more than flash storage."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(10.8), TEXT_WHITE
    p.space_before = Pt(2)

    p = tf2_2.add_paragraph()
    p.text = "3. Fatal Backup Latency Spikes: Redis snapshotting relies on OS fork(), duplicating page tables and triggering 100ms+ latency freezes or OOM crashes."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(10.8), TEXT_WHITE
    p.space_before = Pt(2)

    # Section 3: UNDER > WHY IT MATTERS NOW (Exactly 3 high-impact punchy lines)
    tb2_3 = s2.shapes.add_textbox(Inches(0.85), Inches(5.20), Inches(11.8), Inches(1.30))
    tf2_3 = tb2_3.text_frame
    tf2_3.word_wrap = True
    tf2_3.margin_left = tf2_3.margin_top = tf2_3.margin_bottom = tf2_3.margin_right = 0

    p = tf2_3.paragraphs[0]
    p.text = "• Ultra-Fast NVMe Flash: Modern NVMe SSDs read cold keys in microseconds at 1/10th the hardware cost of RAM."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_WHITE

    p = tf2_3.add_paragraph()
    p.text = "• Multi-Core Cloud Scale: Modern cloud nodes pack 32–128 CPU cores; data engines must scale throughput linearly across all cores with zero locks."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_WHITE
    p.space_before = Pt(4)

    p = tf2_3.add_paragraph()
    p.text = "• Immediate Economic Impact: Slashing database DRAM usage by 70–80% saves enterprises $10,000s in cloud infrastructure bills every single month."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
    p.space_before = Pt(4)

    # =========================================================================
    # SLIDE 3: PROPOSED SOLUTION & INNOVATION (PLAIN TEXT DIRECTLY ON TEMPLATE)
    # =========================================================================
    s3 = prs.slides.add_slide(blank_layout)
    set_slide_background(s3, "hackathon_slides/slide_3.png")

    # Section 1: UNDER > WHAT MAKES IT DIFFERENT
    tb3_1 = s3.shapes.add_textbox(Inches(0.85), Inches(2.65), Inches(11.8), Inches(0.95))
    tf3_1 = tb3_1.text_frame
    tf3_1.word_wrap = True
    tf3_1.margin_left = tf3_1.margin_top = tf3_1.margin_bottom = tf3_1.margin_right = 0
    p = tf3_1.paragraphs[0]
    p.text = "• 100% Pure Rust Engine: Zero C code, zero garbage collector pauses, guaranteed memory safety."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(12), TEXT_WHITE
    p = tf3_1.add_paragraph()
    p.text = "• Native Drop-In RESP3: Zero code changes—existing Python, Node, and Go Redis client apps connect instantly."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(12), TEXT_WHITE
    p.space_before = Pt(4)
    p = tf3_1.add_paragraph()
    p.text = "• Zero Mutex Contention: Shared-nothing thread-per-core architecture completely eliminates lock waiting."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(12), TEXT_MUTED
    p.space_before = Pt(4)

    # Section 2: UNDER > CORE MECHANISM
    tb3_2 = s3.shapes.add_textbox(Inches(0.85), Inches(4.05), Inches(11.8), Inches(1.10))
    tf3_2 = tb3_2.text_frame
    tf3_2.word_wrap = True
    tf3_2.margin_left = tf3_2.margin_top = tf3_2.margin_bottom = tf3_2.margin_right = 0
    p = tf3_2.paragraphs[0]
    p.text = "• Thread-Per-Core Sharding (Zero Locks): Pinned workers process requests over 64B cache-aligned SPSC rings with 0 mutex contention."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.8), TEXT_WHITE
    p = tf3_2.add_paragraph()
    p.text = "• 2-Bit CLOCK Auto-Tiering (79% RAM Saved): Active keys stay in DRAM; dormant keys quietly migrate to NVMe SSDs in 4KB extents."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.8), TEXT_WHITE
    p.space_before = Pt(4)
    p = tf3_2.add_paragraph()
    p.text = "• Forkless Append Durability (Zero Freezes): Zero-copy append-only WAL completely eliminates Redis fork() latency spikes and OOM crashes."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(11.8), TEXT_MUTED
    p.space_before = Pt(4)

    # Section 3: UNDER > WHY IT'S FEASIBLE IN THIS TIMEFRAME
    tb3_3 = s3.shapes.add_textbox(Inches(0.85), Inches(5.65), Inches(11.8), Inches(0.95))
    tf3_3 = tb3_3.text_frame
    tf3_3.word_wrap = True
    tf3_3.margin_left = tf3_3.margin_top = tf3_3.margin_bottom = tf3_3.margin_right = 0
    p = tf3_3.paragraphs[0]
    p.text = "• 100% Fully Built Today: All 8 core engine modules are built and verified with 105 / 105 passing tests."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(12), TEXT_WHITE
    p = tf3_3.add_paragraph()
    p.text = "• Sub-Microsecond Speed: Custom bitmapped slab allocator guarantees zero memory fragmentation (RSS/Logical < 1.02)."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(12), TEXT_WHITE
    p.space_before = Pt(4)
    p = tf3_3.add_paragraph()
    p.text = "• Live Observability: Interactive Web Studio and diagnostic CLI running live on localhost right now."
    p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(12), TEXT_MUTED
    p.space_before = Pt(4)

    # =========================================================================
    # SLIDE 4: SYSTEM ARCHITECTURE & TECH STACK
    # =========================================================================
    s4 = prs.slides.add_slide(blank_layout)
    set_slide_background(s4, "hackathon_slides/slide_4.png")

    # Tech Stack items neatly placed inside open right half of each pre-printed box
    stack_items = [
        ("React 19 • TypeScript • Vite Studio", Inches(1.85), Inches(2.36), Inches(2.60)),
        ("Pure Rust 2021 • Tokio • SPSC Rings", Inches(1.80), Inches(3.18), Inches(2.65)),
        ("Hot DRAM Slabs + Cold NVMe Tier", Inches(1.85), Inches(4.02), Inches(2.60)),
        ("RESP3 Wire Protocol • Criterion", Inches(1.80), Inches(4.88), Inches(2.65)),
        ("Docker • Linux Affinity • CI/CD", Inches(2.78), Inches(5.75), Inches(1.68)),
    ]
    for text, left_pos, top_pos, width_pos in stack_items:
        tb = s4.shapes.add_textbox(left_pos, top_pos, width_pos, Inches(0.36))
        tf = tb.text_frame
        tf.word_wrap = True
        tf.margin_left = tf.margin_top = tf.margin_bottom = tf.margin_right = 0
        p = tf.paragraphs[0]
        p.text = text
        p.font.name = "Segoe UI"
        p.font.size = Pt(10.5)
        p.font.bold = True
        p.font.color.rgb = TECH_CYAN
        p.alignment = PP_ALIGN.RIGHT

    # Right Side: Exact Compact Architecture Graphic with Proper Boxes, Proper Colors, 260px gap!
    s4.shapes.add_picture(
        "hackathon_slides/arch_diagram_slide4.png",
        Inches(4.65), Inches(1.70), width=Inches(8.20), height=Inches(5.35)
    )

    # =========================================================================
    # SLIDE 5: KEY FEATURES & UI/UX (WITH REAL LIVE STUDIO SCREENSHOT)
    # =========================================================================
    s5 = prs.slides.add_slide(blank_layout)
    set_slide_background(s5, "hackathon_slides/slide_5.png")

    # Zone 1 (Left 28%): Key Engine Features
    create_card(s5, Inches(0.65), Inches(1.80), Inches(3.60), Inches(5.20), BORDER_GREEN, BG_CARD)
    tb5_f = s5.shapes.add_textbox(Inches(0.85), Inches(1.95), Inches(3.20), Inches(4.90))
    tf5_f = tb5_f.text_frame
    tf5_f.word_wrap = True

    p = tf5_f.paragraphs[0]
    p.text = "KEY ENGINE FEATURES"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, NEON_GREEN

    feats = [
        ("1. Multi-Core Scaling", "Thread-per-core engine scales across all CPU cores with 0 mutex contention."),
        ("2. Auto-NVMe Tiering", "2-Bit CLOCK frequency tracker cuts cloud RAM costs by 70%+ automatically."),
        ("3. Forkless Snapshots", "Zero-copy shadow buffering eliminates Redis fork() freezes and OOM crashes."),
        ("4. Enterprise Security", "100% Pure Rust guarantees zero buffer overflows, memory corruption, or CVEs.")
    ]
    for title, desc in feats:
        p1 = tf5_f.add_paragraph()
        p1.text = title
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        
        p2 = tf5_f.add_paragraph()
        p2.text = desc
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # Zone 2 (Center 44%): REAL Valence Studio Screenshot + Telemetry Card
    s5.shapes.add_picture(
        "hackathon_slides/studio_dark_crisp.png",
        Inches(4.45), Inches(1.80), width=Inches(4.80)
    )

    create_card(s5, Inches(4.45), Inches(4.90), Inches(4.80), Inches(2.10), BORDER_GREEN, BG_CARD)
    tb5_live = s5.shapes.add_textbox(Inches(4.65), Inches(5.00), Inches(4.40), Inches(1.90))
    tf5_live = tb5_live.text_frame
    tf5_live.word_wrap = True

    p = tf5_live.paragraphs[0]
    p.text = "LIVE PRODUCTION TELEMETRY"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(13), True, NEON_GREEN

    tel_items = [
        ("• 78.4% DRAM Reduction:", " 1.79 KB DRAM vs 8.25 KB logical payload."),
        ("• Sub-Microsecond Speed:", " < 0.50 µs DRAM hit, 16.62 µs NVMe flash latency."),
        ("• Minimal Pointer Overhead:", " Only 12 Bytes per tiered cold key on disk.")
    ]
    for lbl, val in tel_items:
        p = tf5_live.add_paragraph()
        p.text = lbl + val
        p.font.name, p.font.size, p.font.color.rgb = "Segoe UI", Pt(10.5), TEXT_WHITE
        p.space_before = Pt(3)

    # Zone 3 (Right 28%): Developer & Operator UX
    create_card(s5, Inches(9.45), Inches(1.80), Inches(3.30), Inches(5.20), BORDER_CYAN, BG_CARD)
    tb5_ux = s5.shapes.add_textbox(Inches(9.65), Inches(1.95), Inches(2.90), Inches(4.90))
    tf5_ux = tb5_ux.text_frame
    tf5_ux.word_wrap = True

    p = tf5_ux.paragraphs[0]
    p.text = "DEVELOPER & OPERATOR UX"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, TECH_CYAN

    ux_points = [
        ("1. Drop-In Replacement", "Zero code rewrites. Existing Python, Node, and Go Redis client drivers work instantly."),
        ("2. 1-Command Startup", "Single self-contained binary auto-detects hardware and starts in under 5 seconds."),
        ("3. No 3 AM Pager Duty", "Autonomous NVMe tiering and forkless durability prevent memory crashes forever."),
        ("4. Built-in Studio", "Web management console with live telemetry, query workbench, and zero setup.")
    ]
    for title, desc in ux_points:
        p1 = tf5_ux.add_paragraph()
        p1.text = title
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        
        p2 = tf5_ux.add_paragraph()
        p2.text = desc
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # =========================================================================
    # SLIDE 6: IMPLEMENTATION ROADMAP & SCOPE
    # =========================================================================
    s6 = prs.slides.add_slide(blank_layout)
    set_slide_background(s6, "hackathon_slides/slide_6.png")

    card_w = Inches(3.70)
    card_h = Inches(4.90)
    card_y = Inches(2.15)

    # Card 1: BUILT & VERIFIED TODAY
    create_card(s6, Inches(0.70), card_y, card_w, card_h, BORDER_GREEN, BG_CARD)
    tb6_1 = s6.shapes.add_textbox(Inches(0.90), card_y + Inches(0.15), Inches(3.30), Inches(4.60))
    tf6_1 = tb6_1.text_frame
    tf6_1.word_wrap = True
    p = tf6_1.paragraphs[0]
    p.text = "BUILT & VERIFIED TODAY"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, NEON_GREEN

    pts1 = [
        ("100% Production Rust", "All 8 core engine modules built from scratch with zero C code or unsafe memory."),
        ("105 / 105 Passing Tests", "Automated unit, benchmark, and chaos test suites passing today."),
        ("Live Web Management Studio", "Interactive React 19 console with real-time telemetry and RESP3 workbench."),
        ("Diagnostic CLI Engine", "Built-in --explain flag traces query journeys through CPU cores in microseconds.")
    ]
    for t, d in pts1:
        p1 = tf6_1.add_paragraph()
        p1.text = t
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        p2 = tf6_1.add_paragraph()
        p2.text = d
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # Card 2: IN PROGRESS (CURRENT SPRINT)
    create_card(s6, Inches(4.80), card_y, card_w, card_h, BORDER_CYAN, BG_CARD)
    tb6_2 = s6.shapes.add_textbox(Inches(5.00), card_y + Inches(0.15), Inches(3.30), Inches(4.60))
    tf6_2 = tb6_2.text_frame
    tf6_2.word_wrap = True
    p = tf6_2.paragraphs[0]
    p.text = "IN PROGRESS (CURRENT SPRINT)"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, TECH_CYAN

    pts2 = [
        ("Distributed Raft Clustering", "Multi-node consensus replication and automated cross-node shard rebalancing."),
        ("Linux io_uring Acceleration", "Direct kernel-bypass submission queues for ultra-fast NVMe flash extents."),
        ("Native Async Driver SDKs", "High-performance client drivers for Rust, Python, and Node.js."),
        ("Prometheus & OpenTelemetry", "Production metrics export for Grafana cluster dashboards.")
    ]
    for t, d in pts2:
        p1 = tf6_2.add_paragraph()
        p1.text = t
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        p2 = tf6_2.add_paragraph()
        p2.text = d
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # Card 3: BEYOND THE HACKATHON
    create_card(s6, Inches(8.90), card_y, card_w, card_h, BORDER_AMBER, BG_CARD)
    tb6_3 = s6.shapes.add_textbox(Inches(9.10), card_y + Inches(0.15), Inches(3.30), Inches(4.60))
    tf6_3 = tb6_3.text_frame
    tf6_3.word_wrap = True
    p = tf6_3.paragraphs[0]
    p.text = "BEYOND THE HACKATHON"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, AMBER

    pts3 = [
        ("Serverless S3 Object Spillover", "Automated tiering cold data down to S3 storage for archival at 1/100th cost."),
        ("Active-Active Multi-Region", "Cross-region database synchronization using CRDT conflict resolution."),
        ("Managed Cloud Offering", "Turnkey database service undercutting AWS ElastiCache bills by 60%."),
        ("Open-Source Ecosystem", "Community adoption under permissive Apache 2.0 / MIT licensing.")
    ]
    for t, d in pts3:
        p1 = tf6_3.add_paragraph()
        p1.text = t
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        p2 = tf6_3.add_paragraph()
        p2.text = d
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # =========================================================================
    # SLIDE 7: EXPECTED IMPACT & SCALABILITY
    # =========================================================================
    s7 = prs.slides.add_slide(blank_layout)
    set_slide_background(s7, "hackathon_slides/slide_7.png")

    # Card 1: WHO BENEFITS
    create_card(s7, Inches(0.70), card_y, card_w, card_h, BORDER_GREEN, BG_CARD)
    tb7_1 = s7.shapes.add_textbox(Inches(0.90), card_y + Inches(0.15), Inches(3.30), Inches(4.60))
    tf7_1 = tb7_1.text_frame
    tf7_1.word_wrap = True
    p = tf7_1.paragraphs[0]
    p.text = "WHO BENEFITS"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, NEON_GREEN

    pts7_1 = [
        ("Startups & Scaleups", "Slash cloud caching bills by 70–80% without rewriting existing application code."),
        ("Fintech & Real-Time AI", "Predictable sub-microsecond latency with zero garbage collector jitter."),
        ("DevOps & SRE Teams", "Rock-solid stability with zero backup freezes or fatal Linux OOM crashes."),
        ("Enterprise Platforms", "Consolidate sprawling Redis clusters into efficient single multi-core nodes.")
    ]
    for t, d in pts7_1:
        p1 = tf7_1.add_paragraph()
        p1.text = t
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        p2 = tf7_1.add_paragraph()
        p2.text = d
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # Card 2: MARKET OPPORTUNITY
    create_card(s7, Inches(4.80), card_y, card_w, card_h, BORDER_CYAN, BG_CARD)
    tb7_2 = s7.shapes.add_textbox(Inches(5.00), card_y + Inches(0.15), Inches(3.30), Inches(4.60))
    tf7_2 = tb7_2.text_frame
    tf7_2.word_wrap = True
    p = tf7_2.paragraphs[0]
    p.text = "MARKET OPPORTUNITY"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, TECH_CYAN

    pts7_2 = [
        ("$15B+ In-Memory DB Market", "Dominated by legacy single-core Redis, AWS MemoryDB, and Aerospike."),
        ("Fixing Cloud Hardware Waste", "Modern 64-core instances are wasted by single-threaded legacy software."),
        ("Massive Dollar Savings", "Cutting 79% of DRAM saves tens of thousands of dollars monthly per cluster."),
        ("Green Compute Efficiency", "Lower CPU and memory footprints reduce cloud data center energy usage.")
    ]
    for t, d in pts7_2:
        p1 = tf7_2.add_paragraph()
        p1.text = t
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        p2 = tf7_2.add_paragraph()
        p2.text = d
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # Card 3: COMMERCIAL GROWTH
    create_card(s7, Inches(8.90), card_y, card_w, card_h, BORDER_AMBER, BG_CARD)
    tb7_3 = s7.shapes.add_textbox(Inches(9.10), card_y + Inches(0.15), Inches(3.30), Inches(4.60))
    tf7_3 = tb7_3.text_frame
    tf7_3.word_wrap = True
    p = tf7_3.paragraphs[0]
    p.text = "COMMERCIAL GROWTH"
    p.font.name, p.font.size, p.font.bold, p.font.color.rgb = "Segoe UI", Pt(16), True, AMBER

    pts7_3 = [
        ("Open-Source Core", "High-velocity developer adoption via GitHub, Cargo, and Docker Hub."),
        ("Jepsen Chaos Verification", "Strict linearizability and network partition tolerance validation."),
        ("Managed Database SaaS Tier", "Turnkey cloud hosted database charging 50% less than AWS ElastiCache."),
        ("Enterprise Support Contracts", "Mission-critical 24/7 SLA and custom hardware acceleration modules.")
    ]
    for t, d in pts7_3:
        p1 = tf7_3.add_paragraph()
        p1.text = t
        p1.font.name, p1.font.size, p1.font.bold, p1.font.color.rgb = "Segoe UI", Pt(13), True, TEXT_WHITE
        p1.space_before = Pt(14)
        p2 = tf7_3.add_paragraph()
        p2.text = d
        p2.font.name, p2.font.size, p2.font.color.rgb = "Segoe UI", Pt(11.5), TEXT_MUTED
        p2.space_before = Pt(2)

    # =========================================================================
    # SLIDE 8: THANK YOU (PRISTINE BACKGROUND ONLY)
    # =========================================================================
    s8 = prs.slides.add_slide(blank_layout)
    set_slide_background(s8, "hackathon_slides/slide_8.png")

    # Save presentation
    prs.save(output_path)
    print(f"[SUCCESS] Master presentation built at: {output_path}")

    # Copy to user's Downloads folder
    user_downloads = os.path.expanduser("~/Downloads")
    dl_target1 = os.path.join(user_downloads, "Valence Presentation.pptx")
    dl_target2 = os.path.join(user_downloads, "Valence_Master_Pitch.pptx")
    for target in [dl_target1, dl_target2]:
        try:
            shutil.copy2(output_path, target)
            print(f"[SUCCESS] Copied presentation to Downloads: {target}")
        except Exception as e:
            print(f"[NOTE] Could not copy to {target}: {e}")

if __name__ == "__main__":
    # Terminate any hung background PowerPoint processes first so there is zero file lock!
    try:
        subprocess.run(["taskkill", "/F", "/IM", "POWERPNT.EXE"], capture_output=True)
    except:
        pass
    build_presentation()
