//! Project Fysarum: Storage Fabric - Zero-Copy Arena
//!
//! This module maps continuous NVMe physical storage directly into 
//! topological mathematical structures (Simplices) without any 
//! serialization or deserialization overhead.

use bytemuck::{Pod, Zeroable};
use memmap2::{MmapMut, MmapOptions};
use std::fs::OpenOptions;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Alignment Error: Storage file size is not a multiple of the Simplex size")]
    AlignmentError,
}

/// A 0-Simplex represents a single data vertex in our topological graph.
/// 
/// #[repr(C)] guarantees that the Rust compiler will layout this struct in memory 
/// exactly as defined, with no hidden padding. This is critical because it allows 
/// us to cast raw NVMe bytes directly into this struct instantly.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct VertexSimplex {
    /// A unique 64-bit identifier (e.g., mapped via a Hilbert Curve)
    pub entity_id: u64,
    /// A localized vector of state. To keep this strictly bare-metal, 
    /// we avoid heap-allocated `Vec<u8>` and use fixed-size arrays.
    pub state_vector: [f64; 4], 
    /// A Lamport timestamp or CRDT sequence number for lockless merging
    pub logical_clock: u64,
}

/// The SimplexArena is a continuous memory-mapped view of the disk.
/// The OS kernel handles paging data in and out of RAM automatically.
pub struct SimplexArena {
    /// The raw memory map tied to the underlying file descriptor
    mmap: MmapMut,
    /// The maximum number of simplices this arena can hold
    capacity: usize,
}

impl SimplexArena {
    /// Opens or creates a memory-mapped arena backed by a physical file.
    pub fn new<P: AsRef<Path>>(path: P, initial_capacity: usize) -> Result<Self, StorageError> {
        let file_size = (initial_capacity * std::mem::size_of::<VertexSimplex>()) as u64;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Pre-allocate the physical file size on disk
        file.set_len(file_size)?;

        // Map the file directly into the virtual address space of our Rust process
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        Ok(Self {
            mmap,
            capacity: initial_capacity,
        })
    }

    /// Zero-cost cast from the raw memory map into a slice of Simplices.
    /// 
    /// This is where the magic happens. We don't parse JSON, Protobuf, or Parquet.
    /// We tell the CPU: "Look at these bytes and treat them as an array of structs."
    /// It executes in O(1) time, compiling down to a single pointer cast.
    pub fn as_simplices(&self) -> Result<&[VertexSimplex], StorageError> {
        bytemuck::try_cast_slice(self.mmap.as_ref()).map_err(|_| StorageError::AlignmentError)
    }

    /// Zero-cost mutable cast for lockless CRDT state updates.
    pub fn as_simplices_mut(&mut self) -> Result<&mut [VertexSimplex], StorageError> {
        bytemuck::try_cast_slice_mut(self.mmap.as_mut())
            .map_err(|_| StorageError::AlignmentError)
    }

    /// Read a specific simplex via index. O(1) memory access.
    pub fn get(&self, index: usize) -> Result<Option<&VertexSimplex>, StorageError> {
        if index >= self.capacity {
            return Ok(None);
        }
        let simplices = self.as_simplices()?;
        Ok(Some(&simplices[index]))
    }
}