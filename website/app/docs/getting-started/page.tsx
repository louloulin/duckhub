import type { Metadata } from 'next'
import { InstallationGuide } from '@/components/docs/installation-guide'
import { ConfigurationGuide } from '@/components/docs/configuration-guide'
import { FirstQueryGuide } from '@/components/docs/first-query-guide'
import { NextSteps } from '@/components/docs/next-steps'

export const metadata: Metadata = {
  title: '快速开始 - DuckHub文档',
  description: '5分钟快速上手DuckHub，从安装部署到第一个查询的完整指南。',
  keywords: ['快速开始', '安装', '配置', '教程', '入门'],
}

const installationMethods = [
  {
    title: 'Docker 部署',
    description: '推荐方式，快速启动完整环境',
    icon: '🐳',
    code: `# 拉取镜像
docker pull duckhub/duckhub:latest

# 启动服务
docker run -d \
  --name duckhub \
  -p 8080:8080 \
  -p 5432:5432 \
  -v duckhub-data:/var/lib/duckhub \
  duckhub/duckhub:latest

# 检查服务状态
docker logs duckhub`,
    pros: ['一键部署', '环境隔离', '易于管理'],
    requirements: 'Docker 20.10+'
  },
  {
    title: 'Kubernetes 部署',
    description: '生产环境推荐，支持高可用',
    icon: '☸️',
    code: `# 添加 Helm 仓库
helm repo add duckhub https://charts.duckhub.com
helm repo update

# 安装 DuckHub
helm install duckhub duckhub/duckhub \
  --namespace duckhub \
  --create-namespace \
  --set persistence.enabled=true \
  --set ingress.enabled=true

# 检查部署状态
kubectl get pods -n duckhub`,
    pros: ['高可用', '自动扩缩容', '生产就绪'],
    requirements: 'Kubernetes 1.20+, Helm 3.0+'
  },
  {
    title: '二进制部署',
    description: '直接运行，适合开发测试',
    icon: '📦',
    code: `# 下载二进制文件
wget https://github.com/duckhub/duckhub/releases/latest/download/duckhub-linux-amd64.tar.gz

# 解压并安装
tar -xzf duckhub-linux-amd64.tar.gz
sudo mv duckhub /usr/local/bin/

# 启动服务
duckhub server --config config.yaml

# 后台运行
nohup duckhub server --config config.yaml > duckhub.log 2>&1 &`,
    pros: ['轻量级', '直接控制', '快速测试'],
    requirements: 'Linux/macOS, 4GB+ RAM'
  }
]

const configurationSteps = [
  {
    step: 1,
    title: '基础配置',
    description: '配置数据库连接和基本参数',
    code: `# config.yaml
server:
  host: 0.0.0.0
  port: 8080
  
database:
  host: localhost
  port: 5432
  name: duckhub
  user: duckhub
  password: your_password
  
storage:
  type: s3
  bucket: duckhub-data
  region: us-east-1
  
auth:
  enabled: true
  jwt_secret: your_jwt_secret
  
logging:
  level: info
  format: json`
  },
  {
    step: 2,
    title: '数据源配置',
    description: '连接您的数据源',
    code: `# 添加 PostgreSQL 数据源
curl -X POST http://localhost:8080/api/v1/datasources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "postgres-prod",
    "type": "postgresql",
    "config": {
      "host": "prod-db.example.com",
      "port": 5432,
      "database": "production",
      "username": "readonly",
      "password": "password"
    }
  }'

# 添加 S3 数据源
curl -X POST http://localhost:8080/api/v1/datasources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "s3-datalake",
    "type": "s3",
    "config": {
      "bucket": "my-datalake",
      "region": "us-west-2",
      "access_key": "AKIAIOSFODNN7EXAMPLE",
      "secret_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
    }
  }'`
  },
  {
    step: 3,
    title: '用户权限配置',
    description: '设置用户角色和权限',
    code: `# 创建管理员用户
curl -X POST http://localhost:8080/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "email": "admin@example.com",
    "password": "secure_password",
    "role": "admin"
  }'

# 创建分析师角色
curl -X POST http://localhost:8080/api/v1/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "analyst",
    "permissions": [
      "query:read",
      "dashboard:read",
      "datasource:read"
    ]
  }'

# 分配用户角色
curl -X POST http://localhost:8080/api/v1/users/john/roles \
  -H "Content-Type: application/json" \
  -d '{"role": "analyst"}'`
  }
]

const firstQueryExamples = [
  {
    title: 'SQL 查询',
    description: '使用标准 SQL 查询数据',
    code: `-- 查询订单统计
SELECT 
    DATE_TRUNC('day', order_date) as day,
    COUNT(*) as order_count,
    SUM(amount) as total_revenue,
    AVG(amount) as avg_order_value
FROM orders 
WHERE order_date >= '2024-01-01'
GROUP BY DATE_TRUNC('day', order_date)
ORDER BY day DESC
LIMIT 30;`,
    language: 'sql'
  },
  {
    title: 'AI 自然语言查询',
    description: '使用自然语言描述查询需求',
    code: `用户输入："查询最近30天每天的订单数量和总收入"

AI 自动生成：
SELECT 
    DATE_TRUNC('day', order_date) as day,
    COUNT(*) as order_count,
    SUM(amount) as total_revenue
FROM orders 
WHERE order_date >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE_TRUNC('day', order_date)
ORDER BY day DESC;`,
    language: 'text'
  },
  {
    title: 'REST API 调用',
    description: '通过 API 执行查询',
    code: `# 执行 SQL 查询
curl -X POST http://localhost:8080/api/v1/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your_token" \
  -d '{
    "sql": "SELECT COUNT(*) FROM orders WHERE order_date >= CURRENT_DATE",
    "datasource": "postgres-prod"
  }'

# 响应结果
{
  "query_id": "q_123456",
  "status": "completed",
  "rows": [
    {"count": 1250}
  ],
  "execution_time": "0.045s",
  "rows_affected": 1
}`,
    language: 'bash'
  }
]

const nextSteps = [
  {
    title: '探索 AI 助手',
    description: '学习如何使用自然语言查询数据',
    href: '/docs/guides/ai-assistant',
    icon: '🤖'
  },
  {
    title: '性能优化',
    description: '了解查询优化和性能调优技巧',
    href: '/docs/guides/performance',
    icon: '⚡'
  },
  {
    title: '安全配置',
    description: '配置企业级安全和权限控制',
    href: '/docs/guides/security',
    icon: '🔒'
  },
  {
    title: 'API 参考',
    description: '查看完整的 API 文档和示例',
    href: '/docs/api-reference',
    icon: '📚'
  }
]

/**
 * 快速开始页面
 * 提供详细的安装、配置和使用指南
 */
export default function GettingStartedPage() {
  return (
    <div className="prose prose-lg max-w-none">
      <div className="not-prose mb-12">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">
          快速开始
        </h1>
        <p className="text-xl text-gray-600">
          5分钟快速上手DuckHub，从安装部署到第一个查询的完整指南
        </p>
      </div>

      <InstallationGuide methods={installationMethods} />
      
      <ConfigurationGuide steps={configurationSteps} />
      
      <FirstQueryGuide examples={firstQueryExamples} />
      
      <NextSteps steps={nextSteps} />
    </div>
  )
}