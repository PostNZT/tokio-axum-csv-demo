use csv::ReaderBuilder;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use anyhow::Result;

mod performance_utils {
    include!("../src/performance_utils.rs");
}

use performance_utils::{PerformanceTimer, SalesRecord};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Tokio CSV Processing Demo");
    println!("===========================");
    
    // Ensure sample data exists
    generate_sample_data_if_needed().await?;
    
    // Test different file sizes with different strategies
    let files = [
        ("sample_data/small_data.csv", "Small (1K records)"),
        ("sample_data/medium_data.csv", "Medium (100K records)"),
        ("sample_data/large_data.csv", "Large (1M records)"),
    ];

    for (file_path, description) in files {
        if Path::new(file_path).exists() {
            println!("\n🔍 Processing: {}", description);
            
            // Method 1: Async file reading + sync CSV parsing
            async_file_sync_csv(file_path).await?;
            
            // Method 2: Streaming async CSV processing
            streaming_async_csv(file_path).await?;
            
            // Method 3: Concurrent chunk processing
            concurrent_chunk_processing(file_path).await?;
            
            println!("{}", "=".repeat(50));
        } else {
            println!("⚠️  {} not found, skipping...", file_path);
        }
    }

    Ok(())
}

async fn async_file_sync_csv(file_path: &str) -> Result<()> {
    let timer = PerformanceTimer::new(format!("Async File + Sync CSV: {}", file_path));
    
    // Read entire file asynchronously
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    
    // Parse CSV synchronously
    let mut reader = ReaderBuilder::new().from_reader(contents.as_bytes());
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: SalesRecord = result?;
        records.push(record);
    }
    
    timer.finish(records.len());
    Ok(())
}

async fn streaming_async_csv(file_path: &str) -> Result<()> {
    let timer = PerformanceTimer::new(format!("Streaming Async CSV: {}", file_path));
    
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    
    // Read in chunks to simulate streaming
    let mut buffer = Vec::new();
    let mut buf_reader = reader;
    buf_reader.read_to_end(&mut buffer).await?;
    
    // Process the buffer
    let mut csv_reader = ReaderBuilder::new().from_reader(&buffer[..]);
    let mut record_count = 0;
    
    for result in csv_reader.deserialize() {
        let _record: SalesRecord = result?;
        record_count += 1;
        
        // Simulate some async processing work
        if record_count % 10000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    timer.finish(record_count);
    Ok(())
}

async fn concurrent_chunk_processing(file_path: &str) -> Result<()> {
    let timer = PerformanceTimer::new(format!("Concurrent Chunk Processing: {}", file_path));
    
    // Read file
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    
    // Split into chunks for concurrent processing
    let lines: Vec<&str> = contents.lines().collect();
    let header = lines[0];
    let data_lines = &lines[1..];
    
    let chunk_size = 10000.max(data_lines.len() / 4); // At least 4 chunks
    let chunks: Vec<_> = data_lines.chunks(chunk_size).collect();
    
    println!("   Processing {} chunks of ~{} records each", chunks.len(), chunk_size);
    
    // Process chunks concurrently
    let mut tasks = Vec::new();
    
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_data = format!("{}\n{}", header, chunk.join("\n"));
        
        let task = tokio::spawn(async move {
            let mut reader = ReaderBuilder::new().from_reader(chunk_data.as_bytes());
            let mut count = 0;
            
            for result in reader.deserialize() {
                let _record: SalesRecord = result.unwrap();
                count += 1;
            }
            
            println!("     Chunk {} processed: {} records", i + 1, count);
            count
        });
        
        tasks.push(task);
    }
    
    // Wait for all chunks to complete
    let mut total_records = 0;
    for task in tasks {
        total_records += task.await?;
    }
    
    timer.finish(total_records);
    Ok(())
}

async fn generate_sample_data_if_needed() -> Result<()> {
    use std::process::Command;
    
    if !Path::new("sample_data").exists() {
        println!("📁 Sample data not found. Generating...");
        
        // Generate small dataset for demo
        let output = Command::new("cargo")
            .args(&["run", "--bin", "generate_data", "--", "--size", "small"])
            .output()?;
            
        if !output.status.success() {
            println!("⚠️  Could not generate sample data. Creating minimal data...");
            tokio::fs::create_dir_all("sample_data").await?;
            
            let minimal_csv = "id,customer_name,product,quantity,price,date,region\n1,John Doe,Laptop,1,999.99,2024-01-01,North\n2,Jane Smith,Mouse,2,29.99,2024-01-02,South";
            tokio::fs::write("sample_data/small_data.csv", minimal_csv).await?;
        }
    }
    
    Ok(())
}