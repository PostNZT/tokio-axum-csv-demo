use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tower_http::services::ServeDir;

mod performance_utils {
    include!("../src/performance_utils.rs");
}

use performance_utils::{PerformanceTimer, PerformanceMetrics, SalesRecord};

// Shared application state
type SharedState = Arc<Mutex<AppState>>;

#[derive(Clone)]
struct AppState {
    upload_metrics: Vec<PerformanceMetrics>,
    processing_metrics: Vec<PerformanceMetrics>,
    cached_data: HashMap<String, Vec<SalesRecord>>,
}

#[derive(Deserialize)]
struct AnalysisQuery {
    group_by: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct AnalysisResult {
    total_records: usize,
    total_revenue: f64,
    average_price: f64,
    top_products: Vec<ProductSummary>,
    processing_time_ms: u128,
}

#[derive(Serialize)]
struct ProductSummary {
    product: String,
    total_sales: f64,
    quantity_sold: u32,
}

#[tokio::main]
async fn main() {
    println!("🌐 Axum CSV Processing Server");
    println!("============================");
    
    // Initialize shared state
    let state = Arc::new(Mutex::new(AppState {
        upload_metrics: Vec::new(),
        processing_metrics: Vec::new(),
        cached_data: HashMap::new(),
    }));
    
    // Build the application with routes
    let app = Router::new()
        // File serving
        .nest_service("/files", ServeDir::new("sample_data"))
        
        // CSV processing endpoints
        .route("/", get(root_handler))
        .route("/upload", post(upload_csv))
        .route("/process/:filename", get(process_csv_file))
        .route("/analyze/:filename", get(analyze_csv))
        .route("/compare", get(compare_processing_methods))
        .route("/metrics", get(get_metrics))
        .route("/benchmark", post(run_benchmark))
        
        // Add shared state
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("\n📋 CSV Processing Endpoints:");
    println!("  GET  / - API documentation");
    println!("  POST /upload - Upload CSV file");
    println!("  GET  /process/:filename - Process CSV with performance metrics");
    println!("  GET  /analyze/:filename - Analyze CSV data");
    println!("  GET  /compare - Compare different processing methods");
    println!("  GET  /metrics - View performance metrics");
    println!("  POST /benchmark - Run performance benchmark");
    println!("  GET  /files/ - Access uploaded files");
    println!("\n💡 Try these curl commands:");
    println!("  curl http://127.0.0.1:3000/");
    println!("  curl http://127.0.0.1:3000/process/small_data.csv");
    println!("  curl http://127.0.0.1:3000/analyze/small_data.csv");
    println!("  curl -F 'file=@sample_data/small_data.csv' http://127.0.0.1:3000/upload");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "Axum CSV Processing Server",
        "description": "Demonstrates CSV processing performance using Axum + Tokio",
        "endpoints": {
            "upload": "POST /upload - Upload CSV files",
            "process": "GET /process/:filename - Process CSV with metrics",
            "analyze": "GET /analyze/:filename - Analyze CSV data",
            "compare": "GET /compare - Compare processing methods",
            "metrics": "GET /metrics - View performance metrics",
            "benchmark": "POST /benchmark - Run benchmarks"
        },
        "sample_files": [
            "/files/small_data.csv",
            "/files/medium_data.csv", 
            "/files/large_data.csv"
        ]
    }))
}

async fn upload_csv(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let timer = PerformanceTimer::new("CSV File Upload".to_string());
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("uploaded.csv").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            // Save file
            let file_path = format!("uploads/{}", filename);
            fs::create_dir_all("uploads").await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            fs::write(&file_path, &data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            // Record metrics
            let metrics = timer.finish(data.len());
            {
                let mut app_state = state.lock().unwrap();
                app_state.upload_metrics.push(metrics);
            }
            
            return Ok(Json(serde_json::json!({
                "message": "File uploaded successfully",
                "filename": filename,
                "size_bytes": data.len(),
                "path": file_path
            })));
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

async fn process_csv_file(
    axum::extract::Path(filename): axum::extract::Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let file_path = if filename.starts_with("sample_data/") {
        filename
    } else {
        format!("sample_data/{}", filename)
    };
    
    let timer = PerformanceTimer::new(format!("Processing {}", filename));
    
    // Read and parse CSV
    let content = fs::read_to_string(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: SalesRecord = result.map_err(|_| StatusCode::BAD_REQUEST)?;
        records.push(record);
    }
    
    // Cache the data
    {
        let mut app_state = state.lock().unwrap();
        app_state.cached_data.insert(filename.clone(), records.clone());
    }
    
    let metrics = timer.finish(records.len());
    
    // Store metrics
    {
        let mut app_state = state.lock().unwrap();
        app_state.processing_metrics.push(metrics.clone());
    }
    
    Ok(Json(serde_json::json!({
        "filename": filename,
        "records_processed": records.len(),
        "processing_time_ms": metrics.duration.as_millis(),
        "records_per_second": metrics.records_per_second,
        "sample_records": records.iter().take(3).collect::<Vec<_>>()
    })))
}

async fn analyze_csv(
    axum::extract::Path(filename): axum::extract::Path<String>,
    Query(params): Query<AnalysisQuery>,
    State(state): State<SharedState>,
) -> Result<Json<AnalysisResult>, StatusCode> {
    let start = std::time::Instant::now();
    
    // Get cached data or load file
    let records = {
        let app_state = state.lock().unwrap();
        app_state.cached_data.get(&filename).cloned()
    };
    
    let records = match records {
        Some(data) => data,
        None => {
            // Load file if not cached
            let file_path = format!("sample_data/{}", filename);
            let content = fs::read_to_string(&file_path)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            
            let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
            let mut loaded_records = Vec::new();
            
            for result in reader.deserialize() {
                let record: SalesRecord = result.map_err(|_| StatusCode::BAD_REQUEST)?;
                loaded_records.push(record);
            }
            
            loaded_records
        }
    };
    
    // Perform analysis
    let total_revenue: f64 = records.iter()
        .map(|r| r.price * r.quantity as f64)
        .sum();
    
    let average_price = records.iter()
        .map(|r| r.price)
        .sum::<f64>() / records.len() as f64;
    
    // Group by product for top products
    let mut product_map: HashMap<String, (f64, u32)> = HashMap::new();
    for record in &records {
        let sales = record.price * record.quantity as f64;
        let entry = product_map.entry(record.product.clone()).or_insert((0.0, 0));
        entry.0 += sales;
        entry.1 += record.quantity;
    }
    
    let mut top_products: Vec<ProductSummary> = product_map
        .into_iter()
        .map(|(product, (total_sales, quantity_sold))| ProductSummary {
            product,
            total_sales,
            quantity_sold,
        })
        .collect();
    
    top_products.sort_by(|a, b| b.total_sales.partial_cmp(&a.total_sales).unwrap());
    
    if let Some(limit) = params.limit {
        top_products.truncate(limit);
    }
    
    let processing_time = start.elapsed();
    
    Ok(Json(AnalysisResult {
        total_records: records.len(),
        total_revenue,
        average_price,
        top_products,
        processing_time_ms: processing_time.as_millis(),
    }))
}

async fn compare_processing_methods(
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    println!("🔄 Running processing method comparison...");
    
    let test_file = "sample_data/small_data.csv";
    let mut results = Vec::new();
    
    // Method 1: Standard async processing
    if let Ok(content) = fs::read_to_string(test_file).await {
        let timer = PerformanceTimer::new("Standard Async Processing".to_string());
        
        let mut reader = ReaderBuilder::new().from_reader(content.as_bytes());
        let mut count = 0;
        for result in reader.deserialize() {
            let _record: SalesRecord = result.unwrap();
            count += 1;
        }
        
        let metrics = timer.finish(count);
        results.push(serde_json::json!({
            "method": "Standard Async",
            "records": count,
            "duration_ms": metrics.duration.as_millis(),
            "records_per_second": metrics.records_per_second
        }));
    }
    
    // Method 2: Chunked processing
    if let Ok(content) = fs::read_to_string(test_file).await {
        let timer = PerformanceTimer::new("Chunked Processing".to_string());
        
        let lines: Vec<&str> = content.lines().collect();
        let chunk_size = 1000;
        let chunks: Vec<_> = lines[1..].chunks(chunk_size).collect(); // Skip header
        
        let mut total_count = 0;
        for chunk in chunks {
            let chunk_data = format!("{}\n{}", lines[0], chunk.join("\n"));
            let mut reader = ReaderBuilder::new().from_reader(chunk_data.as_bytes());
            
            for result in reader.deserialize() {
                let _record: SalesRecord = result.unwrap();
                total_count += 1;
            }
            
            // Yield to allow other tasks
            tokio::task::yield_now().await;
        }
        
        let metrics = timer.finish(total_count);
        results.push(serde_json::json!({
            "method": "Chunked Processing",
            "records": total_count,
            "duration_ms": metrics.duration.as_millis(),
            "records_per_second": metrics.records_per_second
        }));
    }
    
    Json(serde_json::json!({
        "comparison": "CSV Processing Methods",
        "test_file": test_file,
        "results": results
    }))
}

async fn get_metrics(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let app_state = state.lock().unwrap();
    
    Json(serde_json::json!({
        "upload_metrics": app_state.upload_metrics,
        "processing_metrics": app_state.processing_metrics,
        "cached_files": app_state.cached_data.keys().collect::<Vec<_>>()
    }))
}

async fn run_benchmark(State(state): State<SharedState>) -> Json<serde_json::Value> {
    println!("🏃 Running comprehensive CSV processing benchmark...");
    
    let files = ["small_data.csv", "medium_data.csv", "large_data.csv"];
    let mut benchmark_results = Vec::new();
    
    for filename in files {
        let file_path = format!("sample_data/{}", filename);
        
        if !std::path::Path::new(&file_path).exists() {
            continue;
        }
        
        println!("  Benchmarking: {}", filename);
        
        // Benchmark file reading
        let timer = PerformanceTimer::new(format!("File Read: {}", filename));
        let content = match fs::read_to_string(&file_path).await {
            Ok(content) => content,
            Err(_) => continue,
        };
        let read_metrics = timer.finish(content.len());
        
        // Benchmark CSV parsing
        let timer = PerformanceTimer::new(format!("CSV Parse: {}", filename));
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
        
        let parse_metrics = timer.finish(records.len());
        
        benchmark_results.push(serde_json::json!({
            "file": filename,
            "file_size_bytes": content.len(),
            "records_count": records.len(),
            "read_performance": {
                "duration_ms": read_metrics.duration.as_millis(),
                "bytes_per_second": content.len() as f64 / read_metrics.duration.as_secs_f64()
            },
            "parse_performance": {
                "duration_ms": parse_metrics.duration.as_millis(),
                "records_per_second": parse_metrics.records_per_second
            }
        }));
    }
    
    Json(serde_json::json!({
        "benchmark": "CSV Processing Performance",
        "timestamp": chrono::Utc::now(),
        "results": benchmark_results
    }))
}