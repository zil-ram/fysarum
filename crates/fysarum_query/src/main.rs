use fysarum_query::engine::{FysarumEngine, EngineConfig, FilterCondition};
use fysarum_storage::arena::VertexSimplex;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    println!("--- Project Fysarum: Enterprise Config Test ---");

    let db_path = "./fysarum_massive.fys";
    let target_rows = 50_000_000;
    
    // Engine configuration used for this run
    let config = EngineConfig {
        wal_enabled: false, // writes skip the WAL; no crash recovery from the log
        dynamic_mode: false,
    };

    println!("Booting engine in vector mode (WAL Disabled)...");
    let mut engine = FysarumEngine::open_with_config(db_path, target_rows, config)?;

    println!("Synthesizing 50,000,000 records...");
    let insert_start = Instant::now();
    for i in 0..target_rows {
        engine.insert_raw(i, VertexSimplex {
            entity_id: i as u64 + 1,
            state_vector: [1.0, i as f64, 0.0, 0.0],
            logical_clock: 1,
        })?;
    }
    println!("Ingestion completed in {:?}", insert_start.elapsed());

    println!("Igniting Compute Mesh...");
    let query_start = Instant::now();
    
    let final_result = engine.wavefront()
        .filter(FilterCondition::GreaterThan(1, 25_000_000.0))
        .sum(0)
        .execute()?;

    println!("------------------------------------------------");
    println!("Query Complete in {:?}", query_start.elapsed());
    println!("Total Sum Result: {}", final_result);
    println!("------------------------------------------------");

    Ok(())
}