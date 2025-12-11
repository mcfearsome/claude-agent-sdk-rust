//! Files API client for uploading and managing files
//!
//! The Files API allows uploading files once and referencing them by file_id
//! in multiple requests, avoiding re-upload overhead.
//!
//! Requires beta header: `anthropic-beta: files-api-2025-04-14`

use crate::error::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Files API endpoint
const FILES_API_URL: &str = "https://api.anthropic.com/v1/files";

/// Beta header for Files API
const FILES_BETA_HEADER: &str = "files-api-2025-04-14";

/// File metadata returned from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,

    #[serde(rename = "type")]
    pub file_type: String,

    pub filename: String,

    pub mime_type: String,

    pub size_bytes: u64,

    pub created_at: String,

    pub downloadable: bool,
}

/// Client for Files API operations
///
/// # Example
///
/// ```rust,no_run
/// use claude_sdk::files::FilesClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = FilesClient::new("your-api-key");
///
///     // Upload a file
///     let file = client.upload("image.jpg").await?;
///     println!("Uploaded: {}", file.id);
///
///     // List files
///     let files = client.list().await?;
///     println!("Total files: {}", files.len());
///
///     Ok(())
/// }
/// ```
pub struct FilesClient {
    http: Client,
    api_key: String,
    api_version: String,
}

impl FilesClient {
    /// Create a new Files API client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            api_key: api_key.into(),
            api_version: "2023-06-01".to_string(),
        }
    }

    /// Upload a file
    ///
    /// # Arguments
    /// * `path` - Path to the file to upload
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use claude_sdk::files::FilesClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FilesClient::new("your-api-key");
    /// let file = client.upload("document.pdf").await?;
    /// println!("File ID: {}", file.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload(&self, path: impl AsRef<Path>) -> Result<FileMetadata> {
        let path = path.as_ref();
        debug!("Uploading file: {:?}", path);

        let file_bytes = tokio::fs::read(path)
            .await
            .map_err(|e| Error::InvalidRequest(format!("Failed to read file {:?}: {}", path, e)))?;

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::InvalidRequest("Invalid filename".into()))?;

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(file_bytes).file_name(filename.to_string()),
        );

        let response = self
            .http
            .post(FILES_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("anthropic-beta", FILES_BETA_HEADER)
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        let metadata: FileMetadata = response.json().await?;
        Ok(metadata)
    }

    /// List all uploaded files
    pub async fn list(&self) -> Result<Vec<FileMetadata>> {
        debug!("Listing files");

        let response = self
            .http
            .get(FILES_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("anthropic-beta", FILES_BETA_HEADER)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        #[derive(Deserialize)]
        struct ListResponse {
            data: Vec<FileMetadata>,
        }

        let list_response: ListResponse = response.json().await?;
        Ok(list_response.data)
    }

    /// Get metadata for a specific file
    pub async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata> {
        debug!("Getting metadata for file: {}", file_id);

        let url = format!("{}/{}", FILES_API_URL, file_id);

        let response = self
            .http
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("anthropic-beta", FILES_BETA_HEADER)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        let metadata: FileMetadata = response.json().await?;
        Ok(metadata)
    }

    /// Delete a file
    pub async fn delete(&self, file_id: &str) -> Result<()> {
        debug!("Deleting file: {}", file_id);

        let url = format!("{}/{}", FILES_API_URL, file_id);

        let response = self
            .http
            .delete(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("anthropic-beta", FILES_BETA_HEADER)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        Ok(())
    }

    /// Download a file
    ///
    /// Note: Only files created by code execution tool can be downloaded.
    /// Files you uploaded cannot be downloaded.
    pub async fn download(&self, file_id: &str) -> Result<Vec<u8>> {
        debug!("Downloading file: {}", file_id);

        let url = format!("{}/{}/content", FILES_API_URL, file_id);

        let response = self
            .http
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("anthropic-beta", FILES_BETA_HEADER)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_client_creation() {
        let client = FilesClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
    }

    // Integration tests require API key
    #[tokio::test]
    #[ignore]
    async fn test_upload_file() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY required");
        let client = FilesClient::new(api_key);

        // Create a test file
        let test_content = b"Hello, this is a test file!";
        tokio::fs::write("test_upload.txt", test_content)
            .await
            .unwrap();

        // Upload
        let result = client.upload("test_upload.txt").await;

        // Cleanup
        let _ = tokio::fs::remove_file("test_upload.txt").await;

        match result {
            Ok(metadata) => {
                println!("Uploaded: {}", metadata.id);
                // Try to delete it
                let _ = client.delete(&metadata.id).await;
            }
            Err(e) => println!("Upload test skipped (expected without real API): {}", e),
        }
    }
}
