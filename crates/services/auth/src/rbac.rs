//! RBAC权限管理模块
//! 
//! 实现基于角色的访问控制(Role-Based Access Control)：
//! - 用户(User)
//! - 角色(Role) 
//! - 权限(Permission)
//! - 资源(Resource)

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, debug, instrument};

/// RBAC权限管理器
pub struct RBACManager {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 用户缓存
    users_cache: HashMap<String, User>,
    /// 角色缓存
    roles_cache: HashMap<String, Role>,
    /// 权限缓存
    permissions_cache: HashMap<String, Permission>,
    /// 用户角色映射缓存
    user_roles_cache: HashMap<String, HashSet<String>>,
    /// 角色权限映射缓存
    role_permissions_cache: HashMap<String, HashSet<String>>,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最后登录时间
    pub last_login_at: Option<DateTime<Utc>>,
    /// 用户属性
    pub attributes: HashMap<String, String>,
}

/// 角色信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色ID
    pub id: String,
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// 权限ID
    pub id: String,
    /// 权限名称
    pub name: String,
    /// 资源类型
    pub resource: String,
    /// 操作类型
    pub action: String,
    /// 权限描述
    pub description: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 资源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// 资源ID
    pub id: String,
    /// 资源名称
    pub name: String,
    /// 资源类型
    pub resource_type: String,
    /// 资源描述
    pub description: String,
    /// 父资源ID
    pub parent_id: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl RBACManager {
    /// 创建新的RBAC管理器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>) -> Result<Self> {
        let mut manager = Self {
            engine,
            users_cache: HashMap::new(),
            roles_cache: HashMap::new(),
            permissions_cache: HashMap::new(),
            user_roles_cache: HashMap::new(),
            role_permissions_cache: HashMap::new(),
        };

        // 初始化数据库表
        manager.initialize_tables().await?;
        
        // 加载缓存
        manager.load_cache().await?;

        info!("RBAC权限管理器初始化完成");
        Ok(manager)
    }

    /// 初始化数据库表
    async fn initialize_tables(&self) -> Result<()> {
        // 创建用户表
        let create_users_sql = r#"
            CREATE TABLE IF NOT EXISTS rbac_users (
                id VARCHAR PRIMARY KEY,
                username VARCHAR UNIQUE NOT NULL,
                email VARCHAR UNIQUE NOT NULL,
                display_name VARCHAR NOT NULL,
                enabled BOOLEAN DEFAULT true,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_login_at TIMESTAMP,
                attributes JSON
            )
        "#;
        self.engine.execute(create_users_sql).await?;

        // 创建角色表
        let create_roles_sql = r#"
            CREATE TABLE IF NOT EXISTS rbac_roles (
                id VARCHAR PRIMARY KEY,
                name VARCHAR UNIQUE NOT NULL,
                description TEXT,
                enabled BOOLEAN DEFAULT true,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        self.engine.execute(create_roles_sql).await?;

        // 创建权限表
        let create_permissions_sql = r#"
            CREATE TABLE IF NOT EXISTS rbac_permissions (
                id VARCHAR PRIMARY KEY,
                name VARCHAR UNIQUE NOT NULL,
                resource VARCHAR NOT NULL,
                action VARCHAR NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        self.engine.execute(create_permissions_sql).await?;

        // 创建用户角色关联表
        let create_user_roles_sql = r#"
            CREATE TABLE IF NOT EXISTS rbac_user_roles (
                user_id VARCHAR NOT NULL,
                role_id VARCHAR NOT NULL,
                assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                assigned_by VARCHAR,
                PRIMARY KEY (user_id, role_id)
            )
        "#;
        self.engine.execute(create_user_roles_sql).await?;

        // 创建角色权限关联表
        let create_role_permissions_sql = r#"
            CREATE TABLE IF NOT EXISTS rbac_role_permissions (
                role_id VARCHAR NOT NULL,
                permission_id VARCHAR NOT NULL,
                assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                assigned_by VARCHAR,
                PRIMARY KEY (role_id, permission_id)
            )
        "#;
        self.engine.execute(create_role_permissions_sql).await?;

        // 创建资源表
        let create_resources_sql = r#"
            CREATE TABLE IF NOT EXISTS rbac_resources (
                id VARCHAR PRIMARY KEY,
                name VARCHAR NOT NULL,
                resource_type VARCHAR NOT NULL,
                description TEXT,
                parent_id VARCHAR,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        self.engine.execute(create_resources_sql).await?;

        info!("RBAC数据库表初始化完成");
        Ok(())
    }

    /// 加载缓存
    async fn load_cache(&mut self) -> Result<()> {
        // 加载用户
        let users_result = self.engine.query("SELECT * FROM rbac_users").await?;
        for row in &users_result {
            if let Ok(user) = self.parse_user_from_row(row) {
                self.users_cache.insert(user.id.clone(), user);
            }
        }

        // 加载角色
        let roles_result = self.engine.query("SELECT * FROM rbac_roles").await?;
        for row in &roles_result {
            if let Ok(role) = self.parse_role_from_row(row) {
                self.roles_cache.insert(role.id.clone(), role);
            }
        }

        // 加载权限
        let permissions_result = self.engine.query("SELECT * FROM rbac_permissions").await?;
        for row in &permissions_result {
            if let Ok(permission) = self.parse_permission_from_row(row) {
                self.permissions_cache.insert(permission.id.clone(), permission);
            }
        }

        // 加载用户角色映射
        let user_roles_result = self.engine.query("SELECT user_id, role_id FROM rbac_user_roles").await?;
        for row in &user_roles_result {
            if let (Some(user_id), Some(role_id)) = (row.get("user_id"), row.get("role_id")) {
                if let (Some(user_id_str), Some(role_id_str)) = (user_id.as_str(), role_id.as_str()) {
                    self.user_roles_cache
                        .entry(user_id_str.to_string())
                        .or_insert_with(HashSet::new)
                        .insert(role_id_str.to_string());
                }
            }
        }

        // 加载角色权限映射
        let role_permissions_result = self.engine.query("SELECT role_id, permission_id FROM rbac_role_permissions").await?;
        for row in &role_permissions_result {
            if let (Some(role_id), Some(permission_id)) = (row.get("role_id"), row.get("permission_id")) {
                if let (Some(role_id_str), Some(permission_id_str)) = (role_id.as_str(), permission_id.as_str()) {
                    self.role_permissions_cache
                        .entry(role_id_str.to_string())
                        .or_insert_with(HashSet::new)
                        .insert(permission_id_str.to_string());
                }
            }
        }

        info!("RBAC缓存加载完成");
        Ok(())
    }

    /// 检查用户权限
    #[instrument(skip(self))]
    pub async fn check_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<bool> {
        // 获取用户的所有角色
        let empty_roles = HashSet::new();
        let user_roles = self.user_roles_cache.get(user_id).unwrap_or(&empty_roles);

        // 检查每个角色的权限
        for role_id in user_roles {
            if let Some(role_permissions) = self.role_permissions_cache.get(role_id) {
                for permission_id in role_permissions {
                    if let Some(permission) = self.permissions_cache.get(permission_id) {
                        if permission.resource == resource && permission.action == action {
                            debug!("用户 {} 通过角色 {} 拥有权限 {}:{}", user_id, role_id, resource, action);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        debug!("用户 {} 没有权限 {}:{}", user_id, resource, action);
        Ok(false)
    }

    /// 创建用户
    #[instrument(skip(self))]
    pub async fn create_user(&mut self, username: &str, email: &str, display_name: &str) -> Result<User> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            attributes: HashMap::new(),
        };

        // 插入数据库
        let insert_sql = format!(
            r#"INSERT INTO rbac_users (id, username, email, display_name, enabled, created_at, updated_at, attributes)
               VALUES ('{}', '{}', '{}', '{}', {}, '{}', '{}', '{}')"#,
            user.id,
            user.username,
            user.email,
            user.display_name,
            user.enabled,
            user.created_at.to_rfc3339(),
            user.updated_at.to_rfc3339(),
            serde_json::to_string(&user.attributes).unwrap_or_default(),
        );

        self.engine.execute(&insert_sql).await?;

        // 更新缓存
        self.users_cache.insert(user.id.clone(), user.clone());

        info!("创建用户: {} ({})", user.username, user.id);
        Ok(user)
    }

    /// 创建角色
    #[instrument(skip(self))]
    pub async fn create_role(&mut self, name: &str, description: &str) -> Result<Role> {
        let role = Role {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 插入数据库
        let insert_sql = format!(
            r#"INSERT INTO rbac_roles (id, name, description, enabled, created_at, updated_at)
               VALUES ('{}', '{}', '{}', {}, '{}', '{}')"#,
            role.id,
            role.name,
            role.description,
            role.enabled,
            role.created_at.to_rfc3339(),
            role.updated_at.to_rfc3339(),
        );

        self.engine.execute(&insert_sql).await?;

        // 更新缓存
        self.roles_cache.insert(role.id.clone(), role.clone());

        info!("创建角色: {} ({})", role.name, role.id);
        Ok(role)
    }

    /// 分配角色给用户
    #[instrument(skip(self))]
    pub async fn assign_role_to_user(&mut self, user_id: &str, role_id: &str) -> Result<()> {
        // 检查用户和角色是否存在
        if !self.users_cache.contains_key(user_id) {
            return Err(DuckHubError::validation(format!("用户不存在: {}", user_id)));
        }
        if !self.roles_cache.contains_key(role_id) {
            return Err(DuckHubError::validation(format!("角色不存在: {}", role_id)));
        }

        // 插入数据库
        let insert_sql = format!(
            r#"INSERT OR IGNORE INTO rbac_user_roles (user_id, role_id, assigned_at)
               VALUES ('{}', '{}', '{}')"#,
            user_id,
            role_id,
            Utc::now().to_rfc3339()
        );

        self.engine.execute(&insert_sql).await?;

        // 更新缓存
        self.user_roles_cache
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());

        info!("分配角色 {} 给用户 {}", role_id, user_id);
        Ok(())
    }

    /// 获取用户信息
    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users_cache.get(user_id)
    }

    /// 根据用户名获取用户
    pub fn get_user_by_username(&self, username: &str) -> Option<&User> {
        self.users_cache.values().find(|user| user.username == username)
    }

    /// 解析用户数据行
    fn parse_user_from_row(&self, row: &HashMap<String, serde_json::Value>) -> Result<User> {
        Ok(User {
            id: row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            username: row.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            email: row.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            display_name: row.get("display_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: row.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            attributes: HashMap::new(),
        })
    }

    /// 解析角色数据行
    fn parse_role_from_row(&self, row: &HashMap<String, serde_json::Value>) -> Result<Role> {
        Ok(Role {
            id: row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: row.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: row.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// 解析权限数据行
    fn parse_permission_from_row(&self, row: &HashMap<String, serde_json::Value>) -> Result<Permission> {
        Ok(Permission {
            id: row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            resource: row.get("resource").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            action: row.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: row.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            created_at: Utc::now(),
        })
    }
}
