use reqwest::Client;
use scraper::{Html, Selector};
use tokio::main;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the HTTP client
    let client = Client::new();

    // Define the URL to scrape
    let url = "https://youtube.com"; // Replace with your target URL

    // Send an HTTP GET request and await the response
    let response = client.get(url).send().await?;

    // Check if the request was successful (HTTP status code 200)
    if response.status().is_success() {
        // Read the response body as bytes
        let body_bytes = response.bytes().await?;

        // Convert the response body to a string
        let body_str = String::from_utf8_lossy(&body_bytes);

        println!("ALL: {}", body_str);

        // Parse the HTML content using the scraper library
        let document = Html::parse_document(&body_str);

        // Define a CSS selector to extract specific elements
        let title_selector = Selector::parse("title").unwrap();

        // Use the selector to find and print the page title
        if let Some(title_element) = document.select(&title_selector).next() {
            let title_text = title_element.text().collect::<Vec<_>>().join(" ");
            println!("Page Title: {}", title_text);
        } else {
            println!("Title element not found.");
        }
    } else {
        println!(
            "HTTP request failed with status code: {}",
            response.status()
        );
    }

    Ok(())
}
