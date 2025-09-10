"use client"

import { motion } from 'framer-motion'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Database, Bot, Zap, Shield, ArrowRight, CheckCircle } from 'lucide-react'
import Link from 'next/link'
import { cn } from '@/lib/utils'

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
      bgGradient: "from-blue-50 to-cyan-50",
      darkBgGradient: "from-blue-950/20 to-cyan-950/20",
      href: "/features/ducklake",
      features: [
        "ACID事务支持",
        "时间旅行查询",
        "Schema演进",
        "快照管理",
        "多云存储集成"
      ],
      stats: {
        label: "查询性能提升",
        value: "10-100x"
      }
    },
    {
      icon: Bot,
      title: "AI智能助手",
      description: "自然语言转SQL、智能查询优化、对话式数据分析，让数据分析更简单",
      gradient: "from-purple-500 to-pink-500",
      bgGradient: "from-purple-50 to-pink-50",
      darkBgGradient: "from-purple-950/20 to-pink-950/20",
      href: "/features/ai-assistant",
      features: [
        "自然语言查询",
        "智能优化建议",
        "对话式分析",
        "异常检测",
        "自动化报告"
      ],
      stats: {
        label: "分析效率提升",
        value: "5x"
      }
    },
    {
      icon: Zap,
      title: "高性能引擎",
      description: "基于DuckDB的向量化执行、并行处理、智能缓存，提供极致性能",
      gradient: "from-orange-500 to-red-500",
      bgGradient: "from-orange-50 to-red-50",
      darkBgGradient: "from-orange-950/20 to-red-950/20",
      href: "/features/performance",
      features: [
        "向量化执行",
        "并行处理",
        "查询缓存",
        "性能优化",
        "内存管理"
      ],
      stats: {
        label: "响应时间",
        value: "<100ms"
      }
    },
    {
      icon: Shield,
      title: "企业级安全",
      description: "RBAC权限控制、数据加密、审计追踪，满足金融行业安全要求",
      gradient: "from-green-500 to-emerald-500",
      bgGradient: "from-green-50 to-emerald-50",
      darkBgGradient: "from-green-950/20 to-emerald-950/20",
      href: "/features/enterprise",
      features: [
        "RBAC权限",
        "数据加密",
        "审计追踪",
        "合规支持",
        "访问控制"
      ],
      stats: {
        label: "安全等级",
        value: "金融级"
      }
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
            核心功能
          </Badge>
          <h2 className="text-4xl md:text-5xl font-bold text-foreground mb-4">
            强大的功能特性
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
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

        {/* 底部CTA */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mt-16"
        >
          <p className="text-lg text-muted-foreground mb-6">
            想了解更多功能特性？
          </p>
          <Button size="lg" asChild>
            <Link href="/features" className="flex items-center space-x-2">
              <span>查看所有功能</span>
              <ArrowRight className="w-4 h-4" />
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
interface FeatureCardProps {
  icon: React.ComponentType<{ className?: string }>
  title: string
  description: string
  gradient: string
  bgGradient: string
  darkBgGradient: string
  href: string
  features: string[]
  stats: {
    label: string
    value: string
  }
}

function FeatureCard({ 
  icon: Icon, 
  title, 
  description, 
  gradient, 
  bgGradient, 
  darkBgGradient, 
  href, 
  features, 
  stats 
}: FeatureCardProps) {
  return (
    <Card className={cn(
      "group h-full hover:shadow-xl transition-all duration-300 border-0 overflow-hidden",
      `bg-gradient-to-br ${bgGradient} dark:${darkBgGradient}`
    )}>
      <CardHeader className="pb-4">
        <div className="flex items-start justify-between">
          <div className="flex items-center space-x-4">
            <div className={cn(
              "p-3 rounded-xl shadow-lg transition-transform duration-300 group-hover:scale-110",
              `bg-gradient-to-r ${gradient}`
            )}>
              <Icon className="w-6 h-6 text-white" />
            </div>
            <div>
              <CardTitle className="text-xl font-semibold text-foreground mb-1">
                {title}
              </CardTitle>
              <Badge variant="outline" className="text-xs">
                {stats.label}: {stats.value}
              </Badge>
            </div>
          </div>
        </div>
        <CardDescription className="text-muted-foreground leading-relaxed">
          {description}
        </CardDescription>
      </CardHeader>
      
      <CardContent className="pt-0">
        {/* 功能列表 */}
        <div className="space-y-2 mb-6">
          {features.map((feature, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, x: -10 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
              className="flex items-center space-x-2"
            >
              <CheckCircle className="w-4 h-4 text-green-500 flex-shrink-0" />
              <span className="text-sm text-muted-foreground">{feature}</span>
            </motion.div>
          ))}
        </div>

        {/* 了解更多按钮 */}
        <Button 
          variant="ghost" 
          className="w-full group-hover:bg-white/50 dark:group-hover:bg-gray-800/50 transition-colors duration-300"
          asChild
        >
          <Link href={href} className="flex items-center justify-center space-x-2">
            <span>了解更多</span>
            <ArrowRight className="w-4 h-4 transition-transform duration-300 group-hover:translate-x-1" />
          </Link>
        </Button>
      </CardContent>
    </Card>
  )
}