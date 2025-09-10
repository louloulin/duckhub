export const siteConfig = {
  name: "DuckHub",
  description: "企业级金融数据湖平台 - 基于DuckDB + DuckLake构建的现代化数据平台，提供ACID事务、时间旅行查询和AI智能分析",
  url: "https://duckhub.com",
  ogImage: "https://duckhub.com/og.jpg",
  links: {
    twitter: "https://twitter.com/duckhub",
    github: "https://github.com/duckhub/duckhub",
    docs: "https://docs.duckhub.com",
    discord: "https://discord.gg/duckhub",
    linkedin: "https://linkedin.com/company/duckhub",
  },
  keywords: [
    "数据湖",
    "DuckDB",
    "DuckLake",
    "金融数据",
    "AI分析",
    "企业级",
    "OLAP",
    "时间旅行查询",
    "ACID事务",
    "数据平台",
    "商业智能",
    "数据仓库",
    "实时分析",
    "云原生",
    "微服务"
  ],
  authors: [
    {
      name: "DuckHub Team",
      url: "https://duckhub.com",
    },
  ],
  creator: "DuckHub Team",
  company: {
    name: "DuckHub Inc.",
    address: "北京市朝阳区",
    email: "contact@duckhub.com",
    phone: "+86 400-123-4567",
  },
  features: {
    ducklake: {
      title: "DuckLake数据湖",
      description: "ACID事务、时间旅行查询、Schema演进",
      icon: "database",
    },
    ai: {
      title: "AI智能助手",
      description: "自然语言转SQL、智能查询优化",
      icon: "bot",
    },
    performance: {
      title: "高性能引擎",
      description: "向量化执行、并行处理、查询缓存",
      icon: "zap",
    },
    security: {
      title: "企业级安全",
      description: "RBAC权限、数据加密、审计追踪",
      icon: "shield",
    },
  },
  stats: {
    performance: "10-100x",
    responseTime: "<100ms",
    availability: "99.9%",
    qps: "1000+",
  },
  pricing: {
    free: {
      name: "开源版",
      price: "免费",
      description: "适合个人开发者和小团队",
      features: [
        "基础数据湖功能",
        "社区支持",
        "开源代码",
        "基础文档",
      ],
    },
    pro: {
      name: "专业版",
      price: "¥999/月",
      description: "适合中小企业",
      features: [
        "完整数据湖功能",
        "AI智能助手",
        "技术支持",
        "高级监控",
        "SLA保证",
      ],
    },
    enterprise: {
      name: "企业版",
      price: "联系销售",
      description: "适合大型企业",
      features: [
        "所有专业版功能",
        "私有化部署",
        "定制开发",
        "专属支持",
        "培训服务",
      ],
    },
  },
}

export type SiteConfig = typeof siteConfig

// 导航配置
export const navConfig = {
  main: [
    {
      title: "功能特性",
      href: "/features",
      description: "了解DuckHub的核心功能和技术优势",
    },
    {
      title: "定价",
      href: "/pricing",
      description: "选择适合您的定价方案",
    },
    {
      title: "文档",
      href: "/docs",
      description: "完整的开发文档和API参考",
    },
    {
      title: "博客",
      href: "/blog",
      description: "技术文章和产品更新",
    },
  ],
  features: [
    {
      title: "DuckLake数据湖",
      href: "/features/ducklake",
      description: "ACID事务、时间旅行查询、Schema演进",
      icon: "database",
    },
    {
      title: "AI智能助手",
      href: "/features/ai-assistant",
      description: "自然语言转SQL、智能查询优化",
      icon: "bot",
    },
    {
      title: "高性能引擎",
      href: "/features/performance",
      description: "向量化执行、并行处理、查询缓存",
      icon: "zap",
    },
    {
      title: "企业级安全",
      href: "/features/enterprise",
      description: "RBAC权限、数据加密、审计追踪",
      icon: "shield",
    },
  ],
  docs: [
    {
      title: "快速开始",
      href: "/docs/getting-started",
      description: "5分钟快速上手DuckHub",
    },
    {
      title: "API参考",
      href: "/docs/api-reference",
      description: "完整的API文档",
    },
    {
      title: "使用指南",
      href: "/docs/guides",
      description: "详细的使用教程",
    },
    {
      title: "示例",
      href: "/docs/examples",
      description: "代码示例和最佳实践",
    },
  ],
  footer: {
    product: [
      { title: "功能特性", href: "/features" },
      { title: "定价", href: "/pricing" },
      { title: "更新日志", href: "/changelog" },
      { title: "路线图", href: "/roadmap" },
    ],
    docs: [
      { title: "快速开始", href: "/docs/getting-started" },
      { title: "API参考", href: "/docs/api-reference" },
      { title: "使用指南", href: "/docs/guides" },
      { title: "示例", href: "/docs/examples" },
    ],
    company: [
      { title: "关于我们", href: "/about" },
      { title: "博客", href: "/blog" },
      { title: "联系我们", href: "/contact" },
      { title: "招聘", href: "/careers" },
    ],
    legal: [
      { title: "隐私政策", href: "/privacy" },
      { title: "服务条款", href: "/terms" },
      { title: "Cookie政策", href: "/cookies" },
    ],
  },
}

export type NavConfig = typeof navConfig