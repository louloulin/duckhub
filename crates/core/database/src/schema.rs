//! Schema management for DuckLake

use duckhub_common::prelude::*;
use duckhub_common::types::{Schema, Field, DataType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Schema metadata
#[derive(Debug, Clone)]
pub struct SchemaMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

/// Schema change type
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaChangeType {
    AddColumn,
    DropColumn,
    AlterColumnType,
    AlterColumnNullability,
}

/// Schema change
#[derive(Debug, Clone)]
pub struct SchemaChange {
    pub change_type: SchemaChangeType,
    pub column_name: String,
    pub old_type: Option<DataType>,
    pub new_type: Option<DataType>,
    pub timestamp: DateTime<Utc>,
}

/// Schema compatibility check result
#[derive(Debug, Clone)]
pub enum CompatibilityCheck {
    Compatible,
    IncompatibleType { old_type: DataType, new_type: DataType },
    IncompatibleNullability { old_nullable: bool, new_nullable: bool },
}

/// Schema manager
#[derive(Debug)]
pub struct SchemaManager {
    schemas: RwLock<HashMap<String, Schema>>,
    metadata: RwLock<HashMap<String, SchemaMetadata>>,
    changes: RwLock<HashMap<String, Vec<SchemaChange>>>,
}

impl SchemaManager {
    /// Create a new schema manager
    pub fn new() -> Self {
        Self {
            schemas: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            changes: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a schema
    pub async fn register_schema(&self, name: &str, schema: Schema) -> Result<()> {
        let now = Utc::now();
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;
        
        // Fix borrowing issue by getting version first
        let version = metadata.get(name).map(|m| m.version + 1).unwrap_or(1);
        let created_at = metadata.get(name)
            .map(|m| m.created_at)
            .unwrap_or(now);

        metadata.insert(name.to_string(), SchemaMetadata {
            created_at,
            updated_at: now,
            version,
        });
        
        schemas.insert(name.to_string(), schema);
        Ok(())
    }
    
    /// Get a schema
    pub async fn get_schema(&self, name: &str) -> Result<Schema> {
        let schemas = self.schemas.read().await;
        schemas.get(name)
            .cloned()
            .ok_or_else(|| DuckHubError::database(format!("Schema not found: {}", name)))
    }
    
    /// Update a schema
    pub async fn update_schema(&self, name: &str, schema: Schema) -> Result<()> {
        let now = Utc::now();
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;
        
        // Fix borrowing issue by getting version first
        let version = metadata.get(name).map(|m| m.version + 1).unwrap_or(1);
        let created_at = metadata.get(name)
            .map(|m| m.created_at)
            .unwrap_or(now);

        metadata.insert(name.to_string(), SchemaMetadata {
            created_at,
            updated_at: now,
            version,
        });
        
        schemas.insert(name.to_string(), schema);
        Ok(())
    }
    
    /// Delete a schema
    pub async fn delete_schema(&self, name: &str) -> Result<()> {
        let mut schemas = self.schemas.write().await;
        let mut metadata = self.metadata.write().await;
        let mut changes = self.changes.write().await;
        
        schemas.remove(name);
        metadata.remove(name);
        changes.remove(name);
        
        Ok(())
    }
    
    /// Get schema metadata
    pub async fn get_schema_metadata(&self, name: &str) -> Result<SchemaMetadata> {
        let _metadata = self.metadata.read().await;
        
        // Mock implementation
        Ok(SchemaMetadata {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        })
    }
    
    /// Check if a schema exists
    pub async fn schema_exists(&self, name: &str) -> bool {
        let schemas = self.schemas.read().await;
        schemas.contains_key(name)
    }
    
    /// Get all schema names
    pub async fn get_schema_names(&self) -> Vec<String> {
        let schemas = self.schemas.read().await;
        schemas.keys().cloned().collect()
    }
    
    /// Check schema compatibility
    pub fn check_schema_compatibility(old_schema: &Schema, new_schema: &Schema) -> Result<Vec<CompatibilityCheck>> {
        let mut results = Vec::new();
        
        // Check existing columns
        for old_field in &old_schema.fields {
            if let Some(new_field) = new_schema.fields.iter().find(|f| f.name == old_field.name) {
                // Check type compatibility
                if old_field.data_type != new_field.data_type {
                    results.push(CompatibilityCheck::IncompatibleType {
                        old_type: old_field.data_type.clone(),
                        new_type: new_field.data_type.clone(),
                    });
                }
                
                // Check nullability
                if old_field.nullable && !new_field.nullable {
                    results.push(CompatibilityCheck::IncompatibleNullability {
                        old_nullable: old_field.nullable,
                        new_nullable: new_field.nullable,
                    });
                }
            }
        }
        
        if results.is_empty() {
            results.push(CompatibilityCheck::Compatible);
        }
        
        Ok(results)
    }
    
    /// Check if a type can be safely promoted
    pub fn can_safely_promote_type(from_type: &DataType, to_type: &DataType) -> bool {
        match (from_type, to_type) {
            // Integer promotions
            (DataType::Int8, DataType::Int16) => true,
            (DataType::Int8, DataType::Int32) => true,
            (DataType::Int8, DataType::Int64) => true,
            (DataType::Int16, DataType::Int32) => true,
            (DataType::Int16, DataType::Int64) => true,
            (DataType::Int32, DataType::Int64) => true,
            
            // Float promotions
            (DataType::Float32, DataType::Float64) => true,
            
            // String promotions
            (DataType::String, DataType::String) => true,
            
            // Same type
            (a, b) if a == b => true,
            
            // All other cases
            _ => false,
        }
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}
