import type { Metadata } from 'next'
import { DocsHero } from '@/components/docs/docs-hero'
import { DocsNavigation } from '@/components/docs/docs-navigation'
import { QuickStart } from '@/components/docs/quick-start'
import { PopularGuides } from '@/components/docs/popular-guides'

export const metadata: Metadata = {
  title: '文档中心 - DuckHub开发者文档',
  description: 'DuckHub完整的开发者文档，包含快速开始、API参考、使用指南和最佳实践。',
  keywords: ['文档', 'API', '开发者', '教程', '指南', '快速开始'],
}

const docsSections = [
  {
    title: '快速开始',
    description: '5分钟快速上手DuckHub',
    icon: '🚀',
    href: '/docs/getting-started',
    items: [
      { title: '安装部署', href: '/docs/getting-started/installation' },
      { title: '基础配置', href: '/docs/getting-started/configuration' },
      { title: '第一个查询', href: '/docs/getting-started/first-query' },
      { title: '连接数据源', href: '/docs/getting-started/data-sources' }
    ]
  },
  {
    title: 'API参考',
    description: '完整的API文档和示例',
    icon: '📚',
    href: '/docs/api-reference',
    items: [
      { title: 'REST API', href: '/docs/api-reference/rest-api' },
      { title: 'GraphQL API', href: '/docs/api-reference/graphql' },
      { title: 'WebSocket API', href: '/docs/api-reference/websocket' },
      { title: 'SDK参考', href: '/docs/api-reference/sdks' }
    ]
  },
  {
    title: '使用指南',
    description: '详细的功能使用教程',
    icon: '📖',
    href: '/docs/guides',
    items: [
      { title: '数据湖管理', href: '/docs/guides/data-lake' },
      { title: 'AI助手使用', href: '/docs/guides/ai-assistant' },
      { title: '性能优化', href: '/docs/guides/performance' },
      { title: '安全配置', href: '/docs/guides/security' }
    ]
  },
  {
    title: '示例代码',
    description: '实际项目中的代码示例',
    icon: '💻',
    href: '/docs/examples',
    items: [
      { title: '数据分析示例', href: '/docs/examples/analytics' },
      { title: '实时查询示例', href: '/docs/examples/real-time' },
      { title: '批处理示例', href: '/docs/examples/batch-processing' },
      { title: '集成示例', href: '/docs/examples/integrations' }
    ]
  },
  {
    title: '部署运维',
    description: '生产环境部署和运维指南',
    icon: '⚙️',
    href: '/docs/deployment',
    items: [
      { title: 'Docker部署', href: '/docs/deployment/docker' },
      { title: 'Kubernetes部署', href: '/docs/deployment/kubernetes' },
      { title: '监控告警', href: '/docs/deployment/monitoring' },
      { title: '备份恢复', href: '/docs/deployment/backup' }
    ]
  },
  {
    title: '故障排除',
    description: '常见问题和解决方案',
    icon: '🔧',
    href: '/docs/troubleshooting',
    items: [
      { title: '常见错误', href: '/docs/troubleshooting/common-errors' },
      { title: '性能问题', href: '/docs/troubleshooting/performance' },
      { title: '连接问题', href: '/docs/troubleshooting/connectivity' },
      { title: '日志分析', href: '/docs/troubleshooting/logs' }
    ]
  }
]

const popularGuides = [
  {
    title: '5分钟快速开始',
    description: '从安装到第一个查询的完整流程',
    href: '/docs/getting-started',
    readTime: '5分钟',
    difficulty: '初级'
  },
  {
    title: 'AI助手完整指南',
    description: '学习如何使用自然语言查询数据',
    href: '/docs/guides/ai-assistant',
    readTime: '15分钟',
    difficulty: '中级'
  },
  {
    title: '性能优化最佳实践',
    description: '提升查询性能的技巧和方法',
    href: '/docs/guides/performance',
    readTime: '20分钟',
    difficulty: '高级'
  },
  {
    title: '企业级安全配置',
    description: '配置RBAC权限和数据加密',
    href: '/docs/guides/security',
    readTime: '25分钟',
    difficulty: '高级'
  }
]

/**
 * 文档首页
 * 展示文档导航和快速开始指南
 */
export default function DocsPage() {
  return (
    <>
      <DocsHero />
      
      <QuickStart />
      
      <DocsNavigation sections={docsSections} />
      
      <PopularGuides guides={popularGuides} />
    </>
  )
}