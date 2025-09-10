import type { Metadata } from 'next'
import { Hero } from '@/components/marketing/feature-hero'
import { FeatureDetails } from '@/components/marketing/feature-details'
import { CodeExample } from '@/components/marketing/code-example'
import { CTA } from '@/components/marketing/cta'
import { Bot, MessageSquare, Lightbulb, TrendingUp, Zap, Brain, CheckCircle } from 'lucide-react'

export const metadata: Metadata = {
  title: 'AI智能助手 - 自然语言转SQL和智能查询优化',
  description: 'DuckHub AI助手支持自然语言转SQL、智能查询优化、对话式数据分析，让数据分析变得更简单高效。',
  keywords: ['AI助手', '自然语言转SQL', '智能查询优化', '对话式分析', '机器学习', '数据分析'],
}

const features = [
  {
    icon: MessageSquare,
    title: '自然语言查询',
    description: '使用自然语言描述查询需求，AI自动转换为高效的SQL语句',
    details: [
      '中英文支持 - 支持中文和英文自然语言输入',
      '语义理解 - 深度理解业务语义和查询意图',
      '上下文感知 - 基于对话历史提供智能建议',
      '多表关联 - 自动识别表间关系和连接条件'
    ]
  },
  {
    icon: Lightbulb,
    title: '智能查询优化',
    description: '基于机器学习的查询优化建议，自动提升查询性能',
    details: [
      '执行计划优化 - 智能选择最优执行路径',
      '索引建议 - 基于查询模式推荐索引策略',
      '性能预测 - 预估查询执行时间和资源消耗',
      '自动调优 - 根据历史数据自动优化参数'
    ]
  },
  {
    icon: TrendingUp,
    title: '对话式分析',
    description: '通过对话方式进行数据探索和分析，降低数据分析门槛',
    details: [
      '交互式探索 - 通过对话逐步深入数据分析',
      '可视化建议 - 智能推荐最适合的图表类型',
      '趋势发现 - 自动识别数据中的异常和趋势',
      '报告生成 - 一键生成专业的分析报告'
    ]
  },
  {
    icon: Brain,
    title: '异常检测',
    description: '基于机器学习的智能异常检测，及时发现数据质量问题',
    details: [
      '实时监控 - 实时检测数据异常和质量问题',
      '模式学习 - 学习正常数据模式，识别异常',
      '智能告警 - 根据业务重要性智能分级告警',
      '根因分析 - 自动分析异常产生的可能原因'
    ]
  }
]

const codeExamples = [
  {
    title: '自然语言查询',
    language: 'text',
    code: `用户输入："查询上个月销售额超过10万的客户信息"

AI理解：
- 时间范围：上个月
- 条件：销售额 > 100,000
- 目标：客户信息

生成SQL：
SELECT c.customer_id, c.name, c.email, SUM(o.amount) as total_sales
FROM customers c
JOIN orders o ON c.customer_id = o.customer_id
WHERE o.order_date >= DATE_TRUNC('month', CURRENT_DATE - INTERVAL '1 month')
  AND o.order_date < DATE_TRUNC('month', CURRENT_DATE)
GROUP BY c.customer_id, c.name, c.email
HAVING SUM(o.amount) > 100000
ORDER BY total_sales DESC;`
  },
  {
    title: '智能查询优化',
    language: 'sql',
    code: `-- 原始查询
SELECT * FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.order_date > '2024-01-01'
ORDER BY o.order_date;

-- AI优化建议
-- 1. 避免SELECT *，只选择需要的列
-- 2. 添加索引：CREATE INDEX idx_orders_date ON orders(order_date)
-- 3. 使用分区表提升性能

-- 优化后查询
SELECT o.id, o.amount, o.order_date, c.name
FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.order_date > '2024-01-01'
ORDER BY o.order_date
LIMIT 1000; -- 添加限制避免大结果集`
  },
  {
    title: '对话式分析',
    language: 'text',
    code: `用户："分析一下我们的销售趋势"
AI："我来为您分析销售趋势。首先看看最近6个月的销售数据..."

用户："重点关注电子产品类别"
AI："好的，我来筛选电子产品的销售数据。发现以下趋势：
- 电子产品销售额呈上升趋势
- 手机类产品占比最高（45%）
- 季节性波动明显，Q4销售额最高"

用户："哪些因素影响了销售？"
AI："基于数据分析，主要影响因素包括：
1. 促销活动（相关性0.78）
2. 新品发布（相关性0.65）
3. 季节因素（相关性0.52）
建议在Q4加大促销力度，提升销售效果。"`
  }
]

const benefits = [
  {
    icon: Zap,
    title: '效率提升',
    description: '将数据查询时间从小时缩短到分钟，大幅提升分析效率'
  },
  {
    icon: CheckCircle,
    title: '降低门槛',
    description: '无需SQL专业知识，业务人员也能轻松进行数据分析'
  },
  {
    icon: Brain,
    title: '智能洞察',
    description: '基于AI的智能分析，发现人工难以察觉的数据洞察'
  }
]

/**
 * AI智能助手功能页面
 * 展示自然语言转SQL、智能查询优化、对话式分析等AI功能
 */
export default function AIAssistantPage() {
  return (
    <>
      <Hero
        title="AI智能助手"
        subtitle="自然语言转SQL和智能查询优化"
        description="基于先进AI技术的智能数据助手，支持自然语言查询、智能优化建议、对话式分析，让每个人都能轻松驾驭数据分析。"
        gradient="from-purple-500 to-pink-500"
        icon={Bot}
        benefits={benefits}
      />
      
      <FeatureDetails
        title="AI核心能力"
        subtitle="让人工智能成为您的数据分析专家"
        features={features}
      />
      
      <CodeExample
        title="AI助手演示"
        subtitle="体验AI驱动的智能数据分析"
        examples={codeExamples}
      />
      
      <CTA
        title="体验AI智能助手"
        description="立即开始您的智能数据分析之旅"
        primaryText="免费试用"
        secondaryText="查看演示"
        primaryHref="/contact"
        secondaryHref="/docs/ai-assistant"
      />
    </>
  )
}