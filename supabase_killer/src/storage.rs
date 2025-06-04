//! # 📁 Storage Service - File Management with ZIK_ZAK
//!
//! Replaces Supabase Storage with ZIK_ZAK's accounting-based file system.
//! Every file is an account, every byte is a balance!

use anyhow::{anyhow, Result};
use axum::{http::HeaderMap, response::Response};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;
use zik_zak::accounting::ZikZakEngine;

#[derive(Clone)]
pub struct StorageService {
    zikzak: ZikZakEngine,
}

impl StorageService {
    pub async fn new() -> Result<Self> {
        let mut zikzak = ZikZakEngine::new("storage_db").await?;
        
        // Initialize storage system
        let _ = zikzak.transfer(
            "system:genesis",
            "storage:system:initialized",
            1,
            HashMap::from([("operation".to_string(), "storage_init".to_string())])
        ).await;

        Ok(Self { zikzak })
    }

    /// List all buckets
    pub async fn list_buckets(&self, _headers: HeaderMap) -> Result<Value> {
        // In ZIK_ZAK, buckets are just namespace prefixes
        Ok(json!([
            {
                "id": "default",
                "name": "default",
                "public": true,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "private",
                "name": "private", 
                "public": false,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]))
    }

    /// Create a new bucket
    pub async fn create_bucket(&mut self, payload: Value, _headers: HeaderMap) -> Result<Value> {
        let bucket_name = payload["name"].as_str()
            .ok_or_else(|| anyhow!("Bucket name required"))?;
        let is_public = payload["public"].as_bool().unwrap_or(false);

        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "create_bucket".to_string());
        metadata.insert("bucket_name".to_string(), bucket_name.to_string());
        metadata.insert("public".to_string(), is_public.to_string());

        // Create bucket in ZIK_ZAK
        self.zikzak.transfer(
            "system:genesis",
            &format!("bucket:{}:existence", bucket_name),
            1,
            metadata
        ).await?;

        Ok(json!({
            "name": bucket_name,
            "id": bucket_name,
            "public": is_public,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now()
        }))
    }

    /// Delete a bucket
    pub async fn delete_bucket(&mut self, bucket: String, _headers: HeaderMap) -> Result<Value> {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "delete_bucket".to_string());
        metadata.insert("bucket_name".to_string(), bucket.clone());

        // Move bucket to void (soft delete)
        self.zikzak.transfer(
            &format!("bucket:{}:existence", bucket),
            "system:void",
            1,
            metadata
        ).await?;

        Ok(json!({
            "message": format!("Bucket {} deleted", bucket)
        }))
    }

    /// List objects in a bucket
    pub async fn list_objects(
        &self, 
        bucket: String,
        _params: HashMap<String, String>,
        _headers: HeaderMap
    ) -> Result<Value> {
        // Return mock objects for demonstration
        Ok(json!([
            {
                "name": "example.txt",
                "id": "example.txt",
                "updated_at": "2024-01-01T00:00:00Z",
                "created_at": "2024-01-01T00:00:00Z",
                "last_accessed_at": "2024-01-01T00:00:00Z",
                "metadata": {
                    "size": 1024,
                    "mimetype": "text/plain"
                },
                "bucket_id": bucket
            }
        ]))
    }

    /// Get an object
    pub async fn get_object(
        &self,
        bucket: String,
        path: String,
        _headers: HeaderMap
    ) -> Result<Response> {
        // Check if file exists in ZIK_ZAK
        let file_exists = self.zikzak.get_balance(&format!("file:{}:{}:existence", bucket, path)).await?;
        
        if file_exists == 0 {
            return Err(anyhow!("File not found"));
        }

        // Return mock file content
        let content = format!("🦖 ZIK_ZAK File Content for {}/{}", bucket, path);
        
        let response = Response::builder()
            .header("content-type", "text/plain")
            .header("content-length", content.len())
            .body(content.into())
            .unwrap();

        Ok(response)
    }

    /// Upload an object
    pub async fn upload_object(
        &mut self,
        bucket: String,
        path: String,
        body: axum::body::Bytes,
        _headers: HeaderMap
    ) -> Result<Value> {
        let file_id = Uuid::new_v4().to_string();
        let file_size = body.len() as i64;

        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "upload_file".to_string());
        metadata.insert("bucket".to_string(), bucket.clone());
        metadata.insert("path".to_string(), path.clone());
        metadata.insert("file_id".to_string(), file_id.clone());
        metadata.insert("size".to_string(), file_size.to_string());

        // Create file existence record
        self.zikzak.transfer(
            "system:genesis",
            &format!("file:{}:{}:existence", bucket, path),
            1,
            metadata.clone()
        ).await?;

        // Store file size
        self.zikzak.transfer(
            "system:genesis",
            &format!("file:{}:{}:size", bucket, path),
            file_size,
            metadata
        ).await?;

        Ok(json!({
            "Key": format!("{}/{}", bucket, path),
            "id": file_id,
            "bucket_id": bucket,
            "storage_path": path,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
            "metadata": {
                "size": file_size,
                "mimetype": "application/octet-stream"
            }
        }))
    }

    /// Delete an object
    pub async fn delete_object(
        &mut self,
        bucket: String,
        path: String,
        _headers: HeaderMap
    ) -> Result<Value> {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "delete_file".to_string());
        metadata.insert("bucket".to_string(), bucket.clone());
        metadata.insert("path".to_string(), path.clone());

        // Move file to void (soft delete)
        self.zikzak.transfer(
            &format!("file:{}:{}:existence", bucket, path),
            "system:void",
            1,
            metadata
        ).await?;

        Ok(json!({
            "message": format!("File {}/{} deleted", bucket, path)
        }))
    }
}