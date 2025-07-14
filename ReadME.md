## Running the Performance Demo

## Project Structure
```
tokio-axum-demo/
├── Cargo.toml
├── README.md
├── sample_data/
│   ├── small_data.csv (1K records)
│   ├── medium_data.csv (100K records)
│   └── large_data.csv (1M records)
├── src/
│   ├── main.rs
│   ├── csv_generator.rs
│   └── performance_utils.rs
└── examples/
    ├── tokio_csv_demo.rs
    ├── axum_csv_server.rs
    └── sync_vs_async_benchmark.rs
```

### 1. Setup the project:
```bash
cargo new tokio-axum-csv-demo
cd tokio-axum-csv-demo
# Copy all the files above
```

### 2. Generate sample data:
```bash
# Generate all sample sizes
cargo run --bin generate_data -- --size small
cargo run --bin generate_data -- --size medium  
cargo run --bin generate_data -- --size large
```

### 3. Run Tokio CSV demo:
```bash
cargo run --bin tokio_csv
```

### 4. Run comprehensive benchmark:
```bash
cargo run --bin benchmark
```

### 5. Start Axum CSV server:
```bash
cargo run --bin axum_csv
```

### 6. Test the web endpoints:
```bash
# Process a CSV file
curl http://127.0.0.1:3000/process/small_data.csv

# Analyze CSV data
curl http://127.0.0.1:3000/analyze/small_data.csv

# Compare processing methods
curl http://127.0.0.1:3000/compare

# Run benchmarks via web API
curl -X POST http://127.0.0.1:3000/benchmark
```

## Performance Insights You'll See:

**📊 Tokio Benefits:**
- Non-blocking I/O operations
- Concurrent processing of multiple files
- Better resource utilization
- Scalable for I/O-bound tasks

**🌐 Axum Advantages:**
- HTTP-based CSV processing
- Real-time performance metrics via API
- File upload and processing pipelines
- Web dashboard for monitoring

**🏆 Performance Comparisons:**
- Sync vs Async processing speeds
- Memory usage patterns
- Concurrent vs sequential processing
- Different file sizes impact

This extended demo will clearly show you how Tokio's async runtime improves CSV processing performance, and how Axum builds web service capabilities on top of that foundation!