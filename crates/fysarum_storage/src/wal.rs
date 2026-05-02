use crate::arena::VertexSimplex;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result, Seek, SeekFrom};
use std::mem;

pub struct WriteAheadLog {
    file: File,
}

#[repr(C)]
struct WalEntry {
    index: usize,
    simplex: VertexSimplex,
}

impl WriteAheadLog {
    /// Opens the append-only log file, creating it if it doesn't exist
    pub fn open(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    /// The golden rule of database durability: write to disk before memory.
    pub fn append(&mut self, index: usize, simplex: VertexSimplex) -> Result<()> {
        let entry = WalEntry { index, simplex };
        
        // Reinterpret `WalEntry` (`repr(C)`) as a byte slice for append-only I/O.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &entry as *const WalEntry as *const u8,
                mem::size_of::<WalEntry>(),
            )
        };
        
        self.file.write_all(bytes)?;
        
        // `sync_data` forces the Linux kernel to flush to the physical NVMe drive.
        // It blocks until the hardware confirms the data is safe.
        self.file.sync_data()?; 
        
        Ok(())
    }

    /// Reads the log file from start to finish to rebuild lost memory state
    pub fn recover(&mut self) -> Result<Vec<(usize, VertexSimplex)>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;
        
        let entry_size = mem::size_of::<WalEntry>();
        let mut recoveries = Vec::new();
        
        // Deserialize the raw bytes back into Rust structs
        for chunk in buffer.chunks_exact(entry_size) {
            let entry: WalEntry = unsafe { std::ptr::read(chunk.as_ptr() as *const _) };
            recoveries.push((entry.index, entry.simplex));
        }
        
        Ok(recoveries)
    }
}