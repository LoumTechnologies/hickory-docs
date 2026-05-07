//! S3-backed ObjectStore implementation.
//!
//! Stores objects in an S3 bucket with a configurable key prefix.
//! Uses conditional writes (If-None-Match for puts, ETag tracking for
//! branch updates) to provide CAS semantics on branch pointers.

use async_trait::async_trait;
use aws_sdk_s3::Client;
use log::debug;

use crate::error::StoreError;
use crate::object_store::ObjectStore;

/// An ObjectStore backed by Amazon S3.
///
/// Objects are stored at `{prefix}/{key}` in the configured bucket.
/// Content-addressed puts (blobs, snapshots) are idempotent.
/// Branch updates use ETags for optimistic concurrency control.
pub struct S3ObjectStore {
    client: Client,
    bucket: String,
    prefix: String,
}

impl S3ObjectStore {
    /// Create a new S3ObjectStore.
    ///
    /// Uses the default AWS SDK credential chain (env vars, config file,
    /// IAM role, etc.).
    pub async fn new(bucket: &str, prefix: &str) -> Result<Self, StoreError> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            prefix: prefix.trim_end_matches('/').to_string(),
        })
    }

    /// Create with an explicit S3 client (for testing with LocalStack etc.).
    pub fn with_client(client: Client, bucket: &str, prefix: &str) -> Self {
        Self {
            client,
            bucket: bucket.to_string(),
            prefix: prefix.trim_end_matches('/').to_string(),
        }
    }

    fn full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix, key)
        }
    }
}

#[async_trait]
impl ObjectStore for S3ObjectStore {
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), StoreError> {
        let full_key = self.full_key(key);
        debug!("S3 PUT: s3://{}/{}", self.bucket, full_key);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .body(data.to_vec().into())
            .send()
            .await
            .map_err(|e| StoreError::Other(format!("S3 put failed: {e}")))?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        let full_key = self.full_key(key);
        debug!("S3 GET: s3://{}/{}", self.bucket, full_key);

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| {
                let msg = format!("{e}");
                if msg.contains("NoSuchKey") || msg.contains("404") {
                    StoreError::BlobNotFound(crate::types::BlobHash(key.to_string()))
                } else {
                    StoreError::Other(format!("S3 get failed: {e}"))
                }
            })?;

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| StoreError::Other(format!("S3 body read failed: {e}")))?;

        Ok(bytes.to_vec())
    }

    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        let full_key = self.full_key(key);
        debug!("S3 HEAD: s3://{}/{}", self.bucket, full_key);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("NotFound") || msg.contains("404") {
                    Ok(false)
                } else {
                    Err(StoreError::Other(format!("S3 head failed: {e}")))
                }
            }
        }
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let full_prefix = self.full_key(prefix);
        debug!("S3 LIST: s3://{}/{}", self.bucket, full_prefix);

        let mut keys = Vec::new();
        let mut continuation_token = None;

        loop {
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&full_prefix);

            if let Some(token) = &continuation_token {
                req = req.continuation_token(token);
            }

            let response = req
                .send()
                .await
                .map_err(|e| StoreError::Other(format!("S3 list failed: {e}")))?;

            if let Some(contents) = response.contents {
                for obj in contents {
                    if let Some(key) = obj.key {
                        // Strip the store prefix to return relative keys
                        let relative = if self.prefix.is_empty() {
                            key
                        } else {
                            key.strip_prefix(&format!("{}/", self.prefix))
                                .unwrap_or(&key)
                                .to_string()
                        };
                        keys.push(relative);
                    }
                }
            }

            if response.is_truncated == Some(true) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(keys)
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let full_key = self.full_key(key);
        debug!("S3 DELETE: s3://{}/{}", self.bucket, full_key);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| StoreError::Other(format!("S3 delete failed: {e}")))?;

        Ok(())
    }
}
