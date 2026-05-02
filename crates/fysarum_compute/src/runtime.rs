use fysarum_storage::arena::VertexSimplex;
use std::thread;

pub trait Actor: Send + Sync {
    fn on_chunk(&mut self, chunk: &[VertexSimplex]);
}

pub struct PetriRuntime;

impl PetriRuntime {
    /// Splits `data` into contiguous chunks, spawns one scoped thread per chunk (thread count is `available_cores` clamped to 1–64), and calls `actor.on_chunk` on each chunk with a clone of `actor`.
    pub fn dispatch_wavefront<A: Actor + Clone + Send>(
        available_cores: usize,
        actor: A,
        data: &[VertexSimplex],
    ) {
        // Thread count follows the caller-supplied core hint, bounded to avoid runaway parallelism.
        let active_threads = available_cores.clamp(1, 64);

        // One chunk per thread; each chunk holds at least one element when data is non-empty.
        let chunk_size = (data.len() / active_threads).max(1);
        
        thread::scope(|s| {
            for chunk in data.chunks(chunk_size) {
                let mut local_actor = actor.clone();
                s.spawn(move || {
                    local_actor.on_chunk(chunk);
                });
            }
        });
    }
}