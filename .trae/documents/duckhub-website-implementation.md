# DuckHub 官网实现计划

## 🚀 项目初始化

### 1. 创建项目结构

```bash
# 在duckhub项目根目录下创建website目录
cd /Users/louloulin/Documents/augment-projects/duckhub
mkdir website
cd website

# 初始化Next.js项目
npx create-next-app@latest . --typescript --tailwind --eslint --app --src-dir --import-alias "@/*"

# 安装核心依赖
npm install @radix-ui/react-accordion @radix-ui/react-alert-dialog @radix-ui/react-avatar @radix-ui/react-checkbox @radix-ui/react-dialog @radix-ui/react-dropdown-menu @radix-ui/react-label @radix-ui/react-navigation-menu @radix-ui/react-popover @radix-ui/react-progress @radix-ui/react-scroll-area @radix-ui/react-select @radix-ui/react-separator @radix-ui/react-sheet @radix-ui/react-slider @radix-ui/react-switch @radix-ui/react-tabs @radix-ui/react-toast @radix-ui/react-tooltip

# 安装UI和动画库
npm install lucide-react framer-motion class-variance-authority clsx tailwind-merge

# 安装开发工具
npm install -D @types/node prettier eslint-config-prettier

# 初始化shadcn/ui
npx shadcn-ui@latest init
```

### 2. 配置shadcn/ui组件

```bash
# 添加核心UI组件
npx shadcn-ui@latest add button
npx shadcn-ui@latest add card
npx shadcn-ui@latest add badge
npx shadcn-ui@latest add dialog
npx shadcn-ui@latest add dropdown-menu
npx shadcn-ui@latest add navigation-menu
npx shadcn-ui@latest add sheet
npx shadcn-ui@latest add tabs
npx shadcn-ui@latest add tooltip
npx shadcn-ui@latest add separator
npx shadcn-ui@latest add skeleton
```

## 📁 项目结构创建

### 创建目录结构

```bash
# 创建主要目录
mkdir -p app/{\(marketing\),docs,blog,api}
mkdir -p app/\(marketing\)/{features,pricing,about,contact}
mkdir -p components/{ui,marketing,docs,blog,common}
mkdir -p lib hooks styles public/{images,icons,videos} content/{docs,blog,pages} config types

# 创建具体功能目录
mkdir -p app/\(marketing\)/features/{ducklake,ai-assistant,enterprise,performance}
mkdir -p content/docs/{getting-started,api-reference,guides,examples}
mkdir -p public/images/{hero,features,testimonials,logos}
```

## 🎨 核心组件实现

### 1. 站点配置文件

```typescript
// config/site.ts
export const siteConfig = {
  name: "DuckHub",
  description: "企业级金融数据湖平台 - 基于DuckDB + DuckLake构建的现代化数据平台",
  url: "https://duckhub.com",
  ogImage: "https://duckhub.com/og.jpg",
  links: {
    twitter: "https://twitter.com/duckhub",
    github: "https://github.com/duckhub/duckhub",
    docs: "https://docs.duckhub.com",
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
    "ACID事务"
  ]
}

export type SiteConfig = typeof siteConfig
```

### 2. 导航配置

```typescript
// config/navigation.ts
import { Database, Bot, Zap, Shield, BookOpen, MessageCircle } from "lucide-react"

export const mainNav = [
  {
    title: "功能特性",
    href: "/features",
    description: "了解DuckHub的核心功能和技术优势",
    items: [
      {
        title: "DuckLake数据湖",
        href: "/features/ducklake",
        description: "ACID事务、时间旅行查询、Schema演进",
        icon: Database
      },
      {
        title: "AI智能助手",
        href: "/features/ai-assistant",
        description: "自然语言转SQL、智能查询优化",
        icon: Bot
      },
      {
        title: "高性能引擎",
        href: "/features/performance",
        description: "向量化执行、并行处理、查询缓存",
        icon: Zap
      },
      {
        title: "企业级安全",
        href: "/features/enterprise",
        description: "RBAC权限、数据加密、审计追踪",
        icon: Shield
      }
    ]
  },
  {
    title: "定价",
    href: "/pricing",
    description: "选择适合您的定价方案"
  },
  {
    title: "文档",
    href: "/docs",
    description: "完整的开发文档和API参考",
    items: [
      {
        title: "快速开始",
        href: "/docs/getting-started",
        description: "5分钟快速上手DuckHub",
        icon: BookOpen
      },
      {
        title: "API参考",
        href: "/docs/api-reference",
        description: "完整的API文档",
        icon: BookOpen
      },
      {
        title: "使用指南",
        href: "/docs/guides",
        description: "详细的使用教程",
        icon: BookOpen
      }
    ]
  },
  {
    title: "博客",
    href: "/blog",
    description: "技术文章和产品更新"
  }
]

export const footerNav = {
  product: [
    { title: "功能特性", href: "/features" },
    { title: "定价", href: "/pricing" },
    { title: "更新日志", href: "/changelog" },
    { title: "路线图", href: "/roadmap" }
  ],
  docs: [
    { title: "快速开始", href: "/docs/getting-started" },
    { title: "API参考", href: "/docs/api-reference" },
    { title: "使用指南", href: "/docs/guides" },
    { title: "示例", href: "/docs/examples" }
  ],
  company: [
    { title: "关于我们", href: "/about" },
    { title: "博客", href: "/blog" },
    { title: "联系我们", href: "/contact" },
    { title: "招聘", href: "/careers" }
  ],
  legal: [
    { title: "隐私政策", href: "/privacy" },
    { title: "服务条款", href: "/terms" },
    { title: "Cookie政策", href: "/cookies" }
  ]
}
```

### 3. 根布局组件

```typescript
// app/layout.tsx
import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import { ThemeProvider } from '@/components/common/theme-provider'
import { Toaster } from '@/components/ui/toaster'
import { Analytics } from '@vercel/analytics/react'
import { SpeedInsights } from '@vercel/speed-insights/next'
import { siteConfig } from '@/config/site'
import './globals.css'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: {
    default: siteConfig.name,
    template: `%s - ${siteConfig.name}`,
  },
  description: siteConfig.description,
  keywords: siteConfig.keywords,
  authors: [
    {
      name: "DuckHub Team",
      url: "https://duckhub.com",
    },
  ],
  creator: "DuckHub Team",
  metadataBase: new URL(siteConfig.url),
  openGraph: {
    type: "website",
    locale: "zh_CN",
    url: siteConfig.url,
    title: siteConfig.name,
    description: siteConfig.description,
    siteName: siteConfig.name,
    images: [
      {
        url: siteConfig.ogImage,
        width: 1200,
        height: 630,
        alt: siteConfig.name,
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: siteConfig.name,
    description: siteConfig.description,
    images: [siteConfig.ogImage],
    creator: "@duckhub",
  },
  icons: {
    icon: "/favicon.ico",
    shortcut: "/favicon-16x16.png",
    apple: "/apple-touch-icon.png",
  },
  manifest: `${siteConfig.url}/site.webmanifest`,
}

/**
 * 根布局组件
 * 提供全局样式、主题、分析工具等基础功能
 */
export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <head />
      <body className={inter.className}>
        <ThemeProvider
          attribute="class"
          defaultTheme="light"
          enableSystem
          disableTransitionOnChange
        >
          <div className="relative flex min-h-screen flex-col">
            <div className="flex-1">{children}</div>
          </div>
          <Toaster />
        </ThemeProvider>
        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  )
}
```

### 4. 营销页面布局

```typescript
// app/(marketing)/layout.tsx
import { Header } from '@/components/common/header'
import { Footer } from '@/components/common/footer'

/**
 * 营销页面布局组件
 * 包含头部导航和底部信息
 */
export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <>
      <Header />
      <main className="flex-1">{children}</main>
      <Footer />
    </>
  )
}
```

### 5. 头部导航组件

```typescript
// components/common/header.tsx
"use client"

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { motion, AnimatePresence } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import { NavigationMenu, NavigationMenuContent, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger } from '@/components/ui/navigation-menu'
import { Database, Menu, X, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { siteConfig } from '@/config/site'
import { mainNav } from '@/config/navigation'

/**
 * 网站头部导航组件
 * 包含Logo、导航菜单、CTA按钮和移动端菜单
 */
export function Header() {
  const [isScrolled, setIsScrolled] = useState(false)
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)
  const pathname = usePathname()

  // 监听滚动状态
  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 10)
    }
    
    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  return (
    <header className={cn(
      "sticky top-0 z-50 w-full border-b transition-all duration-200",
      isScrolled 
        ? "bg-background/80 backdrop-blur-md border-border" 
        : "bg-background/50 border-transparent"
    )}>
      <div className="container flex h-16 items-center justify-between">
        {/* Logo */}
        <Link href="/" className="flex items-center space-x-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-green-500 to-blue-500 shadow-sm">
            <Database className="h-5 w-5 text-white" />
          </div>
          <span className="text-xl font-bold">{siteConfig.name}</span>
        </Link>

        {/* 桌面端导航 */}
        <NavigationMenu className="hidden lg:flex">
          <NavigationMenuList>
            {mainNav.map((item) => (
              <NavigationMenuItem key={item.href}>
                {item.items ? (
                  <>
                    <NavigationMenuTrigger className="h-9">
                      {item.title}
                    </NavigationMenuTrigger>
                    <NavigationMenuContent>
                      <div className="grid w-[400px] gap-3 p-4 md:w-[500px] md:grid-cols-2 lg:w-[600px]">
                        {item.items.map((subItem) => (
                          <NavigationMenuLink key={subItem.href} asChild>
                            <Link
                              href={subItem.href}
                              className="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                            >
                              <div className="flex items-center space-x-2">
                                {subItem.icon && <subItem.icon className="h-4 w-4" />}
                                <div className="text-sm font-medium leading-none">
                                  {subItem.title}
                                </div>
                              </div>
                              <p className="line-clamp-2 text-sm leading-snug text-muted-foreground">
                                {subItem.description}
                              </p>
                            </Link>
                          </NavigationMenuLink>
                        ))}
                      </div>
                    </NavigationMenuContent>
                  </>
                ) : (
                  <NavigationMenuLink asChild>
                    <Link
                      href={item.href}
                      className={cn(
                        "group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none disabled:pointer-events-none disabled:opacity-50 data-[active]:bg-accent/50 data-[state=open]:bg-accent/50",
                        pathname === item.href && "bg-accent text-accent-foreground"
                      )}
                    >
                      {item.title}
                    </Link>
                  </NavigationMenuLink>
                )}
              </NavigationMenuItem>
            ))}
          </NavigationMenuList>
        </NavigationMenu>

        {/* CTA按钮 */}
        <div className="hidden items-center space-x-2 lg:flex">
          <Button variant="ghost" asChild>
            <Link href="/docs">文档</Link>
          </Button>
          <Button asChild>
            <Link href="/contact">开始使用</Link>
          </Button>
        </div>

        {/* 移动端菜单 */}
        <Sheet open={isMobileMenuOpen} onOpenChange={setIsMobileMenuOpen}>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon" className="lg:hidden">
              <Menu className="h-5 w-5" />
              <span className="sr-only">切换菜单</span>
            </Button>
          </SheetTrigger>
          <SheetContent side="right" className="w-[300px] sm:w-[400px]">
            <div className="flex flex-col space-y-4">
              <div className="flex items-center space-x-2">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-green-500 to-blue-500">
                  <Database className="h-5 w-5 text-white" />
                </div>
                <span className="text-xl font-bold">{siteConfig.name}</span>
              </div>
              
              <nav className="flex flex-col space-y-2">
                {mainNav.map((item) => (
                  <div key={item.href}>
                    <Link
                      href={item.href}
                      className="block px-2 py-1 text-lg font-medium hover:text-foreground/80"
                      onClick={() => setIsMobileMenuOpen(false)}
                    >
                      {item.title}
                    </Link>
                    {item.items && (
                      <div className="ml-4 mt-2 space-y-2">
                        {item.items.map((subItem) => (
                          <Link
                            key={subItem.href}
                            href={subItem.href}
                            className="block px-2 py-1 text-sm text-muted-foreground hover:text-foreground"
                            onClick={() => setIsMobileMenuOpen(false)}
                          >
                            {subItem.title}
                          </Link>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </nav>
              
              <div className="flex flex-col space-y-2 pt-4 border-t">
                <Button variant="ghost" asChild>
                  <Link href="/docs" onClick={() => setIsMobileMenuOpen(false)}>
                    文档
                  </Link>
                </Button>
                <Button asChild>
                  <Link href="/contact" onClick={() => setIsMobileMenuOpen(false)}>
                    开始使用
                  </Link>
                </Button>
              </div>
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </header>
  )
}
```

### 6. Hero区域组件

```typescript
// components/marketing/hero.tsx
"use client"

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { motion } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Play, Download, ChevronDown, Database, Zap, Bot, Shield } from 'lucide-react'
import { cn } from '@/lib/utils'

/**
 * 首页Hero区域组件
 * 包含主标题、描述、CTA按钮和背景动画
 */
export function Hero() {
  const [mousePosition, setMousePosition] = useState({ x: 0, y: 0 })

  // 鼠标跟踪效果
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      setMousePosition({ x: e.clientX, y: e.clientY })
    }
    
    window.addEventListener('mousemove', handleMouseMove)
    return () => window.removeEventListener('mousemove', handleMouseMove)
  }, [])

  const features = [
    { icon: Database, label: "DuckLake数据湖" },
    { icon: Bot, label: "AI智能助手" },
    { icon: Zap, label: "高性能引擎" },
    { icon: Shield, label: "企业级安全" }
  ]

  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* 背景渐变 */}
      <div className="absolute inset-0 bg-gradient-to-br from-green-50 via-blue-50 to-purple-50" />
      
      {/* 动态背景元素 */}
      <div className="absolute inset-0 overflow-hidden">
        <div 
          className="absolute w-96 h-96 bg-green-200 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob"
          style={{
            left: `${20 + mousePosition.x * 0.02}%`,
            top: `${20 + mousePosition.y * 0.02}%`,
          }}
        />
        <div 
          className="absolute w-96 h-96 bg-blue-200 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob animation-delay-2000"
          style={{
            right: `${20 - mousePosition.x * 0.02}%`,
            top: `${30 + mousePosition.y * 0.01}%`,
          }}
        />
        <div 
          className="absolute w-96 h-96 bg-purple-200 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob animation-delay-4000"
          style={
            bottom: `${20 - mousePosition.y * 0.02}%`,
            left: `${30 + mousePosition.x * 0.01}%`,
          }}
        />
      </div>

      {/* 网格背景 */}
      <div className="absolute inset-0 bg-grid-slate-100 [mask-image:linear-gradient(0deg,white,rgba(255,255,255,0.6))] dark:bg-grid-slate-700/25 dark:[mask-image:linear-gradient(0deg,rgba(255,255,255,0.1),rgba(255,255,255,0.5))]" />

      <div className="relative z-10 container mx-auto px-4 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="max-w-5xl mx-auto"
        >
          {/* 版本标签 */}
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: 0.2 }}
            className="mb-8"
          >
            <Badge variant="secondary" className="px-4 py-2 text-sm font-medium">
              🎉 DuckHub v2.0 现已发布 - 全新AI功能
            </Badge>
          </motion.div>
          
          {/* 主标题 */}
          <motion.h1 
            className="text-4xl md:text-6xl lg:text-7xl font-bold mb-6 leading-tight"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
          >
            <span className="bg-gradient-to-r from-green-600 via-blue-600 to-purple-600 bg-clip-text text-transparent">
              企业级金融
            </span>
            <br />
            <span className="bg-gradient-to-r from-purple-600 via-blue-600 to-green-600 bg-clip-text text-transparent">
              数据湖平台
            </span>
          </motion.h1>
          
          {/* 副标题 */}
          <motion.p 
            className="text-xl md:text-2xl text-muted-foreground mb-8 max-w-4xl mx-auto leading-relaxed"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4 }}
          >
            基于 <strong className="text-foreground">DuckDB + DuckLake</strong> 构建的现代化数据平台，
            提供 <strong className="text-foreground">ACID事务</strong>、<strong className="text-foreground">时间旅行查询</strong> 和 <strong className="text-foreground">AI智能分析</strong>
          </motion.p>

          {/* CTA按钮 */}
          <motion.div 
            className="flex flex-col sm:flex-row gap-4 justify-center mb-12"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5 }}
          >
            <Button size="lg" className="bg-green-600 hover:bg-green-700 text-white px-8 py-4 text-lg h-14">
              <Play className="w-5 h-5 mr-2" />
              观看演示
            </Button>
            <Button size="lg" variant="outline" className="px-8 py-4 text-lg h-14">
              <Download className="w-5 h-5 mr-2" />
              开始使用
            </Button>
          </motion.div>

          {/* 功能特性标签 */}
          <motion.div 
            className="flex flex-wrap justify-center gap-6 mb-16"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.6 }}
          >
            {features.map((feature, index) => (
              <motion.div
                key={feature.label}
                className="flex items-center space-x-2 px-4 py-2 bg-white/50 backdrop-blur-sm rounded-full border border-white/20"
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.7 + index * 0.1 }}
                whileHover={{ scale: 1.05 }}
              >
                <feature.icon className="w-4 h-4 text-green-600" />
                <span className="text-sm font-medium text-gray-700">{feature.label}</span>
              </motion.div>
            ))}
          </motion.div>

          {/* 统计数据 */}
          <motion.div 
            className="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-2xl mx-auto"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.8 }}
          >
            {[
              { value: "10-100x", label: "性能提升" },
              { value: "<100ms", label: "查询响应" },
              { value: "99.9%", label: "可用性" },
              { value: "1000+", label: "QPS支持" }
            ].map((stat, index) => (
              <div key={stat.label} className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-green-600 mb-1">
                  {stat.value}
                </div>
                <div className="text-sm text-muted-foreground">{stat.label}</div>
              </div>
            ))}
          </motion.div>
        </motion.div>
      </div>

      {/* 滚动指示器 */}
      <motion.div
        className="absolute bottom-8 left-1/2 transform -translate-x-1/2"
        animate={{ y: [0, 10, 0] }}
        transition={{ duration: 2, repeat: Infinity }}
      >
        <ChevronDown className="w-6 h-6 text-muted-foreground" />
      </motion.div>
    </section>
  )
}
```

### 7. 功能特性组件

```typescript
// components/marketing/features.tsx
"use client"

import { motion } from 'framer-motion'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Database, Bot, Zap, Shield, Check, ArrowRight } from 'lucide-react'
import Link from 'next/link'

/**
 * 功能特性展示组件
 * 展示DuckHub的核心功能和技术优势
 */
export function Features() {
  const features = [
    {
      icon: Database,
      title: "DuckLake数据湖",
      description: "企业级数据湖解决方案，提供ACID事务保证、时间旅行查询和Schema演进能力",
      gradient: "from-blue-500 to-cyan-500",
      features: [
        "ACID事务支持",
        "时间旅行查询",
        "Schema演进",
        "快照管理",
        "多云存储"
      ],
      href: "/features/ducklake",
      badge: "核心功能"
    },
    {
      icon: Bot,
      title: "AI智能助手",
      description: "基于大语言模型的智能数据分析助手，支持自然语言查询和智能优化建议",
      gradient: "from-purple-500 to-pink-500",
      features: [
        "自然语言转SQL",
        "智能查询优化",
        "对话式分析",
        "异常检测",
        "智能推荐"
      ],
      href: "/features/ai-assistant",
      badge: "AI驱动"
    },
    {
      icon: Zap,
      title: "高性能引擎",
      description: "基于DuckDB的向量化执行引擎，提供极致的查询性能和并行处理能力",
      gradient: "from-orange-500 to-red-500",
      features: [
        "向量化执行",
        "并行处理",
        "智能缓存",
        "查询优化",
        "内存管理"
      ],
      href: "/features/performance",
      badge: "高性能"
    },
    {
      icon: Shield,
      title: "企业级安全",
      description: "完整的安全体系，包括RBAC权限控制、数据加密和审计追踪，满足金融行业要求",
      gradient: "from-green-500 to-emerald-500",
      features: [
        "RBAC权限控制",
        "数据加密",
        "审计追踪",
        "合规支持",
        "安全监控"
      ],
      href: "/features/enterprise",
      badge: "企业级"
    }
  ]

  return (
    <section className="py-24 bg-gradient-to-b from-white to-gray-50">
      <div className="container mx-auto px-4">
        {/* 标题区域 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <Badge variant="secondary" className="mb-4">
            核心功能
          </Badge>
          <h2 className="text-4xl md:text-5xl font-bold text-gray-900 mb-4">
            强大的功能特性
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
            为金融机构量身打造的数据湖解决方案，集成最新技术和最佳实践
          </p>
        </motion.div>

        {/* 功能网格 */}
        <div className="grid md:grid-cols-2 gap-8 mb-16">
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

        {/* CTA区域 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center"
        >
          <Button size="lg" asChild>
            <Link href="/features">
              查看所有功能
              <ArrowRight className="w-4 h-4 ml-2" />
            </Link>
          </Button>
        </motion.div>
      </div>
    </section>
  )
}

/**
 * 功能特性卡片组件
 */
function FeatureCard({ icon: Icon, title, description, gradient, features, href, badge }) {
  return (
    <Card className="group h-full hover:shadow-xl transition-all duration-300 border-0 bg-white relative overflow-hidden">
      {/* 背景渐变 */}
      <div className={`absolute inset-0 bg-gradient-to-br ${gradient} opacity-0 group-hover:opacity-5 transition-opacity duration-300`} />
      
      <CardContent className="p-8">
        <div className="flex items-start justify-between mb-4">
          <div className={`p-3 rounded-xl bg-gradient-to-r ${gradient} shadow-lg`}>
            <Icon className="w-6 h-6 text-white" />
          </div>
          <Badge variant="outline" className="text-xs">
            {badge}
          </Badge>
        </div>
        
        <h3 className="text-xl font-semibold text-gray-900 mb-3 group-hover:text-gray-800 transition-colors">
          {title}
        </h3>
        
        <p className="text-muted-foreground mb-6 leading-relaxed">
          {description}
        </p>
        
        <ul className="space-y-2 mb-6">
          {features.map((feature, index) => (
            <li key={index} className="flex items-center text-sm text-gray-600">
              <Check className="w-4 h-4 text-green-500 mr-2 flex-shrink-0" />
              {feature}
            </li>
          ))}
        </ul>
        
        <Button variant="ghost" className="w-full group-hover:bg-gray-50" asChild>
          <Link href={href}>
            了解更多
            <ArrowRight className="w-4 h-4 ml-2 group-hover:translate-x-1 transition-transform" />
          </Link>
        </Button>
      </CardContent>
    </Card>
  )
}
```

## 📅 开发时间表

### Week 1: 项目搭建和基础组件
- [x] 项目初始化和依赖安装
- [x] 基础架构搭建
- [x] 设计系统建立
- [x] 核心组件开发（Header, Hero, Features）

### Week 2: 首页和功能页面
- [ ] 完善首页所有组件
- [ ] 开发功能特性详情页
- [ ] 实现响应式设计
- [ ] 添加动画效果

### Week 3: 内容页面
- [ ] 定价页面开发
- [ ] 文档页面框架
- [ ] 关于我们页面
- [ ] 联系我们页面

### Week 4: 优化和部署
- [ ] 性能优化
- [ ] SEO优化
- [ ] 测试和调试
- [ ] 部署配置

## 🚀 下一步行动

1. **立即开始项目搭建**
   ```bash
   cd /Users/louloulin/Documents/augment-projects/duckhub
   mkdir website && cd website
   # 执行上述初始化命令
   ```

2. **创建基础组件**
   - 实现Header组件
   - 实现Hero组件
   - 实现Features组件

3. **完善页面内容**
   - 添加更多营销内容
   - 优化用户体验
   - 完善响应式设计

4. **部署和测试**
   - 配置Vercel部署
   - 进行性能测试
   - 收集用户反馈

通过这个详细的实现计划，我们可以快速构建一个现代化、高质量的DuckHub官网，有效展示产品的技术优势和商业价值。