Fysarum 

A decentralized, zero copy, hardware level data engine.

For over a decade, distributed data ecosystems have been paralyzed by iterative patches to fundamentally flawed abstractions. The rigid schema on write of traditional RDBMS, the walled gardens of cloud warehouses, and the JVM bloated DAG execution of Spark have reached their physical limits.

By abandoning relational models, columnar files, and centralized query planners, Fysarum introduces a unified, mechanically sympathetic architecture rooted in algebraic topology, decentralized actor models, and zero copy kernel bypass memory physics.

Tradeoffs and Architecture Decisions

Fysarum trades traditional database durability, container isolation, and strict memory protection for absolute hardware performance. It is designed for temporary massive datasets rather than general persistent storage. Current memory mapping requires datasets to fit within RAM to avoid blocking page faults. The future addition of io_uring prefetching and Write Ahead Logging will bridge the gap between experimental speed and production resilience.

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

Bare Metal Vector Engine for AI

Because Fysarum's foundational data structure (VertexSimplex) relies on mathematical state vectors rather than standard string rows, it natively acts as an ultra low latency Vector Database.

For AI engineers building local LLM agents, RAG (Retrieval Augmented Generation) pipelines, or semantic search applications, Fysarum can map billions of embedding vectors directly to the CPU's L1 cache, enabling sub millisecond similarity searches that traditional cloud vector databases cannot mathematically achieve.

Getting Started

Prerequisites

OS: Linux (Kernel 5.1+ required for io_uring support).

Compiler: Rust toolchain.

Usage in your own projects

To use Fysarum in your own data pipelines, add it to your Cargo.toml:

[dependencies]
fysarum_query = { git = "https://github.com/zil-ram/fysarum" }


1. Ingesting Real Data (CSV/JSON)

Fysarum only reads the .fys physical format. You must run a one time ETL using the built in importer to translate your legacy files (CSV/JSON) into the zero copy mesh:

use fysarum_query::engine::FysarumEngine;
use fysarum_query::importer::DataImporter;

fn main() -> std::io::Result<()> {
    let mut engine = FysarumEngine::open_or_create("./data.fys", 1_000_000)?;
    
    // Convert a standard CSV into a zero copy NVMe map
    DataImporter::ingest_csv(&mut engine, "my_data.csv", 0)?;
    Ok(())
}


2. High Speed Query Builder

Once data is mapped, querying occurs at hardware speeds using the chained Query Builder.

use fysarum_query::engine::{FysarumEngine, FilterCondition};

fn main() -> std::io::Result<()> {
    // Maps the 2.4GB+ file instantly with zero serialization
    let engine = FysarumEngine::open_or_create("./data.fys", 1_000_000)?;
    
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

[ ] Distributed RDMA Networking (Multi Node Mesh)

[ ] Write Ahead Log (WAL) Crash Resilience

Contributing

Fysarum is open-source and Fysarum 

A decentralized, zero copy, hardware level data engine.

For over a decade, distributed data ecosystems have been paralyzed by iterative patches to fundamentally flawed abstractions. The rigid schema on write of traditional RDBMS, the walled gardens of cloud warehouses, and the JVM bloated DAG execution of Spark have reached their physical limits.

By abandoning relational models, columnar files, and centralized query planners, Fysarum introduces a unified, mechanically sympathetic architecture rooted in algebraic topology, decentralized actor models, and zero copy kernel bypass memory physics.

Tradeoffs and Architecture Decisions

Fysarum trades traditional database durability, container isolation, and strict memory protection for absolute hardware performance. It is designed for temporary massive datasets rather than general persistent storage. Current memory mapping requires datasets to fit within RAM to avoid blocking page faults. The future addition of io_uring prefetching and Write Ahead Logging will bridge the gap between experimental speed and production resilience.

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

Bare Metal Vector Engine for AI

Because Fysarum's foundational data structure (VertexSimplex) relies on mathematical state vectors rather than standard string rows, it natively acts as an ultra low latency Vector Database.

For AI engineers building local LLM agents, RAG (Retrieval Augmented Generation) pipelines, or semantic search applications, Fysarum can map billions of embedding vectors directly to the CPU's L1 cache, enabling sub millisecond similarity searches that traditional cloud vector databases cannot mathematically achieve.

Getting Started

Prerequisites

OS: Linux (Kernel 5.1+ required for io_uring support).

Compiler: Rust toolchain.

Usage in your own projects

To use Fysarum in your own data pipelines, add it to your Cargo.toml:

[dependencies]
fysarum_query = { git = "https://github.com/zil-ram/fysarum" }


1. Ingesting Real Data (CSV/JSON)

Fysarum only reads the .fys physical format. You must run a one time ETL using the built in importer to translate your legacy files (CSV/JSON) into the zero copy mesh:

use fysarum_query::engine::FysarumEngine;
use fysarum_query::importer::DataImporter;

fn main() -> std::io::Result<()> {
    let mut engine = FysarumEngine::open_or_create("./data.fys", 1_000_000)?;
    
    // Convert a standard CSV into a zero copy NVMe map
    DataImporter::ingest_csv(&mut engine, "my_data.csv", 0)?;
    Ok(())
}


2. High Speed Query Builder

Once data is mapped, querying occurs at hardware speeds using the chained Query Builder.

use fysarum_query::engine::{FysarumEngine, FilterCondition};

fn main() -> std::io::Result<()> {
    // Maps the 2.4GB+ file instantly with zero serialization
    let engine = FysarumEngine::open_or_create("./data.fys", 1_000_000)?;
    
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

[ ] Distributed RDMA Networking (Multi Node Mesh)

[ ] Write Ahead Log (WAL) Crash Resilience

Contributing

Fysarum is open source. We welcome contributions from systems engineers, data scientists, and Rustaceans passionate about pushing hardware to its theoretical limits.Fysarum 

A decentralized, zero copy, hardware level data engine.

For over a decade, distributed data ecosystems have been paralyzed by iterative patches to fundamentally flawed abstractions. The rigid schema on write of traditional RDBMS, the walled gardens of cloud warehouses, and the JVM bloated DAG execution of Spark have reached their physical limits.

By abandoning relational models, columnar files, and centralized query planners, Fysarum introduces a unified, mechanically sympathetic architecture rooted in algebraic topology, decentralized actor models, and zero copy kernel bypass memory physics.

Tradeoffs and Architecture Decisions

Fysarum trades traditional database durability, container isolation, and strict memory protection for absolute hardware performance. It is designed for temporary massive datasets rather than general persistent storage. Current memory mapping requires datasets to fit within RAM to avoid blocking page faults. The future addition of io_uring prefetching and Write Ahead Logging will bridge the gap between experimental speed and production resilience.

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

Bare Metal Vector Engine for AI

Because Fysarum's foundational data structure (VertexSimplex) relies on mathematical state vectors rather than standard string rows, it natively acts as an ultra low latency Vector Database.

For AI engineers building local LLM agents, RAG (Retrieval Augmented Generation) pipelines, or semantic search applications, Fysarum can map billions of embedding vectors directly to the CPU's L1 cache, enabling sub millisecond similarity searches that traditional cloud vector databases cannot mathematically achieve.

Getting Started

Prerequisites

OS: Linux (Kernel 5.1+ required for io_uring support).

Compiler: Rust toolchain.

Usage in your own projects

To use Fysarum in your own data pipelines, add it to your Cargo.toml:

[dependencies]
fysarum_query = { git = "https://github.com/zil-ram/fysarum" }


1. Ingesting Real Data (CSV/JSON)

Fysarum only reads the .fys physical format. You must run a one time ETL using the built in importer to translate your legacy files (CSV/JSON) into the zero copy mesh:

use fysarum_query::engine::FysarumEngine;
use fysarum_query::importer::DataImporter;

fn main() -> std::io::Result<()> {
    let mut engine = FysarumEngine::open_or_create("./data.fys", 1_000_000)?;
    
    // Convert a standard CSV into a zero copy NVMe map
    DataImporter::ingest_csv(&mut engine, "my_data.csv", 0)?;
    Ok(())
}


2. High Speed Query Builder

Once data is mapped, querying occurs at hardware speeds using the chained Query Builder.

use fysarum_query::engine::{FysarumEngine, FilterCondition};

fn main() -> std::io::Result<()> {
    // Maps the file with zero serialization
    let engine = FysarumEngine::open_or_create("./data.fys", 1_000_000)?;
    
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

[ ] Distributed RDMA Networking (Multi Node Mesh)

[ ] Write Ahead Log (WAL) Crash Resilience


Fysarum is open-source and contributions are welcome. Pls for feedback as well. :D