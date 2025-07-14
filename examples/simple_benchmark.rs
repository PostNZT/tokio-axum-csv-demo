use csv::ReaderBuilder;
use std::time::Instant;

// We only need to count records, not deserialize them
// So let's use a simpler approach

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏆 Simple Sync vs Async Benchmark");
    println!("=================================");
    
    let file_path = "sample_data/small_data.csv";
    
    if !std::path::Path::new(file_path).exists() {
        println!("❌ Sample data not found. Run: cargo run --bin generate_data");
        return Ok(());
    }
    
    // Test with medium data if available
    let test_files = [
        ("sample_data/small_data.csv", "Small Dataset"),
        ("sample_data/medium_data.csv", "Medium Dataset"),
    ];
    
    for (file_path, description) in test_files {
        if std::path::Path::new(file_path).exists() {
            println!("\n🔍 Testing: {}", description);
            
            // Benchmark 1: Synchronous processing
            sync_benchmark(file_path)?;
            
            // Benchmark 2: Asynchronous processing  
            async_benchmark(file_path).await?;
            
            println!("{}", "-".repeat(30));
        }
    }
    
    Ok(())
}

fn sync_benchmark(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!("🔄 Synchronous Processing:");
    
    let content = std::fs::read_to_string(file_path)?;
    let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
    let mut count = 0;
    
    // Just count records without deserializing to avoid unused field warnings
    for result in reader.records() {
        let _record = result?;
        count += 1;
    }
    
    let duration = start.elapsed();
    let rps = count as f64 / duration.as_secs_f64();
    
    println!("   ✅ {} records in {:?} ({:.0} records/sec)", count, duration, rps);
    Ok(())
}

async fn async_benchmark(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!("⚡ Asynchronous Processing:");
    
    let content = tokio::fs::read_to_string(file_path).await?;
    let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
    let mut count = 0;
    
    // Just count records without deserializing
    for result in reader.records() {
        let _record = result?;
        count += 1;
        
        // Yield every 100 records to demonstrate async behavior
        if count % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let duration = start.elapsed();
    let rps = count as f64 / duration.as_secs_f64();
    
    println!("   ✅ {} records in {:?} ({:.0} records/sec)", count, duration, rps);
    Ok(())
}