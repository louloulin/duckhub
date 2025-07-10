//! Schema management and registry

use duckhub_common::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Schema manager for managing table schemas
pub struct SchemaManager {
    engine: Arc<dyn DatabaseEngine>,
    registry: Arc<SchemaRegistry>,
}

impl SchemaManager {
    /// Create a new schema manager
    pub fn new(engine: Arc<dyn DatabaseEngine>) -> Self {
        Self {
            engine,
            registry: Arc::new(SchemaRegistry::new()),
        }
    }

    /// Get schema registry
    pub fn registry(&self) -> Arc<SchemaRegistry> {
        self.registry.clone()
    }
}

#[async_trait]
impl SchemaManager for SchemaManager {
    #[instrument(skip(self, schema))]
    async fn register_schema(&self, name: &str, schema: &Schema) -> Result<()> {
        // Validate schema
        self.validate_schema(schema)?;
        
        // Register in registry
        self.registry.register(name, schema.clone()).await?;
        
        info!("Registered schema: {}", name);
        Ok(())
    }

    async fn get_schema(&self, name: &str) -> Result<Option<Schema>> {
        // Try registry first
        if let Some(schema) = self.registry.get(name).await? {
            return Ok(Some(schema));
        }

        // Try to get from database
        if self.engine.table_exists(name).await? {
            let schema = self.engine.get_schema(name).await?;
            // Cache in registry
            self.registry.register(name, schema.clone()).await?;
            Ok(Some(schema))
        } else {
            Ok(None)
        }
    }

    async fn update_schema(&self, name: &str, schema: &Schema) -> Result<()> {
        // Validate new schema
        self.validate_schema(schema)?;

        // Check if schema exists
        let existing = self.get_schema(name).await?;
        if existing.is_none() {
            return Err(DuckHubError::not_found(format!("Schema '{}'", name)));
        }

        // Validate compatibility if updating existing schema
        if let Some(old_schema) = existing {
            self.validate_compatibility(&old_schema, schema).await?;
        }

        // Update in registry
        self.registry.update(name, schema.clone()).await?;
        
        info!("Updated schema: {}", name);
        Ok(())
    }

    async fn delete_schema(&self, name: &str) -> Result<()> {
        self.registry.delete(name).await?;
        info!("Deleted schema: {}", name);
        Ok(())
    }

    async fn list_schemas(&self) -> Result<Vec<String>> {
        self.registry.list().await
    }

    async fn validate_compatibility(&self, old_schema: &Schema, new_schema: &Schema) -> Result<bool> {
        // Check for breaking changes
        for old_field in &old_schema.fields {
            if let Some(new_field) = new_schema.fields.iter().find(|f| f.name == old_field.name) {
                // Check if field type changed in an incompatible way
                if !self.is_compatible_type_change(&old_field.data_type, &new_field.data_type) {
                    return Err(DuckHubError::validation(format!(
                        "Incompatible type change for field '{}': {:?} -> {:?}",
                        old_field.name, old_field.data_type, new_field.data_type
                    )));
                }

                // Check if nullable changed from true to false
                if old_field.nullable && !new_field.nullable {
                    return Err(DuckHubError::validation(format!(
                        "Cannot change field '{}' from nullable to non-nullable",
                        old_field.name
                    )));
                }
            } else {
                // Field was removed - this might be breaking
                return Err(DuckHubError::validation(format!(
                    "Field '{}' was removed from schema",
                    old_field.name
                )));
            }
        }

        Ok(true)
    }
}

impl SchemaManager {
    /// Validate a schema
    fn validate_schema(&self, schema: &Schema) -> Result<()> {
        if schema.fields.is_empty() {
            return Err(DuckHubError::validation("Schema must have at least one field"));
        }

        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for field in &schema.fields {
            if !field_names.insert(&field.name) {
                return Err(DuckHubError::validation(format!(
                    "Duplicate field name: {}",
                    field.name
                )));
            }
        }

        // Validate field names
        for field in &schema.fields {
            if field.name.is_empty() {
                return Err(DuckHubError::validation("Field name cannot be empty"));
            }
            
            // Check for valid identifier
            if !field.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(DuckHubError::validation(format!(
                    "Invalid field name: {}",
                    field.name
                )));
            }
        }

        // Validate primary key references
        if let Some(pk_fields) = &schema.primary_key {
            for pk_field in pk_fields {
                if !schema.fields.iter().any(|f| &f.name == pk_field) {
                    return Err(DuckHubError::validation(format!(
                        "Primary key field '{}' not found in schema",
                        pk_field
                    )));
                }
            }
        }

        // Validate index references
        for index in &schema.indexes {
            for index_field in &index.columns {
                if !schema.fields.iter().any(|f| &f.name == index_field) {
                    return Err(DuckHubError::validation(format!(
                        "Index field '{}' not found in schema",
                        index_field
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if type change is compatible
    fn is_compatible_type_change(&self, old_type: &DataType, new_type: &DataType) -> bool {
        match (old_type, new_type) {
            // Same type is always compatible
            (a, b) if a == b => true,
            
            // Widening integer types
            (DataType::Int8, DataType::Int16) |
            (DataType::Int8, DataType::Int32) |
            (DataType::Int8, DataType::Int64) |
            (DataType::Int16, DataType::Int32) |
            (DataType::Int16, DataType::Int64) |
            (DataType::Int32, DataType::Int64) => true,
            
            // Widening unsigned integer types
            (DataType::UInt8, DataType::UInt16) |
            (DataType::UInt8, DataType::UInt32) |
            (DataType::UInt8, DataType::UInt64) |
            (DataType::UInt16, DataType::UInt32) |
            (DataType::UInt16, DataType::UInt64) |
            (DataType::UInt32, DataType::UInt64) => true,
            
            // Widening float types
            (DataType::Float32, DataType::Float64) => true,
            
            // Integer to float (with potential precision loss)
            (DataType::Int32, DataType::Float64) |
            (DataType::Int64, DataType::Float64) => true,
            
            // Everything else is incompatible
            _ => false,
        }
    }
}

/// Schema registry for caching and managing schemas
pub struct SchemaRegistry {
    schemas: RwLock<HashMap<String, Schema>>,
    metadata: RwLock<HashMap<String, SchemaMetadata>>,
}

#[derive(Debug, Clone)]
struct SchemaMetadata {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: u32,
}

impl SchemaRegistry {
    /// Create a new schema registry
    pub fn new() -> Self {
        Self {
            schemas: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Register a schema
    pub async fn register(&self, name: &str, schema: Schema) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;

        let now = now();
        let version = if let Some(existing_meta) = metadata.get(name) {
            existing_meta.version + 1
        } else {
            1
        };

        schemas.insert(name.to_string(), schema);
        metadata.insert(name.to_string(), SchemaMetadata {
            created_at: metadata.get(name).map(|m| m.created_at).unwrap_or(now),
            updated_at: now,
            version,
        });

        debug!("Registered schema '{}' version {}", name, version);
        Ok(())
    }

    /// Get a schema
    pub async fn get(&self, name: &str) -> Result<Option<Schema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas.get(name).cloned())
    }

    /// Update a schema
    pub async fn update(&self, name: &str, schema: Schema) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;

        if !schemas.contains_key(name) {
            return Err(DuckHubError::not_found(format!("Schema '{}'", name)));
        }

        let now = now();
        let version = metadata.get(name).map(|m| m.version + 1).unwrap_or(1);

        schemas.insert(name.to_string(), schema);
        metadata.insert(name.to_string(), SchemaMetadata {
            created_at: metadata.get(name).map(|m| m.created_at).unwrap_or(now),
            updated_at: now,
            version,
        });

        debug!("Updated schema '{}' to version {}", name, version);
        Ok(())
    }

    /// Delete a schema
    pub async fn delete(&self, name: &str) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;

        schemas.remove(name);
        metadata.remove(name);

        debug!("Deleted schema '{}'", name);
        Ok(())
    }

    /// List all schema names
    pub async fn list(&self) -> Result<Vec<String>> {
        let schemas = self.schemas.read().await;
        Ok(schemas.keys().cloned().collect())
    }

    /// Get schema metadata
    pub async fn get_metadata(&self, name: &str) -> Result<Option<SchemaMetadata>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(name).cloned())
    }

    /// Get schema version
    pub async fn get_version(&self, name: &str) -> Result<Option<u32>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(name).map(|m| m.version))
    }

    /// Clear all schemas
    pub async fn clear(&self) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;

        schemas.clear();
        metadata.clear();

        debug!("Cleared all schemas from registry");
        Ok(())
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let schemas = self.schemas.read().await;
        let metadata = self.metadata.read().await;

        RegistryStats {
            total_schemas: schemas.len(),
            total_fields: schemas.values().map(|s| s.fields.len()).sum(),
            average_fields_per_schema: if schemas.is_empty() {
                0.0
            } else {
                schemas.values().map(|s| s.fields.len()).sum::<usize>() as f64 / schemas.len() as f64
            },
        }
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_schemas: usize,
    pub total_fields: usize,
    pub average_fields_per_schema: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema() -> Schema {
        Schema {
            fields: vec![
                Field {
                    name: "id".to_string(),
                    data_type: DataType::Int64,
                    nullable: false,
                    default_value: None,
                    description: Some("Primary key".to_string()),
                },
                Field {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    nullable: false,
                    default_value: None,
                    description: Some("Name field".to_string()),
                },
            ],
            primary_key: Some(vec!["id".to_string()]),
            indexes: vec![],
        }
    }

    #[tokio::test]
    async fn test_schema_registry() {
        let registry = SchemaRegistry::new();
        let schema = create_test_schema();

        // Test register
        registry.register("test_table", schema.clone()).await.unwrap();

        // Test get
        let retrieved = registry.get("test_table").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().fields.len(), 2);

        // Test list
        let schemas = registry.list().await.unwrap();
        assert_eq!(schemas.len(), 1);
        assert!(schemas.contains(&"test_table".to_string()));

        // Test metadata
        let metadata = registry.get_metadata("test_table").await.unwrap();
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().version, 1);

        // Test delete
        registry.delete("test_table").await.unwrap();
        let retrieved = registry.get("test_table").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_schema_validation() {
        let manager = SchemaManager::new(Arc::new(crate::duckdb::DuckDBEngine::new(
            DatabaseConfig {
                duckdb_path: ":memory:".to_string(),
                memory_limit: None,
                threads: None,
                max_memory: None,
                temp_directory: None,
                extensions: Vec::new(),
                pool: PoolConfig::default(),
            }
        ).unwrap()));

        let valid_schema = create_test_schema();
        assert!(manager.validate_schema(&valid_schema).is_ok());

        // Test empty schema
        let empty_schema = Schema {
            fields: vec![],
            primary_key: None,
            indexes: vec![],
        };
        assert!(manager.validate_schema(&empty_schema).is_err());

        // Test duplicate field names
        let duplicate_schema = Schema {
            fields: vec![
                Field {
                    name: "id".to_string(),
                    data_type: DataType::Int64,
                    nullable: false,
                    default_value: None,
                    description: None,
                },
                Field {
                    name: "id".to_string(),
                    data_type: DataType::String,
                    nullable: false,
                    default_value: None,
                    description: None,
                },
            ],
            primary_key: None,
            indexes: vec![],
        };
        assert!(manager.validate_schema(&duplicate_schema).is_err());
    }

    #[test]
    fn test_type_compatibility() {
        let manager = SchemaManager::new(Arc::new(crate::duckdb::DuckDBEngine::new(
            DatabaseConfig {
                duckdb_path: ":memory:".to_string(),
                memory_limit: None,
                threads: None,
                max_memory: None,
                temp_directory: None,
                extensions: Vec::new(),
                pool: PoolConfig::default(),
            }
        ).unwrap()));

        // Compatible changes
        assert!(manager.is_compatible_type_change(&DataType::Int32, &DataType::Int64));
        assert!(manager.is_compatible_type_change(&DataType::Float32, &DataType::Float64));

        // Incompatible changes
        assert!(!manager.is_compatible_type_change(&DataType::Int64, &DataType::Int32));
        assert!(!manager.is_compatible_type_change(&DataType::String, &DataType::Int32));
    }
}
