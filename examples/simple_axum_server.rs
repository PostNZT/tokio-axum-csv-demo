use axum::{
    extract::Path,
    response::Json,
    routing::get,
    Router,
};
use csv::ReaderBuilder;
use serde_json;
use std::time::Instant;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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
    println!("🌐 Simple Axum CSV Server (Fixed)");
    println!("=================================");
    
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/process", get(process_default_csv))
        .route("/process/:filename", get(process_specific_csv))  // ← New dynamic route!
        .route("/health", get(health_check))
        .route("/files", get(list_files));  // ← List available files
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("\n📋 Available endpoints:");
    println!("  GET / - Root endpoint");
    println!("  GET /process - Process default CSV (small_data.csv)");
    println!("  GET /process/:filename - Process specific CSV file");
    println!("  GET /files - List available CSV files");
    println!("  GET /health - Health check");
    println!("\n💡 Try these commands:");
    println!("  curl http://127.0.0.1:3000/files");
    println!("  curl http://127.0.0.1:3000/process/small_data.csv");
    println!("  curl http://127.0.0.1:3000/process/medium_data.csv");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Simple Axum CSV Processing Server",
        "endpoints": {
            "process": "/process - Process default CSV file",
            "process_file": "/process/:filename - Process specific CSV file",
            "files": "/files - List available files",
            "health": "/health - Server health check"
        },
        "examples": {
            "process_small": "/process/small_data.csv",
            "process_medium": "/process/medium_data.csv",
            "list_files": "/files"
        }
    }))
}

async fn process_default_csv() -> Json<serde_json::Value> {
    process_csv_file("small_data.csv").await
}

async fn process_specific_csv(Path(filename): Path<String>) -> Json<serde_json::Value> {
    // Remove .csv extension if provided, then add it back
    let clean_filename = filename.strip_suffix(".csv").unwrap_or(&filename);
    let csv_filename = format!("{}.csv", clean_filename);
    
    process_csv_file(&csv_filename).await
}

async fn process_csv_file(filename: &str) -> Json<serde_json::Value> {
    let start = Instant::now();
    let file_path = format!("sample_data/{}", filename);
    
    println!("🔍 Processing: {}", file_path);
    
    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
            let mut records = Vec::new();
            
            for result in reader.deserialize() {
                match result {
                    Ok(record) => {
                        let record: SalesRecord = record;
                        records.push(record);
                    }
                    Err(e) => {
                        return Json(serde_json::json!({
                            "status": "error",
                            "message": format!("CSV parsing error: {}", e),
                            "file": filename
                        }));
                    }
                }
            }
            
            let duration = start.elapsed();
            let rps = records.len() as f64 / duration.as_secs_f64();
            
            Json(serde_json::json!({
                "status": "success",
                "file": filename,
                "file_path": file_path,
                "records_processed": records.len(),
                "duration_ms": duration.as_millis(),
                "records_per_second": rps as u64,
                "sample_record": records.first()
            }))
        }
        Err(_) => {
            Json(serde_json::json!({
                "status": "error",
                "message": format!("File not found: {}", file_path),
                "available_files": "Try GET /files to see available files",
                "suggestion": "Run 'cargo run --bin generate_data' to create sample files"
            }))
        }
    }
}

async fn list_files() -> Json<serde_json::Value> {
    match tokio::fs::read_dir("sample_data").await {
        Ok(mut entries) => {
            let mut files = Vec::new();
            
            while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".csv") {
                        files.push(filename.to_string());
                    }
                }
            }
            
            Json(serde_json::json!({
                "status": "success",
                "available_files": files,
                "endpoints": files.iter().map(|f| format!("/process/{}", f)).collect::<Vec<_>>()
            }))
        }
        Err(_) => {
            Json(serde_json::json!({
                "status": "error",
                "message": "sample_data directory not found",
                "suggestion": "Run 'cargo run --bin generate_data' to create sample files"
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