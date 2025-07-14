use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub records_processed: usize,
    pub duration: Duration,
    pub records_per_second: f64,
    pub memory_estimate_mb: f64,
}

impl PerformanceMetrics {
    pub fn new(operation: String, records_processed: usize, duration: Duration) -> Self {
        let records_per_second = records_processed as f64 / duration.as_secs_f64();
        let memory_estimate_mb = (records_processed * 100) as f64 / 1_000_000.0; // Rough estimate
        
        Self {
            operation,
            records_processed,
            duration,
            records_per_second,
            memory_estimate_mb,
        }
    }

    pub fn display(&self) {
        println!("📊 Performance Metrics for: {}", self.operation);
        println!("   Records processed: {}", self.records_processed);
        println!("   Duration: {:?}", self.duration);
        println!("   Records/second: {:.2}", self.records_per_second);
        println!("   Est. memory usage: {:.2} MB", self.memory_estimate_mb);
        println!();
    }
}

pub struct PerformanceTimer {
    start: Instant,
    operation: String,
}

impl PerformanceTimer {
    pub fn new(operation: String) -> Self {
        println!("⏱️  Starting: {}", operation);
        Self {
            start: Instant::now(),
            operation,
        }
    }

    pub fn finish(self, records_processed: usize) -> PerformanceMetrics {
        let duration = self.start.elapsed();
        let metrics = PerformanceMetrics::new(self.operation, records_processed, duration);
        metrics.display();
        metrics
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesRecord {
    pub id: u32,
    pub customer_name: String,
    pub product: String,
    pub quantity: u32,
    pub price: f64,
    pub date: String,
    pub region: String,
}