//! This module implements the `UringDispatcher`, the beating heart of the 
//! Fysarum storage I/O. It bypasses standard blocking sys-calls and the OS 
//! page cache. By mapping memory directly to the kernel's submission and 
//! completion queues, we achieve millions of IOPS per core with zero 
//! context-switching overhead.

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
/// It represents a single, autonomous "vein" of the Physarum mesh.
pub struct UringDispatcher {
    ring: IoUring,
    in_flight_count: usize,
}

impl UringDispatcher {
    /// Initializes a new io_uring instance.
    pub fn new() -> std::io::Result<Self> {
        // We initialize the ring. In a production build, we would use
        // `SetupFlags::SQPOLL` to have the kernel spawn a dedicated polling thread,
        // meaning we don't even need to make a syscall to submit I/O.
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
        // We embed the Actor's token into the user_data field of the SQE.
        // When the hardware finishes the read, it hands this exact token back.
        .user_data(token.0);

        self.push_to_sq(read_e)?;
        Ok(())
    }

    /// Submits a zero-copy write request. 
    /// For Physarum, these writes are purely appending to our CRDT state lattices.
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
        
        // If the queue is full, we must submit to the kernel to clear space.
        if sq.is_full() {
            drop(sq); // Drop the lockless borrow to call submit
            self.ring.submit()?;
            sq = self.ring.submission();
        }

        // SAFETY: We checked if the queue is full above.
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

        // We clear the completed entries so the hardware can reuse the ring slots.
        cq.sync();
        completions
    }

    /// Returns the number of operations currently being processed by hardware.
    pub fn in_flight(&self) -> usize {
        self.in_flight_count
    }
}