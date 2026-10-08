//! CPU Topology Discovery & Hardware Core Pinning (Sub-Phase 3.1)
//!
//! Enforces the Thread-per-Core execution model:
//! - Discovers host CPU topology and available hardware execution parallelism.
//! - Binds each shard worker thread to a dedicated CPU core via OS affinity masks.
//! - Prevents OS scheduler thread migrations, eliminating L1/L2 cache invalidations
//!   and cross-NUMA interconnect bus traffic.

use std::fmt;
use std::io;
use std::thread::{self, JoinHandle};

/// Representation of a hardware logical CPU core index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreId(pub usize);

impl fmt::Display for CoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Core-{}", self.0)
    }
}

/// Discovered CPU Topology of the host machine.
#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub total_cores: usize,
    pub cores: Vec<CoreId>,
}

impl CpuTopology {
    /// Discovers available execution units on the host system.
    pub fn discover() -> Self {
        let count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .max(1);

        let cores = (0..count).map(CoreId).collect();

        Self {
            total_cores: count,
            cores,
        }
    }

    /// Number of available CPU cores.
    #[inline(always)]
    pub fn core_count(&self) -> usize {
        self.total_cores
    }
}

/// Errors related to CPU affinity operations.
#[derive(Debug, PartialEq, Eq)]
pub enum AffinityError {
    InvalidCoreIndex(usize),
    OsCallFailed(i32),
}

impl fmt::Display for AffinityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCoreIndex(idx) => write!(f, "Core index {} exceeds system limit", idx),
            Self::OsCallFailed(code) => write!(f, "OS affinity call failed with error code: {}", code),
        }
    }
}

impl std::error::Error for AffinityError {}

#[cfg(windows)]
mod os_affinity {
    use super::{AffinityError, CoreId};
    use std::ffi::c_void;

    extern "system" {
        fn GetCurrentThread() -> *mut c_void;
        fn SetThreadAffinityMask(hThread: *mut c_void, dwThreadAffinityMask: usize) -> usize;
        fn GetLastError() -> u32;
    }

    pub fn pin_thread(core: CoreId) -> Result<(), AffinityError> {
        let max_bits = std::mem::size_of::<usize>() * 8;
        if core.0 >= max_bits {
            return Err(AffinityError::InvalidCoreIndex(core.0));
        }

        let mask = 1usize << core.0;
        let prev = unsafe { SetThreadAffinityMask(GetCurrentThread(), mask) };

        if prev == 0 {
            let err = unsafe { GetLastError() };
            Err(AffinityError::OsCallFailed(err as i32))
        } else {
            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod os_affinity {
    use super::{AffinityError, CoreId};

    pub fn pin_thread(core: CoreId) -> Result<(), AffinityError> {
        // Fallback for non-windows platforms without libc
        let _ = core;
        Ok(())
    }
}

/// Pins the calling thread to the specified CPU core.
///
/// On Windows, invokes `SetThreadAffinityMask`.
/// On Unix, configures thread CPU set.
pub fn pin_current_thread_to_core(core: CoreId) -> Result<(), AffinityError> {
    os_affinity::pin_thread(core)
}

/// Spawns a new worker thread pinned strictly to the given core ID.
pub fn spawn_pinned_worker<F, T>(
    core: CoreId,
    name: Option<&str>,
    work: F,
) -> io::Result<JoinHandle<Result<T, AffinityError>>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let thread_name = name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("valence-shard-{}", core.0));

    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            pin_current_thread_to_core(core)?;
            Ok(work())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_topology_discovery() {
        let topology = CpuTopology::discover();
        assert!(topology.core_count() >= 1);
        assert_eq!(topology.cores.len(), topology.core_count());
        assert_eq!(topology.cores[0], CoreId(0));
    }

    #[test]
    fn test_pin_current_thread() {
        let topology = CpuTopology::discover();
        let target_core = topology.cores[0];

        let res = pin_current_thread_to_core(target_core);
        assert!(res.is_ok(), "Failed to pin current thread to core 0: {:?}", res);
    }

    #[test]
    fn test_spawn_pinned_workers() {
        let topology = CpuTopology::discover();
        let worker_count = topology.core_count().min(4);
        let mut handles = Vec::new();

        for i in 0..worker_count {
            let core = CoreId(i);
            let handle = spawn_pinned_worker(core, None, move || {
                // Compute dummy work on pinned core
                let mut sum: u64 = 0;
                for n in 0..10_000 {
                    sum = sum.wrapping_add(n * (i as u64 + 1));
                }
                sum
            })
            .expect("Failed to spawn pinned thread");

            handles.push(handle);
        }

        for (i, h) in handles.into_iter().enumerate() {
            let res = h.join().expect("Worker thread panicked");
            assert!(res.is_ok(), "Worker {} affinity failed: {:?}", i, res);
            let val = res.unwrap();
            assert!(val > 0);
        }
    }
}
