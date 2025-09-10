# DuckHub 官网技术架构文档

## 📋 架构概述

基于Next.js 14 + React 18 + TypeScript + shadcn/ui构建的现代化官网，采用Supabase风格设计，突出DuckHub作为企业级金融数据湖平台的技术优势。

## 🏗️ 系统架构

### 整体架构图
```
┌─────────────────────────────────────────────────────────────┐
│                    用户访问层                                │
├─────────────────┬─────────────────┬─────────────────────────┤
│   Web浏览器      │   移动设备       │      搜索引擎爬虫        │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │ HTTPS
┌─────────────────────────────────────────────────────────────┐
│                    CDN层 (Vercel Edge)                     │
├─────────────────┬─────────────────┬─────────────────────────┤
│   静态资源缓存   │   边缘计算       │      全球加速            │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                  Next.js应用层                              │
├─────────────────┬─────────────────┬─────────────────────────┤
│   App Router     │   API Routes    │      中间件              │
│   (页面路由)     │   (API接口)     │   (认证/重定向)          │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    组件层                                   │
├─────────────────┬─────────────────┬─────────────────────────┤
│   页面组件       │   UI组件库       │      业务组件            │
│   (Pages)       │   (shadcn/ui)   │   (Features/Pricing)    │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    数据层                                   │
├─────────────────┬─────────────────┬─────────────────────────┤
│   静态内容       │   CMS内容        │      外部API             │
│   (MDX文件)     │   (Sanity/Strapi)│   (GitHub/Analytics)    │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## 🛠️ 技术栈详解

### 前端框架
- **Next.js 14**: 使用App Router，支持SSR/SSG/ISR
- **React 18**: 最新特性，Concurrent Features
- **TypeScript**: 类型安全，提升开发效率

### UI组件库
- **shadcn/ui**: 基于Radix UI的现代组件库
- **Tailwind CSS**: 原子化CSS框架
- **Lucide React**: 一致的图标系统
- **Framer Motion**: 流畅的动画效果

### 开发工具
- **ESLint**: 代码质量检查
- **Prettier**: 代码格式化
- **Husky**: Git钩子管理
- **Commitlint**: 提交信息规范

## 📁 项目结构

```
website/
├── app/                           # Next.js App Router
│   ├── (marketing)/              # 营销页面组
│   │   ├── page.tsx              # 首页
│   │   ├── features/             # 功能特性
│   │   │   ├── page.tsx
│   │   │   ├── ducklake/         # DuckLake功能
│   │   │   ├── ai-assistant/     # AI助手
│   │   │   └── enterprise/       # 企业级功能
│   │   ├── pricing/              # 定价页面
│   │   │   └── page.tsx
│   │   ├── about/                # 关于我们
│   │   │   └── page.tsx
│   │   └── contact/              # 联系我们
│   │       └── page.tsx
│   ├── docs/                     # 文档页面
│   │   ├── page.tsx
│   │   ├── getting-started/
│   │   ├── api-reference/
│   │   └── guides/
│   ├── blog/                     # 博客页面
│   │   ├── page.tsx
│   │   └── [slug]/
│   ├── api/                      # API路由
│   │   ├── contact/
│   │   └── newsletter/
│   ├── globals.css               # 全局样式
│   ├── layout.tsx                # 根布局
│   ├── loading.tsx               # 加载页面
│   ├── not-found.tsx             # 404页面
│   └── error.tsx                 # 错误页面
├── components/                   # 组件库
│   ├── ui/                       # shadcn/ui组件
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── dialog.tsx
│   │   └── ...
│   ├── marketing/                # 营销组件
│   │   ├── hero.tsx
│   │   ├── features.tsx
│   │   ├── testimonials.tsx
│   │   ├── pricing-table.tsx
│   │   └── cta-section.tsx
│   ├── docs/                     # 文档组件
│   │   ├── sidebar.tsx
│   │   ├── toc.tsx
│   │   └── code-block.tsx
│   ├── blog/                     # 博客组件
│   │   ├── post-card.tsx
│   │   └── post-content.tsx
│   └── common/                   # 通用组件
│       ├── header.tsx
│       ├── footer.tsx
│       ├── navigation.tsx
│       └── theme-provider.tsx
├── lib/                          # 工具函数
│   ├── utils.ts                  # 通用工具
│   ├── constants.ts              # 常量定义
│   ├── validations.ts            # 表单验证
│   └── analytics.ts              # 分析工具
├── hooks/                        # 自定义Hook
│   ├── use-scroll.ts
│   ├── use-intersection.ts
│   └── use-theme.ts
├── styles/                       # 样式文件
│   ├── globals.css
│   └── components.css
├── public/                       # 静态资源
│   ├── images/
│   ├── icons/
│   └── videos/
├── content/                      # 内容文件
│   ├── docs/                     # 文档内容
│   ├── blog/                     # 博客内容
│   └── pages/                    # 页面内容
├── config/                       # 配置文件
│   ├── site.ts                   # 站点配置
│   └── navigation.ts             # 导航配置
└── types/                        # 类型定义
    ├── index.ts
    └── content.ts
```

## 🎨 设计系统

### 颜色系统
```typescript
// 主色调 - 绿色系（参考Supabase）
const colors = {
  primary: {
    50: '#f0fdf4',   // 最浅绿
    100: '#dcfce7',  // 浅绿
    200: '#bbf7d0',  // 中浅绿
    300: '#86efac',  // 中绿
    400: '#4ade80',  // 亮绿
    500: '#22c55e',  // 标准绿
    600: '#16a34a',  // 深绿
    700: '#15803d',  // 更深绿
    800: '#166534',  // 很深绿
    900: '#14532d',  // 最深绿
  },
  secondary: {
    50: '#eff6ff',   // 最浅蓝
    500: '#3b82f6',  // 标准蓝
    600: '#2563eb',  // 深蓝
    700: '#1d4ed8',  // 更深蓝
  },
  accent: {
    purple: '#8b5cf6',
    pink: '#ec4899',
    orange: '#f97316',
  }
}
```

### 字体系统
```typescript
const typography = {
  fontFamily: {
    sans: ['Inter', 'system-ui', 'sans-serif'],
    mono: ['JetBrains Mono', 'Consolas', 'monospace'],
  },
  fontSize: {
    xs: '0.75rem',     // 12px
    sm: '0.875rem',    // 14px
    base: '1rem',      // 16px
    lg: '1.125rem',    // 18px
    xl: '1.25rem',     // 20px
    '2xl': '1.5rem',   // 24px
    '3xl': '1.875rem', // 30px
    '4xl': '2.25rem',  // 36px
    '5xl': '3rem',     // 48px
    '6xl': '3.75rem',  // 60px
  }
}
```

### 间距系统
```typescript
const spacing = {
  0: '0px',
  1: '0.25rem',   // 4px
  2: '0.5rem',    // 8px
  3: '0.75rem',   // 12px
  4: '1rem',      // 16px
  5: '1.25rem',   // 20px
  6: '1.5rem',    // 24px
  8: '2rem',      // 32px
  10: '2.5rem',   // 40px
  12: '3rem',     // 48px
  16: '4rem',     // 64px
  20: '5rem',     // 80px
  24: '6rem',     // 96px
}
```

## 🧩 核心组件设计

### 1. Header组件
```typescript
/**
 * 网站头部导航组件
 * 包含Logo、导航菜单、CTA按钮和移动端菜单
 */
export function Header() {
  const [isMenuOpen, setIsMenuOpen] = useState(false)
  const [isScrolled, setIsScrolled] = useState(false)

  // 监听滚动状态
  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 10)
    }
    window.addEventListener('scroll', handleScroll)
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  return (
    <header className={cn(
      "fixed top-0 w-full z-50 transition-all duration-200",
      isScrolled 
        ? "bg-white/80 backdrop-blur-md border-b border-gray-200" 
        : "bg-transparent"
    )}>
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <Link href="/" className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-gradient-to-br from-green-500 to-blue-500 rounded-lg flex items-center justify-center">
              <Database className="w-5 h-5 text-white" />
            </div>
            <span className="text-xl font-bold text-gray-900">DuckHub</span>
          </Link>

          {/* 桌面端导航 */}
          <nav className="hidden md:flex items-center space-x-8">
            <Link href="/features" className="text-gray-600 hover:text-gray-900 transition-colors">
              功能特性
            </Link>
            <Link href="/pricing" className="text-gray-600 hover:text-gray-900 transition-colors">
              定价
            </Link>
            <Link href="/docs" className="text-gray-600 hover:text-gray-900 transition-colors">
              文档
            </Link>
            <Link href="/blog" className="text-gray-600 hover:text-gray-900 transition-colors">
              博客
            </Link>
          </nav>

          {/* CTA按钮 */}
          <div className="hidden md:flex items-center space-x-4">
            <Button variant="ghost" asChild>
              <Link href="/login">登录</Link>
            </Button>
            <Button asChild>
              <Link href="/signup">开始使用</Link>
            </Button>
          </div>

          {/* 移动端菜单按钮 */}
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={() => setIsMenuOpen(!isMenuOpen)}
          >
            {isMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </Button>
        </div>
      </div>

      {/* 移动端菜单 */}
      <AnimatePresence>
        {isMenuOpen && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="md:hidden bg-white border-b border-gray-200"
          >
            <div className="container mx-auto px-4 py-4">
              <nav className="flex flex-col space-y-4">
                <Link href="/features" className="text-gray-600 hover:text-gray-900">
                  功能特性
                </Link>
                <Link href="/pricing" className="text-gray-600 hover:text-gray-900">
                  定价
                </Link>
                <Link href="/docs" className="text-gray-600 hover:text-gray-900">
                  文档
                </Link>
                <Link href="/blog" className="text-gray-600 hover:text-gray-900">
                  博客
                </Link>
                <div className="flex flex-col space-y-2 pt-4 border-t border-gray-200">
                  <Button variant="ghost" asChild>
                    <Link href="/login">登录</Link>
                  </Button>
                  <Button asChild>
                    <Link href="/signup">开始使用</Link>
                  </Button>
                </div>
              </nav>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </header>
  )
}
```

### 2. Hero组件
```typescript
/**
 * 首页Hero区域组件
 * 包含主标题、描述、CTA按钮和背景动画
 */
export function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* 背景渐变 */}
      <div className="absolute inset-0 bg-gradient-to-br from-green-50 via-blue-50 to-purple-50" />
      
      {/* 背景动画元素 */}
      <div className="absolute inset-0">
        <div className="absolute top-1/4 left-1/4 w-64 h-64 bg-green-200 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob" />
        <div className="absolute top-1/3 right-1/4 w-64 h-64 bg-blue-200 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob animation-delay-2000" />
        <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-purple-200 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob animation-delay-4000" />
      </div>

      <div className="relative z-10 container mx-auto px-4 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <h1 className="text-5xl md:text-7xl font-bold mb-6">
            <span className="bg-gradient-to-r from-green-600 via-blue-600 to-purple-600 bg-clip-text text-transparent">
              企业级金融
            </span>
            <br />
            <span className="bg-gradient-to-r from-purple-600 via-blue-600 to-green-600 bg-clip-text text-transparent">
              数据湖平台
            </span>
          </h1>
          
          <p className="text-xl md:text-2xl text-gray-600 mb-8 max-w-4xl mx-auto leading-relaxed">
            基于 <strong>DuckDB + DuckLake</strong> 构建的现代化数据平台，
            提供 <strong>ACID事务</strong>、<strong>时间旅行查询</strong> 和 <strong>AI智能分析</strong>
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
            <Button size="lg" className="bg-green-600 hover:bg-green-700 text-white px-8 py-4 text-lg">
              <Play className="w-5 h-5 mr-2" />
              观看演示
            </Button>
            <Button size="lg" variant="outline" className="px-8 py-4 text-lg">
              <Download className="w-5 h-5 mr-2" />
              开始使用
            </Button>
          </div>

          {/* 技术标签 */}
          <div className="flex flex-wrap justify-center gap-3">
            {['DuckDB', 'DuckLake', 'Rust', 'React', 'AI驱动', '企业级'].map((tag) => (
              <Badge key={tag} variant="secondary" className="px-3 py-1 text-sm">
                {tag}
              </Badge>
            ))}
          </div>
        </motion.div>
      </div>

      {/* 滚动指示器 */}
      <motion.div
        className="absolute bottom-8 left-1/2 transform -translate-x-1/2"
        animate={{ y: [0, 10, 0] }}
        transition={{ duration: 2, repeat: Infinity }}
      >
        <ChevronDown className="w-6 h-6 text-gray-400" />
      </motion.div>
    </section>
  )
}
```

### 3. Features组件
```typescript
/**
 * 功能特性展示组件
 * 展示DuckHub的核心功能和技术优势
 */
export function Features() {
  const features = [
    {
      icon: Database,
      title: "DuckLake数据湖",
      description: "ACID事务保证、时间旅行查询、Schema演进，构建可靠的数据湖基础设施",
      gradient: "from-blue-500 to-cyan-500",
      features: [
        "ACID事务支持",
        "时间旅行查询",
        "Schema演进",
        "快照管理"
      ]
    },
    {
      icon: Bot,
      title: "AI智能助手",
      description: "自然语言转SQL、智能查询优化、对话式数据分析，让数据分析更简单",
      gradient: "from-purple-500 to-pink-500",
      features: [
        "自然语言查询",
        "智能优化建议",
        "对话式分析",
        "异常检测"
      ]
    },
    {
      icon: Zap,
      title: "高性能引擎",
      description: "基于DuckDB的向量化执行、并行处理、智能缓存，提供极致性能",
      gradient: "from-orange-500 to-red-500",
      features: [
        "向量化执行",
        "并行处理",
        "查询缓存",
        "性能优化"
      ]
    },
    {
      icon: Shield,
      title: "企业级安全",
      description: "RBAC权限控制、数据加密、审计追踪，满足金融行业安全要求",
      gradient: "from-green-500 to-emerald-500",
      features: [
        "RBAC权限",
        "数据加密",
        "审计追踪",
        "合规支持"
      ]
    }
  ]

  return (
    <section className="py-24 bg-white">
      <div className="container mx-auto px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <h2 className="text-4xl md:text-5xl font-bold text-gray-900 mb-4">
            强大的功能特性
          </h2>
          <p className="text-xl text-gray-600 max-w-3xl mx-auto">
            为金融机构量身打造的数据湖解决方案，集成最新技术和最佳实践
          </p>
        </motion.div>

        <div className="grid md:grid-cols-2 gap-8">
          {features.map((feature, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
            >
              <FeatureCard {...feature} />
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  )
}

/**
 * 功能特性卡片组件
 */
function FeatureCard({ icon: Icon, title, description, gradient, features }) {
  return (
    <Card className="p-8 h-full hover:shadow-lg transition-shadow duration-300 border-0 bg-gradient-to-br from-gray-50 to-white">
      <div className="flex items-start space-x-4">
        <div className={`p-3 rounded-xl bg-gradient-to-r ${gradient} shadow-lg`}>
          <Icon className="w-6 h-6 text-white" />
        </div>
        <div className="flex-1">
          <h3 className="text-xl font-semibold text-gray-900 mb-2">{title}</h3>
          <p className="text-gray-600 mb-4">{description}</p>
          <ul className="space-y-2">
            {features.map((feature, index) => (
              <li key={index} className="flex items-center text-sm text-gray-500">
                <Check className="w-4 h-4 text-green-500 mr-2" />
                {feature}
              </li>
            ))}
          </ul>
        </div>
      </div>
    </Card>
  )
}
```

## 🚀 性能优化策略

### 1. 代码分割
```typescript
// 动态导入大型组件
const HeavyComponent = dynamic(() => import('./HeavyComponent'), {
  loading: () => <Skeleton className="w-full h-64" />,
  ssr: false
})

// 路由级别的代码分割
const BlogPage = dynamic(() => import('./blog/page'), {
  loading: () => <PageSkeleton />
})
```

### 2. 图片优化
```typescript
// 使用Next.js Image组件
import Image from 'next/image'

<Image
  src="/hero-image.jpg"
  alt="DuckHub Platform"
  width={1200}
  height={600}
  priority
  className="rounded-lg shadow-xl"
  placeholder="blur"
  blurDataURL="data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcUFhYaHSUfGhsjHBYWICwgIyYnKSopGR8tMC0oMCUoKSj/2wBDAQcHBwoIChMKChMoGhYaKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCj/wAARCAAIAAoDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAhEAACAQMDBQAAAAAAAAAAAAABAgMABAUGIWGRkqGx0f/EABUBAQEAAAAAAAAAAAAAAAAAAAMF/8QAGhEAAgIDAAAAAAAAAAAAAAAAAAECEgMRkf/aAAwDAQACEQMRAD8AltJagyeH0AthI5xdrLcNM91BF5pX2HaH9bcfaSXWGaRmknyJckliyjqTzSlT54b6bk+h0R//2Q=="
/>
```

### 3. 缓存策略
```typescript
// API路由缓存
export async function GET() {
  const data = await fetchData()
  
  return NextResponse.json(data, {
    headers: {
      'Cache-Control': 'public, s-maxage=3600, stale-while-revalidate=86400'
    }
  })
}

// 静态生成缓存
export const revalidate = 3600 // 1小时重新验证
```

### 4. SEO优化
```typescript
// 元数据配置
export const metadata: Metadata = {
  title: 'DuckHub - 企业级金融数据湖平台',
  description: '基于DuckDB + DuckLake构建的现代化数据平台，提供ACID事务、时间旅行查询和AI智能分析',
  keywords: ['数据湖', 'DuckDB', 'DuckLake', '金融数据', 'AI分析'],
  authors: [{ name: 'DuckHub Team' }],
  openGraph: {
    title: 'DuckHub - 企业级金融数据湖平台',
    description: '基于DuckDB + DuckLake构建的现代化数据平台',
    images: ['/og-image.jpg'],
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'DuckHub - 企业级金融数据湖平台',
    description: '基于DuckDB + DuckLake构建的现代化数据平台',
    images: ['/twitter-image.jpg'],
  },
}
```

## 📱 响应式设计

### 断点系统
```typescript
const breakpoints = {
  sm: '640px',   // 手机横屏
  md: '768px',   // 平板
  lg: '1024px',  // 笔记本
  xl: '1280px',  // 桌面
  '2xl': '1536px' // 大屏
}
```

### 响应式组件
```typescript
// 响应式网格布局
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
  {items.map(item => <ItemCard key={item.id} {...item} />)}
</div>

// 响应式字体大小
<h1 className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl font-bold">
  响应式标题
</h1>

// 响应式间距
<section className="py-12 md:py-16 lg:py-20 xl:py-24">
  内容区域
</section>
```

## 🔧 开发工具配置

### ESLint配置
```javascript
// .eslintrc.js
module.exports = {
  extends: [
    'next/core-web-vitals',
    '@typescript-eslint/recommended',
    'prettier'
  ],
  rules: {
    '@typescript-eslint/no-unused-vars': 'error',
    '@typescript-eslint/no-explicit-any': 'warn',
    'react-hooks/exhaustive-deps': 'warn'
  }
}
```

### Prettier配置
```javascript
// .prettierrc.js
module.exports = {
  semi: false,
  singleQuote: true,
  tabWidth: 2,
  trailingComma: 'es5',
  printWidth: 80,
  bracketSpacing: true,
  arrowParens: 'avoid'
}
```

### TypeScript配置
```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "es5",
    "lib": ["dom", "dom.iterable", "es6"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "node",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [
      {
        "name": "next"
      }
    ],
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

## 📊 监控和分析

### 性能监控
```typescript
// lib/analytics.ts
export function trackPageView(url: string) {
  if (typeof window !== 'undefined' && window.gtag) {
    window.gtag('config', 'GA_MEASUREMENT_ID', {
      page_path: url,
    })
  }
}

export function trackEvent(action: string, category: string, label?: string, value?: number) {
  if (typeof window !== 'undefined' && window.gtag) {
    window.gtag('event', action, {
      event_category: category,
      event_label: label,
      value: value,
    })
  }
}
```

### Web Vitals监控
```typescript
// app/layout.tsx
import { Analytics } from '@vercel/analytics/react'
import { SpeedInsights } from '@vercel/speed-insights/next'

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  )
}
```

## 🚀 部署配置

### Vercel配置
```json
// vercel.json
{
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "framework": "nextjs",
  "regions": ["hkg1", "sin1"],
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "X-Frame-Options",
          "value": "DENY"
        },
        {
          "key": "X-Content-Type-Options",
          "value": "nosniff"
        }
      ]
    }
  ],
  "redirects": [
    {
      "source": "/home",
      "destination": "/",
      "permanent": true
    }
  ]
}
```

### 环境变量
```bash
# .env.local
NEXT_PUBLIC_SITE_URL=https://duckhub.com
NEXT_PUBLIC_GA_ID=G-XXXXXXXXXX
DATABASE_URL=postgresql://...
EMAIL_SERVER_HOST=smtp.gmail.com
EMAIL_SERVER_PORT=587
```

## 📝 总结

这个技术架构文档详细说明了DuckHub官网的技术实现方案，采用现代化的技术栈和最佳实践，确保网站的性能、可维护性和用户体验。通过模块化的组件设计和完善的开发工具配置，为项目的长期发展奠定了坚实的基础。