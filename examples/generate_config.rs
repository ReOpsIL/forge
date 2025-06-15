// This example generates a new blocks_config.json file
// Run it with: cargo run --example generate_config

use forge::block_config::generate_sample_config;

fn main() {
    println!("Generating new blocks_config.json file...");
    
    // Generate the sample config file
    match generate_sample_config("blocks_config.json") {
        Ok(_) => println!("Sample config generated successfully"),
        Err(e) => eprintln!("Failed to generate sample config: {}", e),
    }
    
    println!("Done. The new blocks_config.json file has been generated.");
}