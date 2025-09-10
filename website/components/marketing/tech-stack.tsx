"use client"

import { motion } from 'framer-motion'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ExternalLink, Code, Database, Cpu, Shield, Cloud, Zap } from 'lucide-react'
import Link from 'next/link'
import { cn } from '@/lib/utils'

/**
 * 技术栈展示组件
 * 展示DuckHub使用的核心技术和架构
 */
export function TechStack() {
  const techCategories = [
    {
      title: "数据引擎",
      icon: Database,
      gradient: "from-blue-500 to-cyan-500",
      bgColor: "bg-blue-50 dark:bg-blue-950/20",
      technologies: [
        {
          name: "DuckDB",
          description: "高性能分析数据库",
          logo: "/logos/duckdb.svg",
          url: "https://duckdb.org",
          featured: true
        },
        {
          name: "DuckLake",
          description: "ACID事务数据湖",
          logo: "/logos/ducklake.svg",
          url: "#",
          featured: true
        },
        {
          name: "Apache Arrow",
          description: "内存列式格式",
          logo: "/logos/arrow.svg",
          url: "https://arrow.apache.org"
        }
      ]
    },
    {
      title: "后端架构",
      icon: Cpu,
      gradient: "from-orange-500 to-red-500",
      bgColor: "bg-orange-50 dark:bg-orange-950/20",
      technologies: [
        {
          name: "Rust",
          description: "系统级编程语言",
          logo: "/logos/rust.svg",
          url: "https://rust-lang.org",
          featured: true
        },
        {
          name: "Actix-Web",
          description: "高性能Web框架",
          logo: "/logos/actix.svg",
          url: "https://actix.rs"
        },
        {
          name: "Tokio",
          description: "异步运行时",
          logo: "/logos/tokio.svg",
          url: "https://tokio.rs"
        }
      ]
    },
    {
      title: "AI & 智能",
      icon: Zap,
      gradient: "from-purple-500 to-pink-500",
      bgColor: "bg-purple-50 dark:bg-purple-950/20",
      technologies: [
        {
          name: "Rig Framework",
          description: "AI应用开发框架",
          logo: "/logos/rig.svg",
          url: "https://rig.rs",
          featured: true
        },
        {
          name: "DeepSeek",
          description: "大语言模型",
          logo: "/logos/deepseek.svg",
          url: "https://deepseek.com"
        },
        {
          name: "OpenAI",
          description: "AI服务集成",
          logo: "/logos/openai.svg",
          url: "https://openai.com"
        }
      ]
    },
    {
      title: "前端技术",
      icon: Code,
      gradient: "from-green-500 to-emerald-500",
      bgColor: "bg-green-50 dark:bg-green-950/20",
      technologies: [
        {
          name: "React",
          description: "用户界面库",
          logo: "/logos/react.svg",
          url: "https://react.dev",
          featured: true
        },
        {
          name: "TypeScript",
          description: "类型安全的JavaScript",
          logo: "/logos/typescript.svg",
          url: "https://typescriptlang.org"
        },
        {
          name: "shadcn/ui",
          description: "现代UI组件库",
          logo: "/logos/shadcn.svg",
          url: "https://ui.shadcn.com"
        }
      ]
    },
    {
      title: "云原生",
      icon: Cloud,
      gradient: "from-indigo-500 to-purple-500",
      bgColor: "bg-indigo-50 dark:bg-indigo-950/20",
      technologies: [
        {
          name: "Docker",
          description: "容器化平台",
          logo: "/logos/docker.svg",
          url: "https://docker.com",
          featured: true
        },
        {
          name: "Kubernetes",
          description: "容器编排",
          logo: "/logos/kubernetes.svg",
          url: "https://kubernetes.io"
        },
        {
          name: "Prometheus",
          description: "监控和告警",
          logo: "/logos/prometheus.svg",
          url: "https://prometheus.io"
        }
      ]
    },
    {
      title: "安全合规",
      icon: Shield,
      gradient: "from-teal-500 to-green-500",
      bgColor: "bg-teal-50 dark:bg-teal-950/20",
      technologies: [
        {
          name: "OAuth 2.0",
          description: "身份认证协议",
          logo: "/logos/oauth.svg",
          url: "https://oauth.net"
        },
        {
          name: "TLS/SSL",
          description: "传输层安全",
          logo: "/logos/ssl.svg",
          url: "#"
        },
        {
          name: "RBAC",
          description: "基于角色的访问控制",
          logo: "/logos/rbac.svg",
          url: "#",
          featured: true
        }
      ]
    }
  ]

  return (
    <section className="py-24 bg-background">
      <div className="container mx-auto px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <Badge variant="secondary" className="mb-4">
            技术架构
          </Badge>
          <h2 className="text-4xl md:text-5xl font-bold text-foreground mb-4">
            现代化技术栈
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
            基于最新技术构建的企业级数据湖平台，确保性能、安全性和可扩展性
          </p>
        </motion.div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
          {techCategories.map((category, index) => (
            <motion.div
              key={category.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
            >
              <TechCategoryCard category={category} />
            </motion.div>
          ))}
        </div>

        {/* 架构图CTA */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mt-16"
        >
          <div className="bg-gradient-to-r from-green-50 to-blue-50 dark:from-green-950/10 dark:to-blue-950/10 rounded-2xl p-8 border border-green-100 dark:border-green-900/20">
            <h3 className="text-2xl font-bold text-foreground mb-4">
              想了解完整的系统架构？
            </h3>
            <p className="text-muted-foreground mb-6 max-w-2xl mx-auto">
              查看详细的技术文档，了解DuckHub如何通过现代化架构实现企业级性能和安全性
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Button size="lg" asChild>
                <Link href="/docs/architecture" className="flex items-center space-x-2">
                  <span>查看架构文档</span>
                  <ExternalLink className="w-4 h-4" />
                </Link>
              </Button>
              <Button size="lg" variant="outline" asChild>
                <Link href="/docs/getting-started" className="flex items-center space-x-2">
                  <span>快速开始</span>
                </Link>
              </Button>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  )
}

/**
 * 技术分类卡片组件
 */
interface TechCategoryCardProps {
  category: {
    title: string
    icon: React.ComponentType<{ className?: string }>
    gradient: string
    bgColor: string
    technologies: Array<{
      name: string
      description: string
      logo: string
      url: string
      featured?: boolean
    }>
  }
}

function TechCategoryCard({ category }: TechCategoryCardProps) {
  const Icon = category.icon
  
  return (
    <Card className={cn("h-full hover:shadow-lg transition-all duration-300 border-0", category.bgColor)}>
      <CardContent className="p-6">
        {/* 分类标题 */}
        <div className="flex items-center space-x-3 mb-6">
          <div className={cn(
            "w-10 h-10 rounded-xl flex items-center justify-center shadow-lg",
            `bg-gradient-to-r ${category.gradient}`
          )}>
            <Icon className="w-5 h-5 text-white" />
          </div>
          <h3 className="text-lg font-semibold text-foreground">
            {category.title}
          </h3>
        </div>
        
        {/* 技术列表 */}
        <div className="space-y-4">
          {category.technologies.map((tech, index) => (
            <motion.div
              key={tech.name}
              initial={{ opacity: 0, x: -10 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
            >
              <TechItem tech={tech} />
            </motion.div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

/**
 * 技术项目组件
 */
interface TechItemProps {
  tech: {
    name: string
    description: string
    logo: string
    url: string
    featured?: boolean
  }
}

function TechItem({ tech }: TechItemProps) {
  return (
    <div className="flex items-center space-x-3 p-3 rounded-lg hover:bg-white/50 dark:hover:bg-gray-800/50 transition-colors group">
      {/* Logo占位符 */}
      <div className="w-8 h-8 rounded-lg bg-gradient-to-r from-gray-200 to-gray-300 dark:from-gray-700 dark:to-gray-600 flex items-center justify-center flex-shrink-0">
        <Code className="w-4 h-4 text-gray-600 dark:text-gray-300" />
      </div>
      
      <div className="flex-1 min-w-0">
        <div className="flex items-center space-x-2">
          <span className="font-medium text-foreground text-sm">
            {tech.name}
          </span>
          {tech.featured && (
            <Badge variant="secondary" className="text-xs px-2 py-0">
              核心
            </Badge>
          )}
        </div>
        <p className="text-xs text-muted-foreground mt-1">
          {tech.description}
        </p>
      </div>
      
      {tech.url !== '#' && (
        <Button
          variant="ghost"
          size="icon"
          className="opacity-0 group-hover:opacity-100 transition-opacity w-6 h-6"
          asChild
        >
          <Link href={tech.url} target="_blank" rel="noopener noreferrer">
            <ExternalLink className="w-3 h-3" />
          </Link>
        </Button>
      )}
    </div>
  )
}