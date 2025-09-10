import type { Metadata } from 'next'
import { Hero } from '@/components/marketing/feature-hero'
import { FeatureDetails } from '@/components/marketing/feature-details'
import { CodeExample } from '@/components/marketing/code-example'
import { CTA } from '@/components/marketing/cta'
import { Shield, Lock, Eye, Users, FileText, Key, CheckCircle } from 'lucide-react'

export const metadata: Metadata = {
  title: '企业级安全 - RBAC权限控制和数据加密',
  description: 'DuckHub提供企业级安全保障，包括RBAC权限控制、数据加密、审计追踪、合规支持，满足金融行业安全要求。',
  keywords: ['企业级安全', 'RBAC权限', '数据加密', '审计追踪', '合规', '访问控制'],
}

const features = [
  {
    icon: Users,
    title: 'RBAC权限控制',
    description: '基于角色的访问控制，精细化权限管理，确保数据访问安全',
    details: [
      '角色定义 - 灵活的角色和权限体系',
      '细粒度控制 - 表级、列级、行级权限控制',
      '动态授权 - 基于条件的动态权限分配',
      '权限继承 - 支持权限的层级继承关系'
    ]
  },
  {
    icon: Lock,
    title: '数据加密',
    description: '全方位的数据加密保护，包括传输加密和存储加密',
    details: [
      '传输加密 - TLS 1.3加密传输通道',
      '存储加密 - AES-256数据存储加密',
      '密钥管理 - 企业级密钥管理系统',
      '字段加密 - 敏感字段的透明加密'
    ]
  },
  {
    icon: Eye,
    title: '审计追踪',
    description: '完整的操作审计日志，满足合规要求和安全监控需求',
    details: [
      '操作记录 - 详细记录所有数据操作',
      '访问日志 - 完整的用户访问轨迹',
      '变更追踪 - 数据变更的完整历史',
      '实时监控 - 异常操作的实时告警'
    ]
  },
  {
    icon: FileText,
    title: '合规支持',
    description: '满足金融行业的各项合规要求，支持多种合规标准',
    details: [
      'SOX合规 - 萨班斯法案合规支持',
      'GDPR合规 - 欧盟数据保护法规',
      '等保认证 - 国家信息安全等级保护',
      'ISO 27001 - 信息安全管理体系认证'
    ]
  }
]

const codeExamples = [
  {
    title: 'RBAC权限配置',
    language: 'sql',
    code: `-- 创建角色
CREATE ROLE analyst;
CREATE ROLE manager;
CREATE ROLE admin;

-- 授予表级权限
GRANT SELECT ON orders TO analyst;
GRANT SELECT, INSERT, UPDATE ON orders TO manager;
GRANT ALL PRIVILEGES ON orders TO admin;

-- 列级权限控制
GRANT SELECT (order_id, customer_id, amount) ON orders TO analyst;
REVOKE SELECT (customer_phone, customer_email) ON orders FROM analyst;

-- 行级安全策略
CREATE POLICY analyst_policy ON orders
    FOR SELECT TO analyst
    USING (region = current_user_region());

-- 用户角色分配
GRANT analyst TO user_john;
GRANT manager TO user_mary;`
  },
  {
    title: '数据加密配置',
    language: 'sql',
    code: `-- 启用表级加密
CREATE TABLE customers (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) ENCRYPTED,  -- 字段级加密
    phone VARCHAR(20) ENCRYPTED,
    address TEXT ENCRYPTED,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) WITH (encryption = 'AES256');

-- 配置加密密钥
SET encryption_key_id = 'customer-data-key-2024';

-- 透明加密查询（自动解密）
SELECT id, name, email FROM customers 
WHERE id = 12345;

-- 加密传输配置
SET ssl_mode = 'require';
SET ssl_cert = '/path/to/client.crt';
SET ssl_key = '/path/to/client.key';`
  },
  {
    title: '审计日志查询',
    language: 'sql',
    code: `-- 查看用户操作审计
SELECT 
    timestamp,
    user_name,
    operation_type,
    table_name,
    affected_rows,
    client_ip,
    session_id
FROM audit_log 
WHERE timestamp >= CURRENT_DATE - INTERVAL '7 days'
  AND operation_type IN ('INSERT', 'UPDATE', 'DELETE')
ORDER BY timestamp DESC;

-- 敏感数据访问审计
SELECT 
    user_name,
    COUNT(*) as access_count,
    MIN(timestamp) as first_access,
    MAX(timestamp) as last_access
FROM audit_log 
WHERE table_name = 'customers'
  AND column_name IN ('email', 'phone', 'address')
  AND timestamp >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY user_name
ORDER BY access_count DESC;

-- 异常操作检测
SELECT * FROM security_alerts 
WHERE alert_type = 'SUSPICIOUS_ACCESS'
  AND status = 'ACTIVE'
ORDER BY created_at DESC;`
  }
]

const securityCertifications = [
  {
    name: 'SOC 2 Type II',
    description: '服务组织控制报告',
    icon: Shield
  },
  {
    name: 'ISO 27001',
    description: '信息安全管理体系',
    icon: Lock
  },
  {
    name: '等保三级',
    description: '国家信息安全等级保护',
    icon: FileText
  },
  {
    name: 'GDPR',
    description: '欧盟数据保护法规',
    icon: Eye
  }
]

const benefits = [
  {
    icon: Shield,
    title: '银行级安全',
    description: '采用金融行业标准的安全措施，保障数据绝对安全'
  },
  {
    icon: CheckCircle,
    title: '合规保证',
    description: '满足各项合规要求，通过多项国际安全认证'
  },
  {
    icon: Key,
    title: '访问控制',
    description: '精细化权限管理，确保数据访问的可控性和可追溯性'
  }
]

/**
 * 企业级安全功能页面
 * 展示RBAC权限控制、数据加密、审计追踪等安全特性
 */
export default function EnterprisePage() {
  return (
    <>
      <Hero
        title="企业级安全"
        subtitle="RBAC权限控制和数据加密"
        description="为金融机构量身打造的企业级安全解决方案，提供RBAC权限控制、数据加密、审计追踪、合规支持等全方位安全保障。"
        gradient="from-green-500 to-emerald-500"
        icon={Shield}
        benefits={benefits}
      />
      
      {/* 安全认证展示 */}
      <section className="py-24 bg-gradient-to-br from-green-50 to-emerald-50">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-gray-900 mb-4">
              安全认证
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              通过多项国际安全认证，确保企业级安全标准
            </p>
          </div>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            {securityCertifications.map((cert, index) => {
              const IconComponent = cert.icon
              return (
                <div key={index} className="text-center">
                  <div className="bg-white rounded-2xl p-8 shadow-lg hover:shadow-xl transition-shadow">
                    <div className="w-16 h-16 bg-gradient-to-br from-green-500 to-emerald-500 rounded-xl flex items-center justify-center mx-auto mb-4">
                      <IconComponent className="w-8 h-8 text-white" />
                    </div>
                    <div className="text-lg font-semibold text-gray-900 mb-2">
                      {cert.name}
                    </div>
                    <div className="text-sm text-gray-600">
                      {cert.description}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </section>
      
      <FeatureDetails
        title="安全核心能力"
        subtitle="全方位的企业级安全保障体系"
        features={features}
      />
      
      <CodeExample
        title="安全配置示例"
        subtitle="了解如何配置和使用DuckHub的安全功能"
        examples={codeExamples}
      />
      
      <CTA
        title="保障数据安全"
        description="立即启用企业级安全保护"
        primaryText="安全评估"
        secondaryText="安全文档"
        primaryHref="/contact"
        secondaryHref="/docs/security"
      />
    </>
  )
}