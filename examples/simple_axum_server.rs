use axum::{
    response::Json,
    routing::get,
    Router,
};
use csv::ReaderBuilder;
use serde_json;
use std::time::Instant;

#[derive(Debug, serde::Deserialize, serde::Serialize)]  // Added Serialize!
struct SalesRecord {
    id: u32,
    customer_name: String,
    product: String,
    quantity: u32,
    price: f64,
    date: String,
    region: String,
}

#[tokio::main]
async fn main() {
    println!("🌐 Simple Axum CSV Server");
    println!("========================");
    
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/process", get(process_csv))
        .route("/health", get(health_check));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("\n📋 Available endpoints:");
    println!("  GET / - Root endpoint");
    println!("  GET /process - Process CSV and return metrics");
    println!("  GET /health - Health check");
    println!("\n💡 Try: curl http://127.0.0.1:3000/process");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Simple Axum CSV Processing Server",
        "endpoints": {
            "process": "/process - Process CSV file with performance metrics",
            "health": "/health - Server health check"
        }
    }))
}

async fn process_csv() -> Json<serde_json::Value> {
    let start = Instant::now();
    let file_path = "sample_data/small_data.csv";
    
    match tokio::fs::read_to_string(file_path).await {
        Ok(content) => {
            let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
            let mut records = Vec::new();
            
            for result in reader.deserialize() {
                match result {
                    Ok(record) => {
                        let record: SalesRecord = record;
                        records.push(record);
                    }
                    Err(_) => break,
                }
            }
            
            let duration = start.elapsed();
            let rps = records.len() as f64 / duration.as_secs_f64();
            
            Json(serde_json::json!({
                "status": "success",
                "file": file_path,
                "records_processed": records.len(),
                "duration_ms": duration.as_millis(),
                "records_per_second": rps,
                "sample_record": records.first()  // Now this will work!
            }))
        }
        Err(_) => {
            Json(serde_json::json!({
                "status": "error",
                "message": "Could not read CSV file. Run 'cargo run --bin generate_data' first."
            }))
        }
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "server": "Axum CSV Processor"
    }))
}