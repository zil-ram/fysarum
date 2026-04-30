use fysarum_query::engine::{FysarumEngine, FilterCondition};
use fysarum_storage::arena::VertexSimplex;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    println!("--- Fysarum query example ---");

    let db_path = "./fysarum_massive.fys";
    let target_rows = 50_000_000;
    
    println!("Allocating zero-copy storage file...");
    let mut engine = FysarumEngine::open_or_create(db_path, target_rows)?;

    println!("Generating synthetic records...");
    for i in 0..target_rows {
        // We set state_vector[1] to equal the ID, so we can filter on it easily
        engine.insert_raw(i, VertexSimplex {
            entity_id: i as u64 + 1,
            state_vector: [1.0, i as f64, 0.0, 0.0],
            logical_clock: 1,
        })?;
    }

    println!("Data mapped successfully. Igniting Atomics...");
    
    let start_time = Instant::now();
    
    // Fluent query pipeline:
    // Sum column 0 only for rows where column 1 is greater than 25,000,000.
    let final_result = engine.wavefront()
        .filter(FilterCondition::GreaterThan(1, 25_000_000.0))
        .sum(0)
        .execute()?;

    let duration = start_time.elapsed();

    println!("------------------------------------------------");
    println!("Query Complete in {:?}", duration);
    // With this synthetic dataset, this filter excludes rows up to and including
    // 25,000,000, so the expected sum of column 0 values is 24,999,999.
    println!("Total Sum Result: {}", final_result);
    println!("------------------------------------------------");

    Ok(())
}