use csv::Writer;
use rand::Rng;
use std::error::Error;
use std::fs::File;
use clap::{Arg, Command};

#[derive(Debug)]
struct SalesRecord {
    id: u32,
    customer_name: String,
    product: String,
    quantity: u32,
    price: f64,
    date: String,
    region: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("CSV Data Generator")
        .about("Generates sample CSV files for performance testing")
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_name("SIZE")
                .help("Size of CSV to generate")
                .value_parser(["small", "medium", "large"])
                .default_value("medium")
        )
        .get_matches();

    let size = matches.get_one::<String>("size").unwrap();
    
    match size.as_str() {
        "small" => generate_csv("sample_data/small_data.csv", 1_000)?,
        "medium" => generate_csv("sample_data/medium_data.csv", 100_000)?,
        "large" => generate_csv("sample_data/large_data.csv", 1_000_000)?,
        _ => unreachable!(),
    }

    println!("✅ Generated {} CSV file successfully!", size);
    Ok(())
}

fn generate_csv(filename: &str, record_count: u32) -> Result<(), Box<dyn Error>> {
    // Create directory if it doesn't exist
    std::fs::create_dir_all("sample_data")?;
    
    let file = File::create(filename)?;
    let mut writer = Writer::from_writer(file);
    let mut rng = rand::thread_rng();
    
    let products = ["Laptop", "Mouse", "Keyboard", "Monitor", "Headphones", "Tablet", "Phone", "Speaker"];
    let regions = ["North", "South", "East", "West", "Central"];
    let first_names = ["John", "Jane", "Bob", "Alice", "Charlie", "Diana", "Eve", "Frank"];
    let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis"];

    // Write header
    writer.write_record(&["id", "customer_name", "product", "quantity", "price", "date", "region"])?;

    println!("Generating {} records for {}...", record_count, filename);
    
    for i in 1..=record_count {
        let record = SalesRecord {
            id: i,
            customer_name: format!("{} {}", 
                first_names[rng.gen_range(0..first_names.len())],
                last_names[rng.gen_range(0..last_names.len())]
            ),
            product: products[rng.gen_range(0..products.len())].to_string(),
            quantity: rng.gen_range(1..=10),
            price: rng.gen_range(10.0..=1000.0),
            date: format!("2024-{:02}-{:02}", rng.gen_range(1..=12), rng.gen_range(1..=28)),
            region: regions[rng.gen_range(0..regions.len())].to_string(),
        };

        writer.write_record(&[
            &record.id.to_string(),
            &record.customer_name,
            &record.product,
            &record.quantity.to_string(),
            &format!("{:.2}", record.price),
            &record.date,
            &record.region,
        ])?;

        if i % 100_000 == 0 {
            println!("  Progress: {} records written", i);
        }
    }

    writer.flush()?;
    println!("✅ Successfully generated {} with {} records", filename, record_count);
    Ok(())
}