use anyhow::{Result, Context};
use console::style;
use serde::{Deserialize, Serialize};
use reqwest;

// Define the struct to match the JSON response from goosed
#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponse {
    version: String,
    config_file: String,
    sessions_dir: String,
    logs_dir: String,
    config_values: Option<std::collections::BTreeMap<String, String>>,
}

fn print_aligned(label: &str, value: &str, width: usize) {
    println!("  {:<width$} {}", label, value, width = width);
}

pub async fn handle_info(verbose: bool) -> Result<()> {
    // For PoC, assume goosed is running on localhost:3000
    let goosed_url = "http://localhost:3000/info";

    let client = reqwest::Client::new();
    let response = client.get(goosed_url).send().await?.json::<InfoResponse>().await?;

    // Use the data from the response
    let basic_padding = 15; // Adjust as needed

    println!("{}", style("Goose Version:").cyan().bold());
    print_aligned("Version:", &response.version, basic_padding);
    println!();

    println!("{}", style("Goose Locations:").cyan().bold());
    print_aligned("Config file:", &response.config_file, basic_padding);
    print_aligned("Sessions dir:", &response.sessions_dir, basic_padding);
    print_aligned("Logs dir:", &response.logs_dir, basic_padding);

    if verbose {
        println!("\n{}", style("Goose Configuration:").cyan().bold());
        if let Some(values) = response.config_values {
            if values.is_empty() {
                println!("  No configuration values set");
                println!(
                    "  Run '{}' to configure goose",
                    style("goose configure").cyan()
                );
            } else {
                if let Ok(yaml) = serde_yaml::to_string(&values) {
                    for line in yaml.lines() {
                        println!("  {}", line);
                    }
                }
            }
        } else {
            println!("  No configuration values set");
        }
    }

    Ok(())
}
