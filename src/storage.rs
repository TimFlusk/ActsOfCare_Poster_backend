use anyhow::{Context, Result};
use s3::{Bucket, Region, creds::Credentials};

const BUCKET_NAME: &str = "tng-images";

pub struct MinioStorage {
    bucket: Box<Bucket>,
}

impl MinioStorage {
    pub fn new() -> Result<Self> {
        let endpoint = std::env::var("MINIO_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        let access_key = std::env::var("MINIO_ACCESS_KEY")
            .unwrap_or_else(|_| "minioadmin".to_string());
        let secret_key = std::env::var("MINIO_SECRET_KEY")
            .unwrap_or_else(|_| "minioadmin".to_string());

        let region = Region::Custom {
            region: "us-east-1".to_string(),
            endpoint,
        };

        let credentials = Credentials::new(
            Some(&access_key),
            Some(&secret_key),
            None,
            None,
            None,
        )?;

        let bucket = Bucket::new(BUCKET_NAME, region, credentials)
            .context("Failed to create bucket handle")?
            .with_path_style(); // Required for MinIO

        tracing::info!("MinIO storage initialised, bucket: {}", BUCKET_NAME);
        Ok(Self { bucket })
    }

    /// Create the bucket if it does not exist.
    pub async fn ensure_bucket(&self) -> Result<()> {
        match self.bucket.exists().await {
            Ok(true) => {
                tracing::info!("Bucket '{}' already exists", BUCKET_NAME);
            }
            Ok(false) => {
                self.bucket.create().await
                    .context("Failed to create bucket")?;
                tracing::info!("Bucket '{}' created", BUCKET_NAME);
            }
            Err(e) => {
                // Non-fatal: log and continue — the server can still receive requests.
                tracing::warn!("Could not check bucket existence: {}", e);
            }
        }
        Ok(())
    }

    /// Upload raw image bytes under the given key (the GUID filename).
    pub async fn put_image(&self, key: &str, data: Vec<u8>) -> Result<()> {
        self.bucket
            .put_object(key, &data)
            .await
            .with_context(|| format!("Failed to upload image '{}'", key))?;
        tracing::debug!("Stored image: {}", key);
        Ok(())
    }
}
