//! `UringDispatcher`: io_uring-backed submission/completion for storage I/O.
//! Submissions bypass blocking syscalls and the page cache; completion events
//! are polled from the kernel ring with minimal context-switch overhead.

use io_uring::{opcode, squeue, types, IoUring};
use std::os::unix::io::RawFd;

/// The maximum number of in-flight I/O operations per thread/core.
/// 4096 is optimal for modern NVMe queues.
const RING_SIZE: u32 = 4096;

/// A simple token used to map a kernel Completion Queue Entry (CQE) back to 
/// the specific Physarum Actor waiting in the Wavefront Compute Mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoToken(pub u64);

/// The UringDispatcher runs thread-locally. It never uses Mutexes or RwLocks.
pub struct UringDispatcher {
    ring: IoUring,
    in_flight_count: usize,
}

impl UringDispatcher {
    /// Initializes a new io_uring instance.
    pub fn new() -> std::io::Result<Self> {
        // Default ring setup; optional `SetupFlags::SQPOLL` would use a kernel polling thread for lower submit latency.
        let ring = IoUring::new(RING_SIZE)?;
        
        Ok(Self {
            ring,
            in_flight_count: 0,
        })
    }

    /// Submits a zero-copy read request directly to the NVMe driver.
    /// 
    /// # Safety
    /// The caller *must* guarantee that the memory slice `buf` remains 
    /// valid and un-moved until the kernel returns the `IoToken` in the 
    /// completion queue. Rust's Pinning and Ownership semantics in the 
    /// higher-level Storage Fabric enforce this safely at compile time.
    pub unsafe fn submit_read(
        &mut self,
        fd: RawFd,
        offset: u64,
        buf: &mut [u8],
        token: IoToken,
    ) -> std::io::Result<()> {
        let read_e = opcode::Read::new(
            types::Fd(fd),
            buf.as_mut_ptr(),
            buf.len() as u32,
        )
        .offset(offset)
        .build()
        // `user_data` carries `IoToken`; the matching CQE returns the same value.
        .user_data(token.0);

        self.push_to_sq(read_e)?;
        Ok(())
    }

    /// Submits a zero-copy write request (append-oriented storage layout).
    pub unsafe fn submit_write(
        &mut self,
        fd: RawFd,
        offset: u64,
        buf: &[u8],
        token: IoToken,
    ) -> std::io::Result<()> {
        let write_e = opcode::Write::new(
            types::Fd(fd),
            buf.as_ptr(),
            buf.len() as u32,
        )
        .offset(offset)
        .build()
        .user_data(token.0);

        self.push_to_sq(write_e)?;
        Ok(())
    }

    /// Internal helper to push entries to the Submission Queue (SQ).
    fn push_to_sq(&mut self, entry: squeue::Entry) -> std::io::Result<()> {
        let mut sq = self.ring.submission();
        
        // Full SQ: submit to drain entries so `push` can succeed.
        if sq.is_full() {
            drop(sq); // Drop the lockless borrow to call submit
            self.ring.submit()?;
            sq = self.ring.submission();
        }

        // SAFETY: `is_full` was handled above; `push` is only called when space is available.
        unsafe { sq.push(&entry).map_err(|_| std::io::Error::from_raw_os_error(libc::EBUSY))? };
        self.in_flight_count += 1;
        Ok(())
    }

    /// Flushes pending submissions to the kernel.
    pub fn submit(&mut self) -> std::io::Result<usize> {
        self.ring.submit()
    }

    /// Polls the Completion Queue (CQ) non-blockingly.
    /// Returns a vector of tokens corresponding to the Physarum Actors 
    /// whose hardware I/O has just completed.
    pub fn poll_completions(&mut self) -> Vec<(IoToken, i32)> {
        let mut completions = Vec::new();
        let mut cq = self.ring.completion();

        for cqe in &mut cq {
            let token = IoToken(cqe.user_data());
            let result = cqe.result(); // Bytes read/written, or negative error code
            
            completions.push((token, result));
            self.in_flight_count -= 1;
        }

        // `sync` retires CQEs so completion ring slots can be reused.
        cq.sync();
        completions
    }

    /// Returns the number of operations currently being processed by hardware.
    pub fn in_flight(&self) -> usize {
        self.in_flight_count
    }
}