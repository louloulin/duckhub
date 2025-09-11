//! 安全功能集成测试

use duckhub_auth::{JWTManager, JWTConfig, KeyRotationManager, KeyRotationConfig, User};
use duckhub_common::prelude::*;
use std::collections::HashMap;
use chrono::Utc;

/// 创建测试用户
fn create_test_user() -> User {
    User {
        id: "test-user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@duckhub.com".to_string(),
        display_name: "Test User".to_string(),
        password_hash: String::new(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
        attributes: HashMap::new(),
    }
}

#[tokio::test]
async fn test_jwt_token_lifecycle() {
    // 创建JWT管理器
    let config = JWTConfig::default();
    let jwt_manager = JWTManager::new(config).unwrap();
    let user = create_test_user();

    // 1. 生成访问令牌和刷新令牌
    let access_token = jwt_manager.generate_token(&user).unwrap();
    let refresh_token = jwt_manager.generate_refresh_token(&user).unwrap();

    println!("✅ 生成令牌成功");
    println!("   访问令牌长度: {}", access_token.len());
    println!("   刷新令牌长度: {}", refresh_token.len());

    // 2. 验证访问令牌
    let claims = jwt_manager.verify_token(&access_token).unwrap();
    assert_eq!(claims.sub, user.id);
    assert_eq!(claims.username, user.username);
    assert_eq!(claims.token_type, "access");

    println!("✅ 访问令牌验证成功");

    // 3. 验证刷新令牌
    let refresh_claims = jwt_manager.verify_token(&refresh_token).unwrap();
    assert_eq!(refresh_claims.sub, user.id);
    assert_eq!(refresh_claims.token_type, "refresh");

    println!("✅ 刷新令牌验证成功");

    // 4. 使用刷新令牌生成新的令牌对
    let (new_access_token, new_refresh_token) = jwt_manager
        .refresh_access_token(&refresh_token)
        .unwrap();

    assert_ne!(new_access_token, access_token);
    assert_ne!(new_refresh_token, refresh_token);

    println!("✅ 令牌刷新成功");

    // 5. 验证新令牌
    let new_claims = jwt_manager.verify_token(&new_access_token).unwrap();
    assert_eq!(new_claims.sub, user.id);

    println!("✅ 新令牌验证成功");

    // 6. 检查令牌过期状态
    let is_expiring = jwt_manager.is_token_expiring_soon(&new_access_token).unwrap();
    assert!(!is_expiring); // 新生成的令牌不应该即将过期

    println!("✅ 令牌过期检查成功");

    // 7. 撤销令牌
    jwt_manager.revoke_token(&new_refresh_token).unwrap();

    println!("✅ 令牌撤销成功");
}

#[tokio::test]
async fn test_api_key_management() {
    // 创建密钥轮换管理器
    let config = KeyRotationConfig::default();
    let key_manager = KeyRotationManager::new(config);
    let user_id = "test-user-123";

    // 1. 生成API密钥
    let api_key = key_manager
        .generate_api_key(
            user_id,
            "测试密钥",
            vec!["read".to_string(), "write".to_string()],
            Some(30), // 30天过期
        )
        .await
        .unwrap();

    println!("✅ API密钥生成成功");
    println!("   密钥ID: {}", api_key.id);
    println!("   密钥名称: {}", api_key.name);
    println!("   权限范围: {:?}", api_key.scopes);
    println!("   密钥前缀: {}", &api_key.key[..3]);

    assert!(api_key.key.starts_with("dk_"));
    assert_eq!(api_key.scopes.len(), 2);
    assert!(api_key.is_active);

    // 2. 验证API密钥
    let validated_key = key_manager.validate_api_key(&api_key.key).await.unwrap();
    assert_eq!(validated_key.id, api_key.id);
    assert_eq!(validated_key.usage_count, 1); // 使用次数应该增加

    println!("✅ API密钥验证成功");

    // 3. 获取用户的所有密钥
    let user_keys = key_manager.get_user_keys(user_id).await.unwrap();
    assert_eq!(user_keys.len(), 1);
    assert_eq!(user_keys[0].id, api_key.id);

    println!("✅ 用户密钥列表获取成功");

    // 4. 轮换API密钥
    let new_key = key_manager.rotate_api_key(&api_key.id).await.unwrap();
    assert_ne!(new_key.id, api_key.id);
    assert_ne!(new_key.key, api_key.key);
    assert!(new_key.name.contains("轮换"));

    println!("✅ API密钥轮换成功");
    println!("   新密钥ID: {}", new_key.id);

    // 5. 验证旧密钥已被撤销
    let old_key_result = key_manager.validate_api_key(&api_key.key).await;
    assert!(old_key_result.is_err());

    println!("✅ 旧密钥撤销验证成功");

    // 6. 撤销新密钥
    key_manager.revoke_api_key(&new_key.id).await.unwrap();

    println!("✅ 新密钥撤销成功");

    // 7. 验证撤销后的密钥无法使用
    let revoked_key_result = key_manager.validate_api_key(&new_key.key).await;
    assert!(revoked_key_result.is_err());

    println!("✅ 撤销密钥验证成功");
}

#[tokio::test]
async fn test_security_edge_cases() {
    let jwt_config = JWTConfig::default();
    let jwt_manager = JWTManager::new(jwt_config).unwrap();
    
    let key_config = KeyRotationConfig::default();
    let key_manager = KeyRotationManager::new(key_config);

    // 1. 测试无效JWT令牌
    let invalid_token_result = jwt_manager.verify_token("invalid.token.here");
    assert!(invalid_token_result.is_err());

    println!("✅ 无效JWT令牌处理正确");

    // 2. 测试无效API密钥
    let invalid_key_result = key_manager.validate_api_key("invalid_key").await;
    assert!(invalid_key_result.is_err());

    println!("✅ 无效API密钥处理正确");

    // 3. 测试不存在的密钥轮换
    let nonexistent_rotation = key_manager.rotate_api_key("nonexistent-key-id").await;
    assert!(nonexistent_rotation.is_err());

    println!("✅ 不存在密钥轮换处理正确");

    // 4. 测试用户密钥数量限制
    let user_id = "limited-user";
    let max_keys = 10; // 默认配置的最大密钥数量

    // 生成最大数量的密钥
    for i in 0..max_keys {
        let _key = key_manager
            .generate_api_key(
                user_id,
                &format!("密钥-{}", i),
                vec!["read".to_string()],
                None,
            )
            .await
            .unwrap();
    }

    // 尝试生成超过限制的密钥
    let over_limit_result = key_manager
        .generate_api_key(
            user_id,
            "超限密钥",
            vec!["read".to_string()],
            None,
        )
        .await;
    
    assert!(over_limit_result.is_err());

    println!("✅ 密钥数量限制处理正确");

    // 5. 测试即将过期的密钥检查
    let expiring_keys = key_manager.get_expiring_keys().await.unwrap();
    // 由于我们生成的密钥都有较长的过期时间，这里应该为空
    assert!(expiring_keys.is_empty());

    println!("✅ 即将过期密钥检查正确");
}

#[tokio::test]
async fn test_security_performance() {
    let jwt_config = JWTConfig::default();
    let jwt_manager = JWTManager::new(jwt_config).unwrap();
    let user = create_test_user();

    let start_time = std::time::Instant::now();

    // 批量生成和验证令牌
    let mut tokens = Vec::new();
    for _ in 0..100 {
        let token = jwt_manager.generate_token(&user).unwrap();
        tokens.push(token);
    }

    let generation_time = start_time.elapsed();
    println!("✅ 100个令牌生成耗时: {:?}", generation_time);

    let start_time = std::time::Instant::now();

    // 批量验证令牌
    for token in &tokens {
        let _claims = jwt_manager.verify_token(token).unwrap();
    }

    let verification_time = start_time.elapsed();
    println!("✅ 100个令牌验证耗时: {:?}", verification_time);

    // 性能断言
    assert!(generation_time.as_millis() < 1000); // 生成应该在1秒内完成
    assert!(verification_time.as_millis() < 500); // 验证应该在0.5秒内完成

    println!("✅ 性能测试通过");
}
