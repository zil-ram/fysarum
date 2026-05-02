use crate::engine::FysarumEngine;
use fysarum_storage::arena::VertexSimplex;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Row shape produced when deserializing JSON Lines or CSV records for ingestion.
#[derive(Deserialize, Debug)]
pub struct RawDataRow {
    pub id: u64,
    pub val0: f64,
    pub val1: f64,
    pub val2: f64,
    pub val3: f64,
}

pub struct DataImporter;

impl DataImporter {
    /// Ingests a standard CSV file directly into the Fysarum zero-copy mesh
    pub fn ingest_csv(engine: &mut FysarumEngine, filepath: &str, start_index: usize) -> std::io::Result<usize> {
        let mut rdr = csv::Reader::from_path(filepath)?;
        let mut count = 0;

        for result in rdr.deserialize() {
            let record: RawDataRow = result?;
            
            let simplex = VertexSimplex {
                entity_id: record.id,
                state_vector: [record.val0, record.val1, record.val2, record.val3],
                logical_clock: 1,
            };

            engine.insert_raw(start_index + count, simplex)?;
            count += 1;
        }
        Ok(count)
    }

    /// Ingests a JSON Lines file (one JSON object per line) into the Fysarum mesh
    pub fn ingest_json_lines(engine: &mut FysarumEngine, filepath: &str, start_index: usize) -> std::io::Result<usize> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() { continue; }
            
            let record: RawDataRow = serde_json::from_str(&line)?;
            
            let simplex = VertexSimplex {
                entity_id: record.id,
                state_vector: [record.val0, record.val1, record.val2, record.val3],
                logical_clock: 1,
            };

            engine.insert_raw(start_index + count, simplex)?;
            count += 1;
        }
        Ok(count)
    }
}