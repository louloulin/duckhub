import type { Metadata } from 'next'
import { Hero } from '@/components/marketing/feature-hero'
import { FeatureDetails } from '@/components/marketing/feature-details'
import { CodeExample } from '@/components/marketing/code-example'
import { CTA } from '@/components/marketing/cta'
import { Zap, Cpu, Database, BarChart3, Gauge, Layers, CheckCircle } from 'lucide-react'

export const metadata: Metadata = {
  title: '高性能引擎 - 向量化执行和并行处理',
  description: 'DuckHub基于DuckDB的高性能查询引擎，提供向量化执行、并行处理、智能缓存，查询性能提升10-100倍。',
  keywords: ['高性能', '向量化执行', '并行处理', '查询缓存', 'DuckDB', '性能优化'],
}

const features = [
  {
    icon: Cpu,
    title: '向量化执行',
    description: '基于SIMD指令的向量化计算，充分利用现代CPU的并行计算能力',
    details: [
      'SIMD优化 - 利用CPU的SIMD指令集加速计算',
      '批量处理 - 一次处理多个数据元素',
      '内存效率 - 优化内存访问模式，减少缓存未命中',
      '类型特化 - 针对不同数据类型的专门优化'
    ]
  },
  {
    icon: Layers,
    title: '并行处理',
    description: '智能的并行查询执行，充分利用多核CPU和分布式计算资源',
    details: [
      '多线程执行 - 自动并行化查询操作',
      '任务调度 - 智能的工作负载分配',
      '资源管理 - 动态调整并行度和资源使用',
      '负载均衡 - 确保各个处理单元负载均衡'
    ]
  },
  {
    icon: Database,
    title: '智能缓存',
    description: '多层次的智能缓存系统，显著提升重复查询的响应速度',
    details: [
      '查询结果缓存 - 缓存频繁查询的结果',
      '元数据缓存 - 缓存表结构和统计信息',
      '执行计划缓存 - 复用优化的执行计划',
      '自适应策略 - 根据访问模式动态调整缓存策略'
    ]
  },
  {
    icon: BarChart3,
    title: '查询优化',
    description: '基于成本的查询优化器，自动选择最优的执行策略',
    details: [
      '成本估算 - 精确的查询成本估算模型',
      '执行计划优化 - 智能选择最优执行路径',
      '统计信息 - 基于实时统计信息的优化决策',
      '自适应优化 - 根据执行反馈持续优化'
    ]
  }
]

const codeExamples = [
  {
    title: '向量化聚合查询',
    language: 'sql',
    code: `-- 大规模数据聚合查询
SELECT 
    product_category,
    COUNT(*) as order_count,
    SUM(amount) as total_revenue,
    AVG(amount) as avg_order_value,
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY amount) as median_amount
FROM orders 
WHERE order_date >= '2024-01-01'
GROUP BY product_category
ORDER BY total_revenue DESC;

-- 性能提升：
-- 传统数据库：45秒
-- DuckHub：0.8秒 (56x 提升)`
  },
  {
    title: '并行窗口函数',
    language: 'sql',
    code: `-- 复杂的窗口函数查询
SELECT 
    customer_id,
    order_date,
    amount,
    -- 移动平均
    AVG(amount) OVER (
        PARTITION BY customer_id 
        ORDER BY order_date 
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) as moving_avg_7d,
    -- 累计求和
    SUM(amount) OVER (
        PARTITION BY customer_id 
        ORDER BY order_date
    ) as cumulative_total,
    -- 排名
    ROW_NUMBER() OVER (
        PARTITION BY customer_id 
        ORDER BY amount DESC
    ) as amount_rank
FROM orders
WHERE customer_id IN (SELECT customer_id FROM top_customers);

-- 并行执行，充分利用多核CPU`
  },
  {
    title: '智能缓存优化',
    language: 'sql',
    code: `-- 启用查询缓存
SET enable_query_cache = true;
SET cache_size = '2GB';

-- 频繁查询会被自动缓存
SELECT 
    DATE_TRUNC('day', order_date) as day,
    COUNT(*) as daily_orders,
    SUM(amount) as daily_revenue
FROM orders 
WHERE order_date >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE_TRUNC('day', order_date)
ORDER BY day;

-- 首次执行：1.2秒
-- 缓存命中：0.05秒 (24x 提升)

-- 查看缓存状态
SELECT * FROM system.cache_stats;`
  }
]

const performanceStats = [
  {
    metric: '查询性能',
    improvement: '10-100x',
    description: '相比传统数据库的性能提升'
  },
  {
    metric: '并发处理',
    improvement: '1000+',
    description: '支持的并发查询数量'
  },
  {
    metric: '响应时间',
    improvement: '<100ms',
    description: '简单查询的平均响应时间'
  },
  {
    metric: '吞吐量',
    improvement: '10M+',
    description: '每秒处理的记录数'
  }
]

const benefits = [
  {
    icon: Zap,
    title: '极致性能',
    description: '基于DuckDB的向量化引擎，查询性能提升10-100倍'
  },
  {
    icon: Gauge,
    title: '低延迟',
    description: '毫秒级查询响应，支持实时数据分析和决策'
  },
  {
    icon: CheckCircle,
    title: '高并发',
    description: '支持1000+并发查询，满足企业级应用需求'
  }
]

/**
 * 高性能引擎功能页面
 * 展示向量化执行、并行处理、查询缓存等性能特性
 */
export default function PerformancePage() {
  return (
    <>
      <Hero
        title="高性能引擎"
        subtitle="向量化执行和并行处理"
        description="基于DuckDB构建的高性能查询引擎，采用向量化执行、并行处理、智能缓存等先进技术，为企业提供极致的查询性能体验。"
        gradient="from-orange-500 to-red-500"
        icon={Zap}
        benefits={benefits}
      />
      
      {/* 性能指标展示 */}
      <section className="py-24 bg-gradient-to-br from-orange-50 to-red-50">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-gray-900 mb-4">
              性能指标
            </h2>
            <p className="text-xl text-gray-600 max-w-3xl mx-auto">
              真实的性能数据，证明DuckHub的卓越表现
            </p>
          </div>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            {performanceStats.map((stat, index) => (
              <div key={index} className="text-center">
                <div className="bg-white rounded-2xl p-8 shadow-lg hover:shadow-xl transition-shadow">
                  <div className="text-3xl font-bold text-orange-600 mb-2">
                    {stat.improvement}
                  </div>
                  <div className="text-lg font-semibold text-gray-900 mb-2">
                    {stat.metric}
                  </div>
                  <div className="text-sm text-gray-600">
                    {stat.description}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>
      
      <FeatureDetails
        title="核心技术特性"
        subtitle="领先的性能优化技术，释放数据的真正潜力"
        features={features}
      />
      
      <CodeExample
        title="性能优化示例"
        subtitle="通过实际案例了解DuckHub的性能优势"
        examples={codeExamples}
      />
      
      <CTA
        title="体验极致性能"
        description="立即测试DuckHub的高性能查询引擎"
        primaryText="性能测试"
        secondaryText="技术文档"
        primaryHref="/contact"
        secondaryHref="/docs/performance"
      />
    </>
  )
}