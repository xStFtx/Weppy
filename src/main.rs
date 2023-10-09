use futures::future::join_all;
use log::{error, info};
use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::time::{sleep, Duration}; // Add imports for log macros

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger with log level "info"
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Initialize the HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(10)) // Set a timeout of 10 seconds for requests
        .build()?;

    // Read target URLs from the "targets.txt" file
    let target_urls = read_target_urls("targets.txt")?;

    // Create a shared vector to store scraped data
    let scraped_data = Arc::new(Mutex::new(Vec::new()));

    let mut tasks = Vec::new();

    for url in target_urls {
        // Clone the client for each request (for concurrency)
        let client = client.clone();

        // Clone the shared scraped_data
        let scraped_data = Arc::clone(&scraped_data);

        // Spawn a Tokio task for each request to fetch concurrently
        tasks.push(tokio::spawn(async move {
            match fetch_and_scrape(&client, &url).await {
                Ok(data) => {
                    // Lock the mutex and push the data to the shared vector
                    let mut data_guard = scraped_data.lock().unwrap();
                    data_guard.push(data);
                }
                Err(err) => error!("Error for '{}': {}", url, err),
            }
        }));

        // Throttle requests by adding a delay between requests
        sleep(Duration::from_secs(1)).await;
    }

    // Wait for all tasks to complet
    for task in tasks {
        task.await?;
    }

    // Extract the scraped data from the mutex
    let scraped_data = scraped_data.lock().unwrap();

    // Process the scraped data (e.g., saving to a database or further analysis)
    for data in scraped_data.iter() {
        info!("Page Title for '{}': {}", data.url, data.title);
        info!("Links on '{}': {:?}", data.url, data.links);
    }

    Ok(())
}

async fn fetch_and_scrape(client: &Client, url: &str) -> Result<ScrapedData, Box<dyn Error>> {
    // Send an HTTP GET request for the target URL and await the response
    let response = client.get(url).send().await?;

    // Check if the request was successful (HTTP status code 200)
    if response.status().is_success() {
        // Read the response body as bytes
        let body_bytes = response.bytes().await?;

        // Convert the response body to a string
        let body_str = String::from_utf8_lossy(&body_bytes);

        // Parse the HTML content using the scraper library
        let document = Html::parse_document(&body_str);

        // Define a CSS selector to extract specific elements
        let title_selector = Selector::parse("title").unwrap();

        // Use the selector to find and print the page title
        let page_title = document
            .select(&title_selector)
            .next()
            .map(|title_element| {
                let title_text = title_element.text().collect::<Vec<_>>().join(" ");
                title_text
            })
            .unwrap_or_else(|| "Title element not found".to_string());

        // You can add more advanced scraping logic here:
        // Example: Extract and collect all the links on the page
        let link_selector = Selector::parse("a[href]").unwrap();
        let links = document
            .select(&link_selector)
            .map(|link| link.value().attr("href").unwrap_or("N/A").to_string())
            .collect();

        Ok(ScrapedData {
            url: url.to_string(),
            title: page_title,
            links,
        })
    } else {
        Err(format!(
            "HTTP request failed with status code: {}",
            response.status()
        )
        .into())
    }
}

fn read_target_urls(filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file_content = std::fs::read_to_string(filename)?;
    let urls: Vec<String> = file_content.lines().map(|s| s.trim().to_string()).collect();
    Ok(urls)
}

#[derive(Debug)]
struct ScrapedData {
    url: String,
    title: String,
    links: Vec<String>,
}
