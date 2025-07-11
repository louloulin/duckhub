//! 权限管理功能

use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 权限定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

/// 用户角色关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Uuid,
}

/// 权限检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_permissions: Vec<String>,
    pub user_permissions: Vec<String>,
}
