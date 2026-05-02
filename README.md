Fysarum

A decentralized, zero copy, hardware level data engine.

For over a decade, distributed data ecosystems have been paralyzed by iterative patches to fundamentally flawed abstractions. The rigid schema on write of traditional RDBMS, the walled gardens of cloud warehouses, and the JVM bloated DAG execution of Spark have reached their physical limits.

By abandoning relational models, columnar files, and centralized query planners, Fysarum introduces a unified, mechanically sympathetic architecture rooted in algebraic topology, decentralized actor models, and zero copy kernel bypass memory physics.

Tradeoffs and Architecture Decisions

Fysarum features a toggleable physics engine. Users can disable the Write Ahead Log (WAL) for absolute hardware performance and zero copy RAM speeds, or enable it for strict disk durability. While current memory mapping previously required datasets to fit within RAM to avoid blocking page faults, Fysarum now utilizes asynchronous kernel prefetching (madvise) to stream data seamlessly from NVMe drives to the L1 cache.

The Architecture

Fysarum bypasses standard OS bottlenecks through a strict four layer architecture:

Layer 1: Hardware Abstraction Plane (io_uring)
Bypasses the OS page cache entirely. Utilizes io_uring for asynchronous, lockless block I/O directly from NVMe drives.

Layer 2: Fractal Storage Fabric (Zero Copy mmap)
Abandons Parquet and ORC formats. Data is stored as mathematical simplices in C memory layouts. Files are mmap'd into memory, meaning the bytes on disk are the Rust structs in RAM. Zero serialization overhead.

Layer 3: Wavefront Compute Mesh (Actor Model)
Abandons DAG stages and JVM Garbage Collection. Fysarum compiles queries into localized state machines (Actors) that swarm the data via lock free channels and hardware level Atomics.

Layer 4: Topological Query Builder
Hides the brutal complexity of the lower layers behind an elegant, chained Rust API.

Dual Engine Physics

Ships with two configurable storage paradigms:

Bare Metal Vector Mode (AI / Quant Finance): Bypasses the Write Ahead Log and uses hardcoded mathematical structures (VertexSimplex) for ultra low latency Vector Similarity Searches and quantitative math.

Dynamic Schema Mode (Enterprise / E-Commerce): Enables the Write Ahead Log for crash resilience and allows the injection of unstructured raw bytes (Strings, Booleans) via the Dynamic Schema Registry.

Getting Started

Prerequisites

OS: Linux (Kernel 5.1+ required for io_uring support).

Compiler: Rust toolchain.

Cargo dependency

Declare the crate in Cargo.toml (git source shown below):

[dependencies]
fysarum_query = { git = "[https://github.com/zil-ram/fysarum] }


1. Ingesting Real Data (CSV/JSON)

Fysarum reads the .fys on-disk layout. CSV/JSON ingestion runs through the built-in importer, which writes that layout:

use fysarum_query::engine::{FysarumEngine, EngineConfig};
use fysarum_query::importer::DataImporter;

fn main() -> std::io::Result<()> {
    // Enable the WAL for safe data ingestion
    let config = EngineConfig { wal_enabled: true, dynamic_mode: false };
    let mut engine = FysarumEngine::open_with_config("./data.fys", 1_000_000, config)?;
    
    // Convert a standard CSV into a zero copy NVMe map
    DataImporter::ingest_csv(&mut engine, "my_data.csv", 0)?;
    Ok(())
}


2. High Speed Query Builder

Fysarum is booted using the EngineConfig, allowing you to toggle durability and schema modes on the fly.

use fysarum_query::engine::{FysarumEngine, EngineConfig, FilterCondition};

fn main() -> std::io::Result<()> {
    // Toggle pure speed vs durability
    let config = EngineConfig {
        wal_enabled: false,  // Danger Mode: Instant writes, no disk bottleneck
        dynamic_mode: false, // Vector Mode: Hardcoded f64 math only
    };

    // Maps the file with zero serialization
    let engine = FysarumEngine::open_with_config("./data.fys", 1_000_000, config)?;
    
    // Calculates the sum of column 0, filtering for rows where column 1 > 25,000,000.
    // Compiles down to hardware level branching and lock free Atomics.
    let result = engine.wavefront()
        .filter(FilterCondition::GreaterThan(1, 25_000_000.0))
        .sum(0)
        .execute()?;
    
    println!("Result: {}", result);
    Ok(())
}


Open Source Core Roadmap

[x] Zero Copy NVMe Memory Mapping

[x] io_uring Hardware Bridge

[x] Lock Free Atomic Compute Mesh

[x] Chained Query Builder

[x] Write Ahead Log (WAL) Crash Resilience

[x] Dynamic Schema Registry

[ ] Distributed RDMA Networking (Multi Node Mesh)

[ ] Evaluating unstructured bytes

Fysarum is open-source and contributions are welcome. Pls reach out for feedback as well. :D