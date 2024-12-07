use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub async fn send_to_discord_webhook(
    webhook_url: &str,
    content: &str,
) -> Result<(), Box<dyn Error>> {
    // Create a JSON payload
    let payload = json!({
        "content": content,
        // Add additional fields if needed, e.g., embeds, username, etc.
    });

    // Create an HTTP client
    let client = Client::new();

    // Send the POST request
    let response = client.post(webhook_url).json(&payload).send().await?;

    // Check if the response is successful
    if response.status().is_success() {
        println!("Message sent successfully!");
    } else {
        eprintln!("Failed to send message. Status: {}", response.status());
    }

    Ok(())
}
