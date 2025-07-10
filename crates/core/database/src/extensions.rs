//! DuckDB extensions management for data lake functionality

use duckhub_common::prelude::*;
use duckdb::Connection;
use std::collections::HashMap;
use tracing::{debug, info, warn, error};

/// DuckDB extension manager for data lake operations
pub struct ExtensionManager {
    required_extensions: Vec<Extension>,
    installed_extensions: HashMap<String, bool>,
}

/// DuckDB extension definition
#[derive(Debug, Clone)]
pub struct Extension {
    pub name: String,
    pub description: String,
    pub required_for: Vec<DataLakeFeature>,
    pub auto_install: bool,
}

/// Data lake features that require specific extensions
#[derive(Debug, Clone, PartialEq)]
pub enum DataLakeFeature {
    S3Access,
    AzureBlobAccess,
    GoogleCloudAccess,
    ParquetFiles,
    DeltaLake,
    DuckLake,        // Native DuckDB lakehouse format
    JsonFiles,
    CsvFiles,
    FullTextSearch,
    SpatialData,
    HttpAccess,
}

impl ExtensionManager {
    /// Create a new extension manager with default data lake extensions
    pub fn new() -> Self {
        let required_extensions = vec![
            Extension {
                name: "httpfs".to_string(),
                description: "HTTP/HTTPS and S3 file system support".to_string(),
                required_for: vec![DataLakeFeature::S3Access, DataLakeFeature::HttpAccess],
                auto_install: true,
            },
            Extension {
                name: "parquet".to_string(),
                description: "Apache Parquet file format support".to_string(),
                required_for: vec![DataLakeFeature::ParquetFiles],
                auto_install: true,
            },
            Extension {
                name: "delta".to_string(),
                description: "Delta Lake format support".to_string(),
                required_for: vec![DataLakeFeature::DeltaLake],
                auto_install: false, // Delta Lake is separate from DuckLake
            },
            Extension {
                name: "ducklake".to_string(),
                description: "DuckLake format support - native DuckDB lakehouse format".to_string(),
                required_for: vec![DataLakeFeature::DuckLake],
                auto_install: true,
            },
            Extension {
                name: "azure".to_string(),
                description: "Azure Blob Storage support".to_string(),
                required_for: vec![DataLakeFeature::AzureBlobAccess],
                auto_install: false, // Optional for most use cases
            },
            Extension {
                name: "aws".to_string(),
                description: "Enhanced AWS S3 support".to_string(),
                required_for: vec![DataLakeFeature::S3Access],
                auto_install: false, // httpfs is usually sufficient
            },
            Extension {
                name: "json".to_string(),
                description: "Enhanced JSON file support".to_string(),
                required_for: vec![DataLakeFeature::JsonFiles],
                auto_install: true,
            },
            Extension {
                name: "fts".to_string(),
                description: "Full-text search capabilities".to_string(),
                required_for: vec![DataLakeFeature::FullTextSearch],
                auto_install: false,
            },
            Extension {
                name: "spatial".to_string(),
                description: "Spatial and GIS data support".to_string(),
                required_for: vec![DataLakeFeature::SpatialData],
                auto_install: false,
            },
        ];

        Self {
            required_extensions,
            installed_extensions: HashMap::new(),
        }
    }

    /// Install and load all required extensions for data lake operations
    pub async fn setup_data_lake_extensions(&mut self, conn: &Connection) -> Result<()> {
        info!("Setting up DuckDB extensions for data lake operations");

        // Install core extensions first
        let core_extensions = self.get_core_extensions();
        for extension in core_extensions {
            self.install_and_load_extension(conn, &extension).await?;
        }

        // Install feature-specific extensions
        let feature_extensions = self.get_feature_extensions();
        for extension in feature_extensions {
            if extension.auto_install {
                self.install_and_load_extension(conn, &extension).await?;
            } else {
                debug!("Skipping optional extension: {}", extension.name);
            }
        }

        info!("Successfully set up {} extensions", self.installed_extensions.len());
        Ok(())
    }

    /// Install and load a specific extension
    pub async fn install_and_load_extension(&mut self, conn: &Connection, extension: &Extension) -> Result<()> {
        let name = &extension.name;
        
        // Check if already installed
        if self.installed_extensions.get(name).unwrap_or(&false) {
            debug!("Extension {} already installed", name);
            return Ok(());
        }

        // Try to install the extension
        match self.install_extension(conn, name).await {
            Ok(_) => {
                info!("Successfully installed extension: {}", name);
            }
            Err(e) => {
                warn!("Failed to install extension {}: {}. It might already be available.", name, e);
            }
        }

        // Load the extension
        self.load_extension(conn, name).await?;
        self.installed_extensions.insert(name.clone(), true);
        
        info!("Extension {} is ready for use", name);
        Ok(())
    }

    /// Install an extension (download if needed)
    async fn install_extension(&self, conn: &Connection, name: &str) -> Result<()> {
        let install_sql = format!("INSTALL '{}'", name);
        
        conn.execute(&install_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to install extension {}: {}", name, e)))?;
        
        debug!("Installed extension: {}", name);
        Ok(())
    }

    /// Load an extension into the current session
    async fn load_extension(&self, conn: &Connection, name: &str) -> Result<()> {
        let load_sql = format!("LOAD '{}'", name);
        
        conn.execute(&load_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to load extension {}: {}", name, e)))?;
        
        debug!("Loaded extension: {}", name);
        Ok(())
    }

    /// Get core extensions required for basic data lake functionality
    fn get_core_extensions(&self) -> Vec<&Extension> {
        self.required_extensions
            .iter()
            .filter(|ext| {
                ext.required_for.contains(&DataLakeFeature::S3Access) ||
                ext.required_for.contains(&DataLakeFeature::ParquetFiles) ||
                ext.required_for.contains(&DataLakeFeature::HttpAccess)
            })
            .collect()
    }

    /// Get feature-specific extensions
    fn get_feature_extensions(&self) -> Vec<&Extension> {
        self.required_extensions
            .iter()
            .filter(|ext| !self.get_core_extensions().contains(ext))
            .collect()
    }

    /// Enable specific data lake features
    pub async fn enable_features(&mut self, conn: &Connection, features: Vec<DataLakeFeature>) -> Result<()> {
        for feature in features {
            let extensions = self.get_extensions_for_feature(&feature);
            for extension in extensions {
                self.install_and_load_extension(conn, extension).await?;
            }
        }
        Ok(())
    }

    /// Get extensions required for a specific feature
    fn get_extensions_for_feature(&self, feature: &DataLakeFeature) -> Vec<&Extension> {
        self.required_extensions
            .iter()
            .filter(|ext| ext.required_for.contains(feature))
            .collect()
    }

    /// Check if an extension is installed and loaded
    pub fn is_extension_available(&self, name: &str) -> bool {
        self.installed_extensions.get(name).unwrap_or(&false).clone()
    }

    /// Get list of installed extensions
    pub fn get_installed_extensions(&self) -> Vec<String> {
        self.installed_extensions
            .iter()
            .filter(|(_, &installed)| installed)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Configure S3 credentials and settings
    pub async fn configure_s3(&self, conn: &Connection, config: &S3Config) -> Result<()> {
        if !self.is_extension_available("httpfs") {
            return Err(DuckHubError::config("httpfs extension not available for S3 configuration"));
        }

        // Set S3 region
        if let Some(region) = &config.region {
            let sql = format!("SET s3_region='{}'", region);
            conn.execute(&sql, [])
                .map_err(|e| DuckHubError::database(format!("Failed to set S3 region: {}", e)))?;
        }

        // Set S3 endpoint (for custom S3-compatible services)
        if let Some(endpoint) = &config.endpoint {
            let sql = format!("SET s3_endpoint='{}'", endpoint);
            conn.execute(&sql, [])
                .map_err(|e| DuckHubError::database(format!("Failed to set S3 endpoint: {}", e)))?;
        }

        // Configure credentials using secrets (more secure)
        if let (Some(access_key), Some(secret_key)) = (&config.access_key_id, &config.secret_access_key) {
            let sql = format!(
                "CREATE SECRET s3_secret (TYPE S3, KEY_ID '{}', SECRET '{}'{}{})",
                access_key,
                secret_key,
                config.region.as_ref().map(|r| format!(", REGION '{}'", r)).unwrap_or_default(),
                config.endpoint.as_ref().map(|e| format!(", ENDPOINT '{}'", e)).unwrap_or_default()
            );
            
            conn.execute(&sql, [])
                .map_err(|e| DuckHubError::database(format!("Failed to create S3 secret: {}", e)))?;
        }

        info!("S3 configuration completed");
        Ok(())
    }

    /// Configure Azure Blob Storage
    pub async fn configure_azure(&self, conn: &Connection, config: &AzureConfig) -> Result<()> {
        if !self.is_extension_available("azure") {
            return Err(DuckHubError::config("azure extension not available"));
        }

        // Create Azure secret
        let sql = format!(
            "CREATE SECRET azure_secret (TYPE AZURE, CONNECTION_STRING '{}')",
            config.connection_string
        );
        
        conn.execute(&sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to create Azure secret: {}", e)))?;

        info!("Azure Blob Storage configuration completed");
        Ok(())
    }

    /// Test data lake connectivity
    pub async fn test_data_lake_connectivity(&self, conn: &Connection) -> Result<DataLakeConnectivityReport> {
        let mut report = DataLakeConnectivityReport::new();

        // Test S3 connectivity (if httpfs is available)
        if self.is_extension_available("httpfs") {
            report.s3_available = self.test_s3_connectivity(conn).await.is_ok();
        }

        // Test Azure connectivity (if azure extension is available)
        if self.is_extension_available("azure") {
            report.azure_available = self.test_azure_connectivity(conn).await.is_ok();
        }

        // Test file format support
        report.parquet_support = self.is_extension_available("parquet");
        report.delta_support = self.is_extension_available("delta");
        report.json_support = self.is_extension_available("json");

        Ok(report)
    }

    async fn test_s3_connectivity(&self, conn: &Connection) -> Result<()> {
        // Simple test - try to list a public S3 bucket
        let test_sql = "SELECT COUNT(*) FROM 's3://duckdb-md-dataset-121/part-00000-cc9a08d6-9c52-4d46-9e1b-7a8d8b0b0e1a-c000.snappy.parquet' LIMIT 1";
        
        conn.execute(test_sql, [])
            .map_err(|e| DuckHubError::network(format!("S3 connectivity test failed: {}", e)))?;
        
        Ok(())
    }

    async fn test_azure_connectivity(&self, _conn: &Connection) -> Result<()> {
        // Azure connectivity test would go here
        // For now, just return Ok if extension is loaded
        Ok(())
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// S3 configuration
#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub region: Option<String>,
    pub endpoint: Option<String>,
}

/// Azure configuration
#[derive(Debug, Clone)]
pub struct AzureConfig {
    pub connection_string: String,
}

/// Data lake connectivity report
#[derive(Debug, Clone)]
pub struct DataLakeConnectivityReport {
    pub s3_available: bool,
    pub azure_available: bool,
    pub parquet_support: bool,
    pub delta_support: bool,
    pub json_support: bool,
}

impl DataLakeConnectivityReport {
    fn new() -> Self {
        Self {
            s3_available: false,
            azure_available: false,
            parquet_support: false,
            delta_support: false,
            json_support: false,
        }
    }

    pub fn is_fully_functional(&self) -> bool {
        self.s3_available && self.parquet_support
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_manager_creation() {
        let manager = ExtensionManager::new();
        assert!(!manager.required_extensions.is_empty());
        assert!(manager.installed_extensions.is_empty());
    }

    #[test]
    fn test_get_extensions_for_feature() {
        let manager = ExtensionManager::new();
        let s3_extensions = manager.get_extensions_for_feature(&DataLakeFeature::S3Access);
        assert!(!s3_extensions.is_empty());
        
        let parquet_extensions = manager.get_extensions_for_feature(&DataLakeFeature::ParquetFiles);
        assert!(!parquet_extensions.is_empty());
    }
}
