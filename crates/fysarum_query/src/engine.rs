use fysarum_compute::runtime::{Actor, PetriRuntime};
use fysarum_storage::arena::{SimplexArena, VertexSimplex};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct FysarumEngine {
    arena: SimplexArena,
}

// --- FILTER PHYSICS ---
/// Strict, compile-time evaluable filter conditions.
#[derive(Clone)]
pub enum FilterCondition {
    GreaterThan(usize, f64),
    LessThan(usize, f64),
    Equals(usize, f64),
}

// --- INTERNAL AUTONOMOUS ACTORS ---
#[derive(Clone)]
struct AutonomousSumActor {
    column_index: usize,
    filter: Option<FilterCondition>,
    total_sum_bits: Arc<AtomicU64>,
}

impl Actor for AutonomousSumActor {
    fn on_chunk(&mut self, chunk: &[VertexSimplex]) {
        let mut local_sum = 0.0;
        
        for simplex in chunk {
            if simplex.entity_id != 0 {
                // Apply the hardware-level branching filter
                let pass = match &self.filter {
                    Some(FilterCondition::GreaterThan(col, val)) => simplex.state_vector[*col] > *val,
                    Some(FilterCondition::LessThan(col, val)) => simplex.state_vector[*col] < *val,
                    // Use a tiny epsilon for float equality
                    Some(FilterCondition::Equals(col, val)) => (simplex.state_vector[*col] - *val).abs() < 1e-9,
                    None => true,
                };

                if pass {
                    local_sum += simplex.state_vector[self.column_index];
                }
            }
        }

        // Lock-free Atomic aggregation
        let mut current = self.total_sum_bits.load(Ordering::Relaxed);
        loop {
            let current_f64 = f64::from_bits(current);
            let new_f64 = current_f64 + local_sum;
            let new_bits = new_f64.to_bits();
            
            match self.total_sum_bits.compare_exchange_weak(
                current, new_bits, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(v) => current = v,
            }
        }
    }
}

// --- FLUENT BUILDER API ---
pub struct WavefrontBuilder<'a> {
    engine: &'a FysarumEngine,
    target_column: Option<usize>,
    filter_condition: Option<FilterCondition>,
}

impl<'a> WavefrontBuilder<'a> {
    pub fn new(engine: &'a FysarumEngine) -> Self {
        Self { engine, target_column: None, filter_condition: None }
    }

    /// Chains a filter condition into the query pipeline
    pub fn filter(mut self, condition: FilterCondition) -> Self {
        self.filter_condition = Some(condition);
        self
    }

    /// Chains the SUM operation
    pub fn sum(mut self, column_index: usize) -> Self {
        self.target_column = Some(column_index);
        self
    }

    /// Finalizes the chain, compiles the Actor, and detonates the compute mesh.
    pub fn execute(self) -> std::io::Result<f64> {
        match self.target_column {
            Some(col) => self.engine.sum_column_internal(col, self.filter_condition),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput, 
                "Query malformed: No operations provided to the wavefront."
            ))
        }
    }
}

// --- USER FACING API ---
impl FysarumEngine {
    pub fn open_or_create<P: AsRef<Path>>(path: P, initial_capacity: usize) -> std::io::Result<Self> {
        let arena = SimplexArena::new(path, initial_capacity)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(Self { arena })
    }

    pub fn insert_raw(&mut self, index: usize, data: VertexSimplex) -> std::io::Result<()> {
        let simplices = self.arena.as_simplices_mut()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            
        if index < simplices.len() {
            simplices[index] = data;
        }
        Ok(())
    }

    /// Entry point for the Fluent API
    pub fn wavefront(&self) -> WavefrontBuilder<'_> {
        WavefrontBuilder::new(self)
    }

    pub fn execute_query<A: Actor + Clone + Send>(&self, query_actor: A) -> std::io::Result<()> {
        let core_count = std::thread::available_parallelism()?.get();
        let simplices = self.arena.as_simplices()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        PetriRuntime::dispatch_wavefront(core_count, query_actor, simplices);
        Ok(())
    }

    // Internal Actor generation
    fn sum_column_internal(&self, column_index: usize, filter: Option<FilterCondition>) -> std::io::Result<f64> {
        if column_index > 3 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Out of bounds"));
        }

        let total_sum_bits = Arc::new(AtomicU64::new(0.0f64.to_bits()));
        
        let actor = AutonomousSumActor {
            column_index,
            filter,
            total_sum_bits: total_sum_bits.clone(),
        };

        self.execute_query(actor)?;

        let result_bits = total_sum_bits.load(Ordering::Relaxed);
        Ok(f64::from_bits(result_bits))
    }
}