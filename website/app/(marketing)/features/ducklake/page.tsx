import type { Metadata } from 'next'
import { Hero } from '@/components/marketing/feature-hero'
import { FeatureDetails } from '@/components/marketing/feature-details'
import { CodeExample } from '@/components/marketing/code-example'
import { CTA } from '@/components/marketing/cta'
import { Database, Clock, GitBranch, Shield, Zap, CheckCircle } from 'lucide-react'

export const metadata: Metadata = {
  title: 'DuckLake数据湖 - 企业级ACID事务和时间旅行查询',
  description: 'DuckLake提供ACID事务保证、时间旅行查询、Schema演进等企业级数据湖功能，构建可靠的数据基础设施。',
  keywords: ['DuckLake', '数据湖', 'ACID事务', '时间旅行查询', 'Schema演进', '快照管理'],
}

const features = [
  {
    icon: Shield,
    title: 'ACID事务保证',
    description: '完整的原子性、一致性、隔离性、持久性保证，确保数据完整性',
    details: [
      '原子性操作 - 要么全部成功，要么全部回滚',
      '一致性保证 - 数据始终保持一致状态',
      '隔离级别 - 支持多种隔离级别配置',
      '持久性存储 - 数据持久化到可靠存储'
    ]
  },
  {
    icon: Clock,
    title: '时间旅行查询',
    description: '基于版本号和时间戳的历史数据查询，轻松回溯任意时间点的数据状态',
    details: [
      '版本查询 - 基于版本号查询历史数据',
      '时间戳查询 - 基于时间戳回溯数据状态',
      '增量查询 - 高效获取数据变更',
      '快照对比 - 比较不同时间点的数据差异'
    ]
  },
  {
    icon: GitBranch,
    title: 'Schema演进',
    description: '灵活的Schema变更管理，支持向后兼容的数据结构演进',
    details: [
      '向后兼容 - 新Schema兼容历史数据',
      '自动迁移 - 智能数据结构迁移',
      '版本管理 - Schema版本控制和回滚',
      '类型安全 - 强类型检查和验证'
    ]
  },
  {
    icon: Database,
    title: '快照管理',
    description: '高效的数据快照创建和管理，支持快速数据恢复和分支操作',
    details: [
      '增量快照 - 只存储变更数据',
      '快速恢复 - 秒级数据恢复',
      '分支操作 - 支持数据分支和合并',
      '压缩优化 - 智能数据压缩存储'
    ]
  }
]

const codeExamples = [
  {
    title: 'ACID事务操作',
    language: 'sql',
    code: `-- 开始事务
BEGIN TRANSACTION;

-- 插入订单数据
INSERT INTO orders (id, customer_id, amount, status)
VALUES (1001, 'C001', 1500.00, 'pending');

-- 更新库存
UPDATE inventory 
SET quantity = quantity - 1 
WHERE product_id = 'P001';

-- 提交事务
COMMIT;`
  },
  {
    title: '时间旅行查询',
    language: 'sql',
    code: `-- 查询指定时间点的数据
SELECT * FROM orders 
FOR SYSTEM_TIME AS OF '2024-01-15 10:30:00';

-- 查询指定版本的数据
SELECT * FROM orders 
FOR VERSION AS OF 12345;

-- 查询时间范围内的变更
SELECT * FROM orders 
FOR SYSTEM_TIME BETWEEN 
  '2024-01-15 09:00:00' AND '2024-01-15 18:00:00';`
  },
  {
    title: 'Schema演进',
    language: 'sql',
    code: `-- 添加新列（向后兼容）
ALTER TABLE customers 
ADD COLUMN email VARCHAR(255) DEFAULT NULL;

-- 创建新版本Schema
CREATE TABLE customers_v2 (
  id BIGINT PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(255),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 数据迁移
INSERT INTO customers_v2 
SELECT id, name, NULL as email, created_at, updated_at 
FROM customers;`
  }
]

const benefits = [
  {
    icon: Zap,
    title: '高性能',
    description: '基于DuckDB的向量化执行引擎，查询性能提升10-100倍'
  },
  {
    icon: Shield,
    title: '数据安全',
    description: '企业级安全保障，支持数据加密和访问控制'
  },
  {
    icon: CheckCircle,
    title: '易于使用',
    description: '标准SQL接口，无需学习新的查询语言'
  }
]

/**
 * DuckLake数据湖功能页面
 * 展示ACID事务、时间旅行查询、Schema演进等核心功能
 */
export default function DuckLakePage() {
  return (
    <>
      <Hero
        title="DuckLake数据湖"
        subtitle="企业级ACID事务和时间旅行查询"
        description="基于DuckDB构建的现代化数据湖，提供ACID事务保证、时间旅行查询、Schema演进等企业级功能，为金融机构构建可靠的数据基础设施。"
        gradient="from-blue-500 to-cyan-500"
        icon={Database}
        benefits={benefits}
      />
      
      <FeatureDetails
        title="核心功能特性"
        subtitle="为企业级数据管理量身打造的强大功能"
        features={features}
      />
      
      <CodeExample
        title="代码示例"
        subtitle="通过实际代码了解DuckLake的强大功能"
        examples={codeExamples}
      />
      
      <CTA
        title="开始使用DuckLake"
        description="立即体验企业级数据湖的强大功能"
        primaryText="免费试用"
        secondaryText="查看文档"
        primaryHref="/contact"
        secondaryHref="/docs/ducklake"
      />
    </>
  )
}