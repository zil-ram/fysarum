use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VertexSimplex {
    pub entity_id: u64,
    pub state_vector: [f64; 4],
    pub logical_clock: u64,
}

pub struct SimplexArena {
    mmap_ptr: *mut libc::c_void,
    pub capacity: usize,
    fd: i32,
}

unsafe impl Send for SimplexArena {}
unsafe impl Sync for SimplexArena {}

impl SimplexArena {
    pub fn new<P: AsRef<std::path::Path>>(path: P, capacity: usize) -> std::io::Result<Self> {
        let file_size = capacity * std::mem::size_of::<VertexSimplex>();
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;

        file.set_len(file_size as u64)?;

        let fd = file.as_raw_fd();
        
        let mmap_ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                file_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if mmap_ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self { mmap_ptr, capacity, fd })
    }

    pub fn prefetch_async(&self) {
        let file_size = self.capacity * std::mem::size_of::<VertexSimplex>();
        unsafe {
            libc::madvise(self.mmap_ptr, file_size, libc::MADV_SEQUENTIAL);
            libc::madvise(self.mmap_ptr, file_size, libc::MADV_WILLNEED);
        }
    }

    pub fn as_simplices(&self) -> std::io::Result<&[VertexSimplex]> {
        unsafe { Ok(std::slice::from_raw_parts(self.mmap_ptr as *const VertexSimplex, self.capacity)) }
    }

    pub fn as_simplices_mut(&mut self) -> std::io::Result<&mut [VertexSimplex]> {
        unsafe { Ok(std::slice::from_raw_parts_mut(self.mmap_ptr as *mut VertexSimplex, self.capacity)) }
    }

    /// Exposes the mmap as a mutable byte slice spanning the full arena capacity.
    /// Dynamic layouts write through this view using schema-defined row strides.
    pub fn as_bytes_mut(&mut self) -> std::io::Result<&mut [u8]> {
        let total_bytes = self.capacity * std::mem::size_of::<VertexSimplex>();
        unsafe { Ok(std::slice::from_raw_parts_mut(self.mmap_ptr as *mut u8, total_bytes)) }
    }
}

impl Drop for SimplexArena {
    fn drop(&mut self) {
        let file_size = self.capacity * std::mem::size_of::<VertexSimplex>();
        unsafe { libc::munmap(self.mmap_ptr, file_size); }
    }
}