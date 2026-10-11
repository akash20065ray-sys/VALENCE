import os
from pptx import Presentation
from pptx.util import Inches, Pt
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.enum.shapes import MSO_SHAPE

def build_perfect_vector_deck(output_path="Valence_CodeApex_Official.pptx"):
    prs = Presentation()
    prs.slide_width = Inches(13.333)
    prs.slide_height = Inches(7.5)
    blank_layout = prs.slide_layouts[6]

    # Clean Cyber Theme Colors
    NEON_GREEN   = RGBColor(34, 197, 94)    # #22C55E
    TECH_CYAN    = RGBColor(6, 182, 212)    # #06B6D4
    TEXT_WHITE   = RGBColor(248, 250, 252)  # #F8FAFC
    TEXT_MUTED   = RGBColor(148, 163, 184)  # #94A3B8
    TEXT_DIM     = RGBColor(100, 116, 139)  # #64748B
    BG_CARD      = RGBColor(11, 23, 18)     # Dark Emerald Tint
    BG_WINDOW    = RGBColor(9, 18, 14)      # Dark Console Slate
    BORDER_GREEN = RGBColor(22, 101, 52)    # Emerald Border
    BORDER_CYAN  = RGBColor(14, 116, 144)   # Cyan Border
    CARD_BORDER  = BORDER_GREEN
    CARD_BG      = BG_CARD

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
    # SLIDE 1: TITLE & TEAM DETAILS
    # =========================================================================
    s1 = prs.slides.add_slide(blank_layout)
    set_slide_background(s1, "hackathon_slides/slide_1.png")

    tb_title = s1.shapes.add_textbox(Inches(0.8), Inches(2.1), Inches(11.733), Inches(1.4))
    tf_title = tb_title.text_frame
    tf_title.word_wrap = True

    p_t = tf_title.paragraphs[0]
    p_t.text = "VALENCE"
    p_t.font.name = "Segoe UI"
    p_t.font.size = Pt(40)
    p_t.font.bold = True
    p_t.font.color.rgb = NEON_GREEN
    p_t.alignment = PP_ALIGN.CENTER

    p_t_sub = tf_title.add_paragraph()
    p_t_sub.text = "A Multi-Core, Cost-Effective Replacement for Redis — Built in Pure Rust"
    p_t_sub.font.name = "Segoe UI"
    p_t_sub.font.size = Pt(18)
    p_t_sub.font.bold = True
    p_t_sub.font.color.rgb = TECH_CYAN
    p_t_sub.alignment = PP_ALIGN.CENTER
    p_t_sub.space_before = Pt(4)

    p_t_tag = tf_title.add_paragraph()
    p_t_tag.text = "Uses all CPU cores • Cuts RAM bills by 79% via NVMe SSDs • Zero crash risk during backups"
    p_t_tag.font.name = "Segoe UI"
    p_t_tag.font.size = Pt(13)
    p_t_tag.font.color.rgb = TEXT_WHITE
    p_t_tag.alignment = PP_ALIGN.CENTER
    p_t_tag.space_before = Pt(4)

    # Team Box Content
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
    # SLIDE 2: PROBLEM STATEMENT & TARGET NEED
    # =========================================================================
    s2 = prs.slides.add_slide(blank_layout)
    set_slide_background(s2, "hackathon_slides/slide_2.png")

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
    # SLIDE 3: PROPOSED SOLUTION & INNOVATION
    # =========================================================================
    s3 = prs.slides.add_slide(blank_layout)
    set_slide_background(s3, "hackathon_slides/slide_3.png")

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
    # SLIDE 4: SYSTEM ARCHITECTURE (Clean, Bold, Razor-Sharp Vector Blocks!)
    # =========================================================================
    s4 = prs.slides.add_slide(blank_layout)
    set_slide_background(s4, "hackathon_slides/slide_4.png")

    # Left Tech Stack boxes (Clean text inside template boxes)
    stack_content = [
        (Inches(0.55), Inches(2.28), Inches(3.6), Inches(0.58), "React 19, TypeScript, Vite\nCustom Vanilla CSS Design System"),
        (Inches(0.55), Inches(3.08), Inches(3.6), Inches(0.58), "Pure Rust 2021, Tokio Async I/O\nLock-Free SPSC Rings, AVX2 SIMD"),
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

    # Right side: Crisp, Vector-Based Architecture Flow (NEVER BLURRY!)
    # We build 4 large, clean, bold pipeline cards arranged in sequence
    rx = Inches(4.7)
    rw = Inches(8.0)

    # Flow Title
    tb_flow = s4.shapes.add_textbox(rx, Inches(1.9), rw, Inches(0.4))
    p_fl = tb_flow.text_frame.paragraphs[0]
    p_fl.text = "HOW A REQUEST FLOWS THROUGH VALENCE (SUB-MICROSECOND PIPELINE)"
    p_fl.font.name = "Segoe UI"
    p_fl.font.size = Pt(12)
    p_fl.font.bold = True
    p_fl.font.color.rgb = TECH_CYAN

    # Step 1: Wire Ingress & Vector Parser
    c1 = create_card(s4, rx, Inches(2.35), rw, Inches(1.0), BORDER_GREEN)
    tb1 = s4.shapes.add_textbox(rx + Inches(0.2), Inches(2.42), rw - Inches(0.4), Inches(0.85))
    tf1 = tb1.text_frame
    p1 = tf1.paragraphs[0]
    p1.text = "1. CLIENT INGRESS & AVX2 SIMD PARSER  •  Latency: ~0.08 µs"
    p1.font.name = "Segoe UI"
    p1.font.size = Pt(13)
    p1.font.bold = True
    p1.font.color.rgb = NEON_GREEN
    p1_sub = tf1.add_paragraph()
    p1_sub.text = "Client sends Redis command over TCP (Port 6379). AVX2 256-bit vector instructions scan 32 bytes per cycle at 289M ops/sec with zero copy."
    p1_sub.font.name = "Segoe UI"
    p1_sub.font.size = Pt(11)
    p1_sub.font.color.rgb = TEXT_WHITE
    p1_sub.space_before = Pt(2)

    # Step 2: Thread-per-Core Shards
    c2 = create_card(s4, rx, Inches(3.48), rw, Inches(1.0), BORDER_CYAN)
    tb2 = s4.shapes.add_textbox(rx + Inches(0.2), Inches(3.55), rw - Inches(0.4), Inches(0.85))
    tf2 = tb2.text_frame
    p2 = tf2.paragraphs[0]
    p2.text = "2. JUMP CONSISTENT ROUTING & HARDWARE CORE PINNING  •  Latency: ~0.15 µs"
    p2.font.name = "Segoe UI"
    p2.font.size = Pt(13)
    p2.font.bold = True
    p2.font.color.rgb = TECH_CYAN
    p2_sub = tf2.add_paragraph()
    p2_sub.text = "Key is hashed in O(ln N) and dispatched to dedicated CPU core via 64-byte cache-padded SPSC ring queue. Zero mutex locks, zero false sharing."
    p2_sub.font.name = "Segoe UI"
    p2_sub.font.size = Pt(11)
    p2_sub.font.color.rgb = TEXT_WHITE
    p2_sub.space_before = Pt(2)

    # Step 3: Robin Hood Flat Index
    c3 = create_card(s4, rx, Inches(4.61), rw, Inches(1.0), BORDER_GREEN)
    tb3 = s4.shapes.add_textbox(rx + Inches(0.2), Inches(4.68), rw - Inches(0.4), Inches(0.85))
    tf3 = tb3.text_frame
    p3 = tf3.paragraphs[0]
    p3.text = "3. FLAT ROBIN HOOD HASH MAP (ZERO TOMBSTONES)  •  Latency: ~0.22 µs"
    p3.font.name = "Segoe UI"
    p3.font.size = Pt(13)
    p3.font.bold = True
    p3.font.color.rgb = NEON_GREEN
    p3_sub = tf3.add_paragraph()
    p3_sub.text = "Continuous memory array with Distance-from-Ideal-Bucket (DIB) probing (1-2 hops). Backward-shift deletion permanently eliminates dead tombstone slots."
    p3_sub.font.name = "Segoe UI"
    p3_sub.font.size = Pt(11)
    p3_sub.font.color.rgb = TEXT_WHITE
    p3_sub.space_before = Pt(2)

    # Step 4: Dual Storage Tiering
    c4 = create_card(s4, rx, Inches(5.74), rw, Inches(1.15), BORDER_CYAN)
    tb4 = s4.shapes.add_textbox(rx + Inches(0.2), Inches(5.81), rw - Inches(0.4), Inches(1.0))
    tf4 = tb4.text_frame
    p4 = tf4.paragraphs[0]
    p4.text = "4. DUAL STORAGE HIERARCHY: HOT RAM vs. COLD NVMe  •  79.1% DRAM Cost Cut"
    p4.font.name = "Segoe UI"
    p4.font.size = Pt(13)
    p4.font.bold = True
    p4.font.color.rgb = TECH_CYAN
    p4_sub = tf4.add_paragraph()
    p4_sub.text = "• HOT DATA: Served instantly from 2MB hugepage slab arena (1.19ns allocation via TZCNT, RSS/Logical < 1.02).\n• COLD DATA: 2-bit CLOCK sweeps untouched keys to 4KB NVMe blocks with 12B pointer. Promotes to RAM on read."
    p4_sub.font.name = "Segoe UI"
    p4_sub.font.size = Pt(10.5)
    p4_sub.font.color.rgb = TEXT_WHITE
    p4_sub.space_before = Pt(2)

    # =========================================================================
    # SLIDE 5: KEY FEATURES & UI/UX (Clean 50/50 Vector Layout!)
    # =========================================================================
    s5 = prs.slides.add_slide(blank_layout)
    set_slide_background(s5, "hackathon_slides/slide_5.png")

    # LEFT 50%: Clean Vector Mockup of Valence Studio Interface
    ui_x = Inches(0.8)
    ui_y = Inches(2.1)
    ui_w = Inches(5.8)
    ui_h = Inches(4.8)

    # Window Card
    create_card(s5, ui_x, ui_y, ui_w, ui_h, BORDER_GREEN, BG_WINDOW)

    # Title Bar
    tb_w = s5.shapes.add_textbox(ui_x + Inches(0.2), ui_y + Inches(0.12), ui_w - Inches(0.4), Inches(0.4))
    tf_w = tb_w.text_frame
    p_w = tf_w.paragraphs[0]
    p_w.text = "VALENCE STUDIO • 127.0.0.1:6379 (ONLINE) • 4 CORES PINNED"
    p_w.font.name = "Segoe UI"
    p_w.font.size = Pt(11)
    p_w.font.bold = True
    p_w.font.color.rgb = NEON_GREEN

    # Sub-panels inside the UI Hero
    # Keys Table preview
    tb_ui_content = s5.shapes.add_textbox(ui_x + Inches(0.2), ui_y + Inches(0.55), ui_w - Inches(0.4), Inches(4.1))
    tf_uic = tb_ui_content.text_frame
    tf_uic.word_wrap = True

    p = tf_uic.paragraphs[0]
    p.text = "LIVE KEYSPACE BROWSER (NO LOCKS / NO CRASHES):"
    p.font.name = "Segoe UI"
    p.font.size = Pt(11)
    p.font.bold = True
    p.font.color.rgb = TECH_CYAN

    ui_keys_data = [
        "📁 user:*     user:session:100     [256 B]  [TTL: 3,412s]  HOT DRAM",
        "📁 user:*     user:profile:alex    [184 B]  [PERSISTENT]   HOT DRAM",
        "📁 auth:*     auth:jwt:token_90    [512 B]  [TTL: 840s]    HOT DRAM",
        "📁 reports:*  model:weights:cache  [4096B]  [COLD (NVMe)]  79% CUT",
    ]
    for uk in ui_keys_data:
        p_k = tf_uic.add_paragraph()
        p_k.text = uk
        p_k.font.name = "Consolas"
        p_k.font.size = Pt(9.5)
        p_k.font.color.rgb = TEXT_WHITE
        p_k.space_before = Pt(3)

    p_inspect = tf_uic.add_paragraph()
    p_inspect.text = "\nRECORD DETAIL INSPECTION (JSON & HEX DUMP):"
    p_inspect.font.name = "Segoe UI"
    p_inspect.font.size = Pt(11)
    p_inspect.font.bold = True
    p_inspect.font.color.rgb = TECH_CYAN

    p_json = tf_uic.add_paragraph()
    p_json.text = '{\n  "userId": 100,\n  "role": "admin",\n  "session": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",\n  "allocated_tier": "TIER-0 HOT (Slab Pool #4)"\n}'
    p_json.font.name = "Consolas"
    p_json.font.size = Pt(9.5)
    p_json.font.color.rgb = NEON_GREEN
    p_json.space_before = Pt(2)

    # RIGHT 50%: 3 BOLD, HUMAN FEATURE CARDS
    fx = Inches(6.9)
    fw = Inches(5.6)
    fh = Inches(1.48)

    # Card 1: Safe Data Inspection
    create_card(s5, fx, Inches(2.1), fw, fh, BORDER_GREEN)
    tb_f1 = s5.shapes.add_textbox(fx + Inches(0.2), Inches(2.18), fw - Inches(0.4), fh - Inches(0.3))
    tf_f1 = tb_f1.text_frame
    p_f1 = tf_f1.paragraphs[0]
    p_f1.text = "1. SAFE ZERO-FREEZE DATA INSPECTOR"
    p_f1.font.name = "Segoe UI"
    p_f1.font.size = Pt(13)
    p_f1.font.bold = True
    p_f1.font.color.rgb = NEON_GREEN
    p_f1_desc = tf_f1.add_paragraph()
    p_f1_desc.text = "In production Redis, typing 'KEYS *' freezes customer traffic. Valence Studio lets developers browse keys, edit JSON values, and inspect hex memory offsets with zero lock contention."
    p_f1_desc.font.name = "Segoe UI"
    p_f1_desc.font.size = Pt(11)
    p_f1_desc.font.color.rgb = TEXT_WHITE
    p_f1_desc.space_before = Pt(4)

    # Card 2: 79% NVMe Cost Cut
    create_card(s5, fx, Inches(3.72), fw, fh, BORDER_CYAN)
    tb_f2 = s5.shapes.add_textbox(fx + Inches(0.2), Inches(3.8), fw - Inches(0.4), fh - Inches(0.3))
    tf_f2 = tb_f2.text_frame
    p_f2 = tf_f2.paragraphs[0]
    p_f2.text = "2. TRANSPARENT 79% NVMe MEMORY COST CUT"
    p_f2.font.name = "Segoe UI"
    p_f2.font.size = Pt(13)
    p_f2.font.bold = True
    p_f2.font.color.rgb = TECH_CYAN
    p_f2_desc = tf_f2.add_paragraph()
    p_f2_desc.text = "The 2-bit CLOCK frequency tracker automatically sweeps cold, untouched keys to fast NVMe SSD storage. Physical DRAM consumption drops by 79.1%, slashing AWS bills with zero code changes."
    p_f2_desc.font.name = "Segoe UI"
    p_f2_desc.font.size = Pt(11)
    p_f2_desc.font.color.rgb = TEXT_WHITE
    p_f2_desc.space_before = Pt(4)

    # Card 3: Query Plan Trace
    create_card(s5, fx, Inches(5.34), fw, fh, BORDER_GREEN)
    tb_f3 = s5.shapes.add_textbox(fx + Inches(0.2), Inches(5.42), fw - Inches(0.4), fh - Inches(0.3))
    tf_f3 = tb_f3.text_frame
    p_f3 = tf_f3.paragraphs[0]
    p_f3.text = "3. MICROSECOND QUERY PLAN TRACE (--explain)"
    p_f3.font.name = "Segoe UI"
    p_f3.font.size = Pt(13)
    p_f3.font.bold = True
    p_f3.font.color.rgb = NEON_GREEN
    p_f3_desc = tf_f3.add_paragraph()
    p_f3_desc.text = "Key-value databases are usually black boxes. Valence lets engineers type '--explain' in the terminal or dashboard to see the exact microsecond journey through SIMD, CPU cores, and storage."
    p_f3_desc.font.name = "Segoe UI"
    p_f3_desc.font.size = Pt(11)
    p_f3_desc.font.color.rgb = TEXT_WHITE
    p_f3_desc.space_before = Pt(4)

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
        create_card(s6, cx, c_y, c_w, c_h, CARD_BORDER, CARD_BG)
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
    # SLIDE 7: EXPECTED IMPACT & SCALABILITY
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
        create_card(s7, cx, c_y, c_w, c_h, CARD_BORDER, CARD_BG)
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

    create_card(s8, Inches(1.5), Inches(4.5), Inches(10.333), Inches(2.3), BORDER_GREEN, CARD_BG)
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

    prs.save(output_path)
    print(f"[SUCCESS] Perfect vector presentation generated at: {output_path}")

    # Copy to Downloads
    downloads_v2 = r"C:\Users\akash\Downloads\Valence_CodeApex_V2.pptx"
    downloads_path = r"C:\Users\akash\Downloads\Valence_CodeApex_Official.pptx"
    import shutil
    shutil.copyfile(output_path, downloads_v2)
    print(f"[SUCCESS] Copied to Downloads as: {downloads_v2}")
    try:
        shutil.copyfile(output_path, downloads_path)
        print(f"[SUCCESS] Updated: {downloads_path}")
    except PermissionError:
        print(f"[INFO] {downloads_path} is currently open in PowerPoint. Saved as {downloads_v2}!")

if __name__ == "__main__":
    build_perfect_vector_deck()
