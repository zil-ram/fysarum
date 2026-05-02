use fysarum_compute::runtime::{Actor, PetriRuntime};
use fysarum_storage::arena::{SimplexArena, VertexSimplex};
use fysarum_storage::schema::Schema;
use fysarum_storage::wal::WriteAheadLog;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub wal_enabled: bool,
    pub dynamic_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { wal_enabled: false, dynamic_mode: false }
    }
}

pub struct FysarumEngine {
    arena: SimplexArena,
    wal: Option<WriteAheadLog>,
    pub config: EngineConfig,
}

#[derive(Clone)]
pub enum FilterCondition {
    GreaterThan(usize, f64),
    LessThan(usize, f64),
    Equals(usize, f64),
}

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
                let pass = match &self.filter {
                    Some(FilterCondition::GreaterThan(col, val)) => simplex.state_vector[*col] > *val,
                    Some(FilterCondition::LessThan(col, val)) => simplex.state_vector[*col] < *val,
                    Some(FilterCondition::Equals(col, val)) => (simplex.state_vector[*col] - *val).abs() < 1e-9,
                    None => true,
                };
                if pass { local_sum += simplex.state_vector[self.column_index]; }
            }
        }
        let mut current = self.total_sum_bits.load(Ordering::Relaxed);
        loop {
            let current_f64 = f64::from_bits(current);
            let new_f64 = current_f64 + local_sum;
            let new_bits = new_f64.to_bits();
            match self.total_sum_bits.compare_exchange_weak(current, new_bits, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(v) => current = v,
            }
        }
    }
}

pub struct WavefrontBuilder<'a> {
    engine: &'a FysarumEngine,
    target_column: Option<usize>,
    filter_condition: Option<FilterCondition>,
}

impl<'a> WavefrontBuilder<'a> {
    pub fn new(engine: &'a FysarumEngine) -> Self {
        Self { engine, target_column: None, filter_condition: None }
    }
    pub fn filter(mut self, condition: FilterCondition) -> Self {
        self.filter_condition = Some(condition);
        self
    }
    pub fn sum(mut self, column_index: usize) -> Self {
        self.target_column = Some(column_index);
        self
    }
    pub fn execute(self) -> std::io::Result<f64> {
        if self.engine.config.dynamic_mode {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported, 
                "wavefront execution is not available while dynamic_mode is enabled"
            ));
        }

        match self.target_column {
            Some(col) => self.engine.sum_column_internal(col, self.filter_condition),
            None => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Query malformed.")),
        }
    }
}

impl FysarumEngine {
    pub fn open_with_config<P: AsRef<Path>>(path: P, initial_capacity: usize, config: EngineConfig) -> std::io::Result<Self> {
        let mut arena = SimplexArena::new(&path, initial_capacity)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        
        let mut wal_instance = None;
        if config.wal_enabled {
            let wal_path = format!("{}.wal", path.as_ref().to_string_lossy());
            let mut wal = WriteAheadLog::open(&wal_path)?;

            let recoveries = wal.recover()?;
            if !recoveries.is_empty() {
                let simplices = arena.as_simplices_mut()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
                for (index, simplex) in &recoveries {
                    if *index < simplices.len() {
                        simplices[*index] = *simplex;
                    }
                }
                println!("[SYSTEM ALERT] Recovered {} records from WAL.", recoveries.len());
            }
            wal_instance = Some(wal);
        }

        Ok(Self { arena, wal: wal_instance, config })
    }

    /// Writes a `VertexSimplex` at `index` in the mmap’d arena and, when WAL is enabled, appends the same update to the log.
    pub fn insert_raw(&mut self, index: usize, data: VertexSimplex) -> std::io::Result<()> {
        if self.config.dynamic_mode {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "insert_raw is only valid when dynamic_mode is false"));
        }

        if let Some(wal) = &mut self.wal {
            wal.append(index, data)?;
        }

        let simplices = self.arena.as_simplices_mut()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            
        if index < simplices.len() { simplices[index] = data; }
        Ok(())
    }

    /// Copies `raw_data` into the arena row starting at byte offset `row_index * schema.row_size()`; row width comes from `schema`.
    pub fn insert_dynamic(&mut self, row_index: usize, schema: &Schema, raw_data: &[u8]) -> std::io::Result<()> {
        if !self.config.dynamic_mode {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "insert_dynamic requires dynamic_mode to be true in EngineConfig"));
        }

        // Row byte range from schema stride
        let stride = schema.row_size();
        let start_byte = row_index * stride;
        let end_byte = start_byte + stride;

        let memory_bytes = self.arena.as_bytes_mut()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        if end_byte <= memory_bytes.len() {
            memory_bytes[start_byte..end_byte].copy_from_slice(raw_data);
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, "row index exceeds mapped capacity"));
        }

        // Dynamic inserts update the mmap only; WAL append is not performed on this path.
        Ok(())
    }

    pub fn wavefront(&self) -> WavefrontBuilder<'_> {
        WavefrontBuilder::new(self)
    }

    pub fn execute_query<A: Actor + Clone + Send>(&self, query_actor: A) -> std::io::Result<()> {
        let core_count = std::thread::available_parallelism()?.get();
        self.arena.prefetch_async();
        let simplices = self.arena.as_simplices()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        PetriRuntime::dispatch_wavefront(core_count, query_actor, simplices);
        Ok(())
    }

    fn sum_column_internal(&self, column_index: usize, filter: Option<FilterCondition>) -> std::io::Result<f64> {
        if column_index > 3 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Out of bounds")); }
        let total_sum_bits = Arc::new(AtomicU64::new(0.0f64.to_bits()));
        let actor = AutonomousSumActor { column_index, filter, total_sum_bits: total_sum_bits.clone() };
        self.execute_query(actor)?;
        Ok(f64::from_bits(total_sum_bits.load(Ordering::Relaxed)))
    }
}