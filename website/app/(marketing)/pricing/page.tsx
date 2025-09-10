import type { Metadata } from 'next'
import { PricingHero } from '@/components/marketing/pricing-hero'
import { PricingTable } from '@/components/marketing/pricing-table'
import { PricingFAQ } from '@/components/marketing/pricing-faq'
import { CTA } from '@/components/marketing/cta'

export const metadata: Metadata = {
  title: '定价方案 - 选择适合您的DuckHub版本',
  description: 'DuckHub提供开源版、专业版、企业版三种定价方案，满足不同规模企业的数据平台需求。',
  keywords: ['定价', '价格', '开源版', '专业版', '企业版', '订阅', '许可证'],
}

const pricingPlans = [
  {
    name: '开源版',
    price: '免费',
    period: '永久免费',
    description: '适合个人开发者和小团队',
    popular: false,
    features: [
      '基础数据湖功能',
      '标准SQL查询',
      '社区支持',
      '开源代码',
      '基础文档',
      '最多5个用户',
      '10GB存储空间',
      '基础监控'
    ],
    limitations: [
      '不包含AI功能',
      '不支持企业级安全',
      '无技术支持SLA',
      '无高级监控'
    ],
    ctaText: '开始使用',
    ctaHref: '/docs/getting-started'
  },
  {
    name: '专业版',
    price: '¥999',
    period: '每月',
    description: '适合中小企业和成长型团队',
    popular: true,
    features: [
      '完整数据湖功能',
      'AI智能助手',
      '高性能查询引擎',
      '技术支持',
      '高级监控',
      'SLA保证 (99.9%)',
      '无限用户',
      '1TB存储空间',
      '实时告警',
      '数据备份',
      'API访问',
      '集成支持'
    ],
    limitations: [
      '不支持私有化部署',
      '不包含定制开发',
      '标准技术支持'
    ],
    ctaText: '免费试用',
    ctaHref: '/contact'
  },
  {
    name: '企业版',
    price: '联系销售',
    period: '定制报价',
    description: '适合大型企业和金融机构',
    popular: false,
    features: [
      '所有专业版功能',
      '企业级安全',
      '私有化部署',
      '定制开发',
      '专属支持团队',
      '培训服务',
      'SLA保证 (99.99%)',
      '无限存储',
      '多云部署',
      '合规认证',
      '数据治理',
      '高级分析',
      '白标定制',
      '专业咨询'
    ],
    limitations: [],
    ctaText: '联系销售',
    ctaHref: '/contact'
  }
]

const comparisonFeatures = [
  {
    category: '核心功能',
    features: [
      {
        name: 'DuckLake数据湖',
        free: true,
        pro: true,
        enterprise: true
      },
      {
        name: 'ACID事务',
        free: true,
        pro: true,
        enterprise: true
      },
      {
        name: '时间旅行查询',
        free: true,
        pro: true,
        enterprise: true
      },
      {
        name: 'AI智能助手',
        free: false,
        pro: true,
        enterprise: true
      },
      {
        name: '高性能引擎',
        free: '基础',
        pro: '完整',
        enterprise: '完整+优化'
      }
    ]
  },
  {
    category: '安全与合规',
    features: [
      {
        name: '基础安全',
        free: true,
        pro: true,
        enterprise: true
      },
      {
        name: 'RBAC权限控制',
        free: false,
        pro: true,
        enterprise: true
      },
      {
        name: '数据加密',
        free: false,
        pro: '传输加密',
        enterprise: '全方位加密'
      },
      {
        name: '审计追踪',
        free: false,
        pro: true,
        enterprise: true
      },
      {
        name: '合规认证',
        free: false,
        pro: false,
        enterprise: true
      }
    ]
  },
  {
    category: '支持与服务',
    features: [
      {
        name: '社区支持',
        free: true,
        pro: true,
        enterprise: true
      },
      {
        name: '技术支持',
        free: false,
        pro: '工作时间',
        enterprise: '7x24小时'
      },
      {
        name: 'SLA保证',
        free: false,
        pro: '99.9%',
        enterprise: '99.99%'
      },
      {
        name: '培训服务',
        free: false,
        pro: false,
        enterprise: true
      },
      {
        name: '定制开发',
        free: false,
        pro: false,
        enterprise: true
      }
    ]
  },
  {
    category: '部署与扩展',
    features: [
      {
        name: '云端部署',
        free: true,
        pro: true,
        enterprise: true
      },
      {
        name: '私有化部署',
        free: false,
        pro: false,
        enterprise: true
      },
      {
        name: '多云支持',
        free: false,
        pro: false,
        enterprise: true
      },
      {
        name: '水平扩展',
        free: '有限',
        pro: '标准',
        enterprise: '无限'
      },
      {
        name: '负载均衡',
        free: false,
        pro: true,
        enterprise: true
      }
    ]
  }
]

const faqs = [
  {
    question: '开源版有什么限制？',
    answer: '开源版提供核心的数据湖功能，但不包含AI智能助手、企业级安全功能和技术支持。适合个人开发者和小团队进行学习和测试。'
  },
  {
    question: '可以从开源版升级到付费版本吗？',
    answer: '当然可以。您可以随时从开源版升级到专业版或企业版，数据可以无缝迁移，不会丢失任何信息。'
  },
  {
    question: '专业版包含多少技术支持？',
    answer: '专业版包含工作时间的技术支持，响应时间为4小时内。支持通过邮件、在线聊天等方式获得帮助。'
  },
  {
    question: '企业版的定制开发包含什么？',
    answer: '企业版的定制开发包括功能定制、界面定制、集成开发、性能优化等。我们会根据您的具体需求提供专业的开发服务。'
  },
  {
    question: '是否支持按年付费？',
    answer: '是的，我们支持按年付费，年付可享受2个月的优惠。企业版客户还可以选择多年期合同以获得更优惠的价格。'
  },
  {
    question: '如何申请试用？',
    answer: '您可以直接注册开源版进行试用，或者联系我们的销售团队申请专业版或企业版的试用。试用期为30天，包含完整功能。'
  }
]

/**
 * 定价页面
 * 展示不同版本的定价方案和功能对比
 */
export default function PricingPage() {
  return (
    <>
      <PricingHero />
      
      <PricingTable 
        plans={pricingPlans}
        comparisonFeatures={comparisonFeatures}
      />
      
      <PricingFAQ faqs={faqs} />
      
      <CTA
        title="准备开始使用DuckHub？"
        description="选择适合您的方案，立即开始构建现代化数据平台"
        primaryText="免费试用"
        secondaryText="联系销售"
        primaryHref="/docs/getting-started"
        secondaryHref="/contact"
      />
    </>
  )
}