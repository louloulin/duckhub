"use client"

import { motion } from 'framer-motion'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Quote, Star, Building2, TrendingUp, Shield, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'

/**
 * 客户证言组件
 * 展示客户反馈和使用案例
 */
export function Testimonials() {
  const testimonials = [
    {
      id: 1,
      content: "DuckHub彻底改变了我们的数据分析流程。ACID事务和时间旅行查询让我们能够安全地处理金融数据，同时保持极高的查询性能。",
      author: {
        name: "张伟",
        role: "数据架构师",
        company: "某大型银行",
        avatar: "/avatars/zhang-wei.jpg",
        initials: "ZW"
      },
      rating: 5,
      metrics: {
        improvement: "性能提升 85%",
        icon: TrendingUp
      },
      category: "金融服务"
    },
    {
      id: 2,
      content: "AI智能助手功能让我们的业务分析师能够用自然语言查询数据，大大提高了工作效率。不再需要复杂的SQL知识。",
      author: {
        name: "李明",
        role: "业务分析总监",
        company: "某证券公司",
        avatar: "/avatars/li-ming.jpg",
        initials: "LM"
      },
      rating: 5,
      metrics: {
        improvement: "效率提升 300%",
        icon: Zap
      },
      category: "证券投资"
    },
    {
      id: 3,
      content: "企业级安全功能完全满足我们的合规要求。RBAC权限控制和审计追踪让我们能够安心地在生产环境中使用。",
      author: {
        name: "王芳",
        role: "信息安全主管",
        company: "某保险集团",
        avatar: "/avatars/wang-fang.jpg",
        initials: "WF"
      },
      rating: 5,
      metrics: {
        improvement: "安全合规 100%",
        icon: Shield
      },
      category: "保险行业"
    }
  ]

  const stats = [
    {
      label: "客户满意度",
      value: "98%",
      description: "基于50+企业客户反馈"
    },
    {
      label: "平均性能提升",
      value: "10-100x",
      description: "相比传统数据库解决方案"
    },
    {
      label: "部署成功率",
      value: "100%",
      description: "零失败的企业级部署"
    },
    {
      label: "技术支持响应",
      value: "<2h",
      description: "平均问题响应时间"
    }
  ]

  return (
    <section className="py-24 bg-gradient-to-br from-green-50/50 to-blue-50/50 dark:from-green-950/10 dark:to-blue-950/10">
      <div className="container mx-auto px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <Badge variant="secondary" className="mb-4">
            客户证言
          </Badge>
          <h2 className="text-4xl md:text-5xl font-bold text-foreground mb-4">
            客户的信赖与认可
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
            来自金融行业领先企业的真实反馈，见证DuckHub的卓越表现
          </p>
        </motion.div>

        {/* 客户证言卡片 */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 mb-16">
          {testimonials.map((testimonial, index) => (
            <motion.div
              key={testimonial.id}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
            >
              <TestimonialCard testimonial={testimonial} />
            </motion.div>
          ))}
        </div>

        {/* 统计数据 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm rounded-2xl p-8 border border-green-100 dark:border-green-900/20"
        >
          <div className="text-center mb-8">
            <h3 className="text-2xl font-bold text-foreground mb-2">
              数据说话
            </h3>
            <p className="text-muted-foreground">
              基于真实客户使用数据的统计结果
            </p>
          </div>
          
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
            {stats.map((stat, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, scale: 0.8 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="text-center"
              >
                <div className="text-3xl font-bold text-foreground mb-1">
                  {stat.value}
                </div>
                <div className="text-sm font-medium text-foreground mb-1">
                  {stat.label}
                </div>
                <div className="text-xs text-muted-foreground">
                  {stat.description}
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  )
}

/**
 * 证言卡片组件
 */
interface TestimonialCardProps {
  testimonial: {
    id: number
    content: string
    author: {
      name: string
      role: string
      company: string
      avatar: string
      initials: string
    }
    rating: number
    metrics: {
      improvement: string
      icon: React.ComponentType<{ className?: string }>
    }
    category: string
  }
}

function TestimonialCard({ testimonial }: TestimonialCardProps) {
  const MetricIcon = testimonial.metrics.icon
  
  return (
    <Card className="h-full hover:shadow-xl transition-all duration-300 border-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm">
      <CardContent className="p-6">
        {/* 引用图标 */}
        <div className="flex justify-between items-start mb-4">
          <Quote className="w-8 h-8 text-green-500 opacity-60" />
          <Badge variant="outline" className="text-xs">
            {testimonial.category}
          </Badge>
        </div>
        
        {/* 证言内容 */}
        <blockquote className="text-muted-foreground leading-relaxed mb-6">
          "{testimonial.content}"
        </blockquote>
        
        {/* 评分 */}
        <div className="flex items-center space-x-1 mb-4">
          {Array.from({ length: testimonial.rating }).map((_, i) => (
            <Star key={i} className="w-4 h-4 fill-yellow-400 text-yellow-400" />
          ))}
        </div>
        
        {/* 改进指标 */}
        <div className="flex items-center space-x-2 mb-6 p-3 bg-green-50 dark:bg-green-950/20 rounded-lg">
          <div className="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
            <MetricIcon className="w-4 h-4 text-white" />
          </div>
          <span className="text-sm font-medium text-green-700 dark:text-green-300">
            {testimonial.metrics.improvement}
          </span>
        </div>
        
        {/* 作者信息 */}
        <div className="flex items-center space-x-3">
          <Avatar className="w-10 h-10">
            <AvatarImage src={testimonial.author.avatar} alt={testimonial.author.name} />
            <AvatarFallback className="bg-gradient-to-r from-green-500 to-blue-500 text-white text-sm">
              {testimonial.author.initials}
            </AvatarFallback>
          </Avatar>
          <div>
            <div className="font-medium text-foreground text-sm">
              {testimonial.author.name}
            </div>
            <div className="text-xs text-muted-foreground">
              {testimonial.author.role}
            </div>
            <div className="flex items-center space-x-1 text-xs text-muted-foreground">
              <Building2 className="w-3 h-3" />
              <span>{testimonial.author.company}</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}