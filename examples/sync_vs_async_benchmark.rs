use csv::ReaderBuilder;
use std::fs;
use std::time::Instant;
use rayon::prelude::*;

mod performance_utils {
    include!("../src/performance_utils.rs");
}

use performance_utils::{PerformanceTimer, SalesRecord};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏆 Comprehensive CSV Processing Benchmark");
    println!("========================================");
    
    let test_files = [
        ("sample_data/small_data.csv", "Small Dataset (1K records)"),
        ("sample_data/medium_data.csv", "Medium Dataset (100K records)"),
        ("sample_data/large_data.csv", "Large Dataset (1M records)"),
    ];
    
    for (file_path, description) in test_files {
        if !std::path::Path::new(file_path).exists() {
            println!("⚠️  {} not found, skipping...", file_path);
            continue;
        }
        
        println!("\n🔍 Testing: {}", description);
        println!("{}", "=".repeat(50));
        
        // Run all benchmarks for this file
        benchmark_sync_processing(file_path)?;
        benchmark_async_processing(file_path).await?;
        benchmark_parallel_processing(file_path)?;
        benchmark_async_parallel_processing(file_path).await?;
        
        println!("{}", "=".repeat(50));
    }
    
    println!("\n📊 Benchmark Summary:");
    println!("• Sync: Traditional single-threaded processing");
    println!("• Async: Tokio async/await with yielding");
    println!("• Parallel: Multi-threaded with Rayon");
    println!("• Async+Parallel: Combine async I/O with parallel processing");
    println!("\n💡 Key Takeaways:");
    println!("• Async shines for I/O-bound operations");
    println!("• Parallel processing helps with CPU-bound work");
    println!("• Combined approach best for large datasets");
    
    Ok(())
}

fn benchmark_sync_processing(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timer = PerformanceTimer::new("🔄 Synchronous Processing".to_string());
    
    let content = fs::read_to_string(file_path)?;
    let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: SalesRecord = result?;
        records.push(record);
    }
    
    timer.finish(records.len());
    Ok(())
}

async fn benchmark_async_processing(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timer = PerformanceTimer::new("⚡ Asynchronous Processing".to_string());
    
    let content = tokio::fs::read_to_string(file_path).await?;
    let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
    let mut records = Vec::new();
    
    let mut count = 0;
    for result in reader.deserialize() {
        let record: SalesRecord = result?;
        records.push(record);
        count += 1;
        
        // Yield control periodically to allow other tasks
        if count % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    timer.finish(records.len());
    Ok(())
}

fn benchmark_parallel_processing(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timer = PerformanceTimer::new("🚀 Parallel Processing (Rayon)".to_string());
    
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        timer.finish(0);
        return Ok(());
    }
    
    let header = lines[0];
    let data_lines = &lines[1..];
    
    // Process chunks in parallel
    let chunk_size = 10000.max(data_lines.len() / num_cpus::get());
    let total_records: usize = data_lines
        .par_chunks(chunk_size)
        .map(|chunk| {
            let chunk_content = format!("{}\n{}", header, chunk.join("\n"));
            let mut reader = ReaderBuilder::new().from_reader(chunk_content.as_bytes());
            let mut count = 0;
            
            for result in reader.deserialize() {
                if let Ok(_record) = result {
                    let _record: SalesRecord = _record;
                    count += 1;
                }
            }
            count
        })
        .sum();
    
    timer.finish(total_records);
    Ok(())
}

async fn benchmark_async_parallel_processing(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timer = PerformanceTimer::new("🔥 Async + Parallel Processing".to_string());
    
    // Async file read
    let content = tokio::fs::read_to_string(file_path).await?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        timer.finish(0);
        return Ok(());
    }
    
    let header = lines[0];
    let data_lines = &lines[1..];
    
    // Split into chunks for concurrent processing
    let chunk_size = 10000.max(data_lines.len() / 8); // 8 concurrent tasks
    let chunks: Vec<_> = data_lines.chunks(chunk_size).collect();
    
    // Process chunks concurrently
    let mut tasks = Vec::new();
    
    for chunk in chunks {
        let chunk_content = format!("{}\n{}", header, chunk.join("\n"));
        
        let task = tokio::spawn(async move {
            let mut reader = ReaderBuilder::new().from_reader(chunk_content.as_bytes());
            let mut count = 0;
            
            for result in reader.deserialize() {
                if let Ok(_record) = result {
                    let _record: SalesRecord = _record;
                    count += 1;
                }
                
                // Yield occasionally within each task
                if count % 1000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            count
        });
        
        tasks.push(task);
    }
    
    // Collect results
    let mut total_records = 0;
    for task in tasks {
        total_records += task.await?;
    }
    
    timer.finish(total_records);
    Ok(())
}