use fysarum_storage::arena::VertexSimplex;
use std::thread;

/// The Actor trait processes massive chunks of memory at once.
pub trait Actor: Send + Sync {
    fn on_chunk(&mut self, chunk: &[VertexSimplex]);
}

/// The PetriRuntime routes data to the physical cores.
pub struct PetriRuntime;

impl PetriRuntime {
    /// The Wavefront Dispatcher
    /// This instantly slices the zero-copy hard drive map into equal chunks 
    /// and feeds them directly to the CPU cores without copying a single byte.
    pub fn dispatch_wavefront<A: Actor + Clone + Send>(
        core_count: usize,
        actor: A,
        data: &[VertexSimplex],
    ) {
        // Divide the input rows evenly across available worker chunks.
        let chunk_size = (data.len() / core_count).max(1);
        
        // thread::scope allows us to spawn threads that borrow the `data` slice
        // without Rust complaining about memory lifetimes.
        thread::scope(|s| {
            for chunk in data.chunks(chunk_size) {
                let mut local_actor = actor.clone();
                s.spawn(move || {
                    // Drop the robot directly onto its chunk of the memory map
                    local_actor.on_chunk(chunk);
                });
            }
        });
    }
}