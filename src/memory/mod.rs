//! Zero-Header Memory Arena & Bitmapped Allocation Primitives
//!
//! Sub-Phase 4: Region-Based Slab Arenas
//!
//! Eliminates heap fragmentation and per-object metadata tax:
//! - 64-bit trailing_zeros() single-cycle allocation bitfields.
//! - 2MB hugepage-aligned slab pools for discrete geometric size classes.
//! - Zero per-object header tax (RSS / Logical RAM < 1.05).

pub mod bitmap;
pub mod slab_arena;

pub use bitmap::{Bitmap64, BitmapError, MultiBitmap};
pub use slab_arena::{
    SlabAllocation, SlabArena, SlabClass, SlabError, SlabStats, CACHELINE_ALIGNMENT,
    DEFAULT_CHUNK_SIZE, HUGEPAGE_ALIGNMENT, SLAB_SIZE_CLASSES,
};
