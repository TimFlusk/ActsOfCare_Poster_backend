use anyhow::{Context, Result};
use minio::s3::MinioClient;
use minio::s3::builders::ObjectContent;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::{ObjectKey, S3Api};

const BUCKET_NAME: &str = "tng-images";

const POSTER_BUCKET: &str = "acts-of-care-posters";
const POTRAIT_BUCKET: &str = "acts-of-care-portraits";


pub struct MinioStorage {
    client: MinioClient,
}

impl MinioStorage {
    pub fn new() -> Result<Self> {
        let endpoint = std::env::var("MINIO_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        let access_key = std::env::var("MINIO_ACCESS_KEY")
            .unwrap_or_else(|_| "minioadmin".to_string());
        let secret_key = std::env::var("MINIO_SECRET_KEY")
            .unwrap_or_else(|_| "minioadmin".to_string());

        let base_url: BaseUrl = endpoint.parse()
            .context("Invalid MINIO_ENDPOINT")?;

        let provider = StaticProvider::new(&access_key, &secret_key, None);

        let client = MinioClient::new(base_url, Some(provider), None, None)
            .context("Failed to create MinIO client")?;

        tracing::info!("MinIO client initialised, bucket: {}", BUCKET_NAME);
        Ok(Self { client })
    }

    /// Create the bucket if it does not already exist.
    pub async fn ensure_bucket(&self) -> Result<()> {
        let exists = self.client
            .bucket_exists(BUCKET_NAME)
            .unwrap()
            .build()
            .send()
            .await
            .context("Failed to check bucket existence")?
            .exists();

        if exists {
            tracing::info!("Bucket '{}' already exists", BUCKET_NAME);
        } else {
            self.client
                .create_bucket(BUCKET_NAME)
                .unwrap()
                .build()
                .send()
                .await
                .context("Failed to create bucket")?;
            tracing::info!("Bucket '{}' created", BUCKET_NAME);
        }

        Ok(())
    }

    /// Upload raw PNG bytes under the given key (the GUID filename).
    pub async fn put_image(&self, key: &str, data: Vec<u8>) -> Result<()> {
        let object_key = ObjectKey::new(key)
            .with_context(|| format!("Invalid object key: {}", key))?;

        let content = ObjectContent::from(data);

        self.client
            .put_object_content(BUCKET_NAME, object_key, content)
            .with_context(|| format!("Failed to build put_object_content for '{}'", key))?
            .build()
            .send()
            .await
            .with_context(|| format!("Failed to upload image '{}'", key))?;

        tracing::debug!("Stored image: {}", key);
        Ok(())
    }

    pub async fn put_portrait(&self, key: &str, data: Vec<u8>) -> Result<()> {
        let object_key = ObjectKey::new(key)
            .with_context(|| format!("Invalid object key: {}", key))?;

        let content = ObjectContent::from(data);

        self.client
            .put_object_content(POTRAIT_BUCKET, object_key, content)
            .with_context(|| format!("Failed to build put_object_content for '{}'", key))?
            .build()
            .send()
            .await
            .with_context(|| format!("Failed to upload image '{}'", key))?;

        tracing::debug!("Stored image: {}", key);
        Ok(())
    }

    pub async fn put_poster(&self, key: &str, data: Vec<u8>) -> Result<()> {
        let object_key = ObjectKey::new(key)
            .with_context(|| format!("Invalid object key: {}", key))?;

        let content = ObjectContent::from(data);

        self.client
            .put_object_content(POSTER_BUCKET, object_key, content)
            .with_context(|| format!("Failed to build put_object_content for '{}'", key))?
            .build()
            .send()
            .await
            .with_context(|| format!("Failed to upload image '{}'", key))?;

        tracing::debug!("Stored image: {}", key);
        Ok(())
    }
}
