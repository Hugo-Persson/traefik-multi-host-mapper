use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub async fn send_to_discord_webhook(
    webhook_url: &str,
    content: &str,
    file_content: Option<String>, // Optional file content as a String
    file_name: Option<&str>,      // Optional file name for the uploaded file
) -> Result<(), Box<dyn Error>> {
    // Create an HTTP client
    let client = Client::new();

    // Start building the multipart form
    let mut form = Form::new().text("payload_json", json!({ "content": content }).to_string()); // Add the JSON payload

    // If file content is provided, add it to the form
    if let Some(content) = file_content {
        let file_name = file_name.unwrap_or("file.txt"); // Use provided name or default
        let part = Part::bytes(content.into_bytes()).file_name(file_name.to_string());
        form = form.part("file", part); // Add the file part to the form
    }

    // Send the POST request with the multipart form
    let response = client.post(webhook_url).multipart(form).send().await?;

    // Check if the response is successful
    if response.status().is_success() {
        println!("Message sent successfully!");
    } else {
        eprintln!(
            "Failed to send message. Status: {}, body: {}",
            response.status(),
            response.text().await?
        );
    }

    Ok(())
}
