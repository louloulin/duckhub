"use client"

import { useEffect, useState } from 'react'
import { motion, useInView } from 'framer-motion'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { TrendingUp, Clock, Shield, Zap, Users, Database, BarChart3, CheckCircle } from 'lucide-react'
import { useRef } from 'react'
import { cn } from '@/lib/utils'

/**
 * 统计数据展示组件
 * 展示DuckHub平台的关键性能指标和统计数据
 */
export function Stats() {
  const stats = [
    {
      icon: TrendingUp,
      label: "性能提升",
      value: "10-100x",
      description: "相比传统数据库",
      gradient: "from-green-500 to-emerald-500",
      bgColor: "bg-green-50 dark:bg-green-950/20"
    },
    {
      icon: Clock,
      label: "响应时间",
      value: "<100ms",
      description: "平均查询响应",
      gradient: "from-blue-500 to-cyan-500",
      bgColor: "bg-blue-50 dark:bg-blue-950/20"
    },
    {
      icon: Shield,
      label: "系统可用性",
      value: "99.9%",
      description: "企业级SLA保证",
      gradient: "from-purple-500 to-pink-500",
      bgColor: "bg-purple-50 dark:bg-purple-950/20"
    },
    {
      icon: Zap,
      label: "并发查询",
      value: "1000+",
      description: "QPS处理能力",
      gradient: "from-orange-500 to-red-500",
      bgColor: "bg-orange-50 dark:bg-orange-950/20"
    }
  ]

  const achievements = [
    {
      icon: Users,
      title: "企业客户",
      value: "50+",
      description: "金融机构信赖选择"
    },
    {
      icon: Database,
      title: "数据处理",
      value: "10PB+",
      description: "累计处理数据量"
    },
    {
      icon: BarChart3,
      title: "查询优化",
      value: "85%",
      description: "平均性能提升"
    },
    {
      icon: CheckCircle,
      title: "测试覆盖",
      value: "86+",
      description: "通过测试用例"
    }
  ]

  return (
    <section className="py-24 bg-muted/30">
      <div className="container mx-auto px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <Badge variant="secondary" className="mb-4">
            性能数据
          </Badge>
          <h2 className="text-4xl md:text-5xl font-bold text-foreground mb-4">
            值得信赖的性能表现
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
            经过严格测试验证的企业级性能指标，为您的业务提供可靠保障
          </p>
        </motion.div>

        {/* 主要性能指标 */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 mb-16">
          {stats.map((stat, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
            >
              <StatCard {...stat} />
            </motion.div>
          ))}
        </div>

        {/* 成就数据 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="bg-gradient-to-r from-green-50 to-blue-50 dark:from-green-950/10 dark:to-blue-950/10 rounded-2xl p-8 border border-green-100 dark:border-green-900/20"
        >
          <div className="text-center mb-8">
            <h3 className="text-2xl font-bold text-foreground mb-2">
              生产环境验证
            </h3>
            <p className="text-muted-foreground">
              在真实业务场景中经过充分验证的可靠数据
            </p>
          </div>
          
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
            {achievements.map((achievement, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, scale: 0.8 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.1 }}
                className="text-center"
              >
                <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-gradient-to-r from-green-500 to-blue-500 flex items-center justify-center">
                  <achievement.icon className="w-6 h-6 text-white" />
                </div>
                <div className="text-2xl font-bold text-foreground mb-1">
                  {achievement.value}
                </div>
                <div className="text-sm font-medium text-foreground mb-1">
                  {achievement.title}
                </div>
                <div className="text-xs text-muted-foreground">
                  {achievement.description}
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
 * 统计卡片组件
 */
interface StatCardProps {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: string
  description: string
  gradient: string
  bgColor: string
}

function StatCard({ icon: Icon, label, value, description, gradient, bgColor }: StatCardProps) {
  const [displayValue, setDisplayValue] = useState('0')
  const [isVisible, setIsVisible] = useState(false)
  const ref = useRef(null)
  const inView = useInView(ref, { once: true })

  useEffect(() => {
    if (inView && !isVisible) {
      setIsVisible(true)
      
      // 数字动画效果
      if (value.includes('x')) {
        // 处理倍数 (如 10-100x)
        const match = value.match(/(\d+)-(\d+)x/)
        if (match) {
          const start = parseInt(match[1])
          const end = parseInt(match[2])
          animateNumber(start, end, (current) => `${current}-${end}x`)
        }
      } else if (value.includes('%')) {
        // 处理百分比 (如 99.9%)
        const num = parseFloat(value.replace('%', ''))
        animateNumber(0, num, (current) => `${current.toFixed(1)}%`)
      } else if (value.includes('ms')) {
        // 处理毫秒 (如 <100ms)
        const num = parseInt(value.replace(/[<>ms]/g, ''))
        animateNumber(0, num, (current) => `<${Math.round(current)}ms`)
      } else if (value.includes('+')) {
        // 处理加号 (如 1000+)
        const num = parseInt(value.replace('+', ''))
        animateNumber(0, num, (current) => `${Math.round(current)}+`)
      } else {
        setDisplayValue(value)
      }
    }
  }, [inView, isVisible, value])

  const animateNumber = (start: number, end: number, formatter: (num: number) => string) => {
    const duration = 2000
    const startTime = Date.now()
    
    const animate = () => {
      const elapsed = Date.now() - startTime
      const progress = Math.min(elapsed / duration, 1)
      
      // 使用缓动函数
      const easeOutQuart = 1 - Math.pow(1 - progress, 4)
      const current = start + (end - start) * easeOutQuart
      
      setDisplayValue(formatter(current))
      
      if (progress < 1) {
        requestAnimationFrame(animate)
      }
    }
    
    animate()
  }

  return (
    <Card ref={ref} className={cn("h-full hover:shadow-lg transition-all duration-300 border-0", bgColor)}>
      <CardContent className="p-6 text-center">
        <div className={cn(
          "w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center shadow-lg",
          `bg-gradient-to-r ${gradient}`
        )}>
          <Icon className="w-6 h-6 text-white" />
        </div>
        
        <div className="text-3xl font-bold text-foreground mb-2">
          {displayValue}
        </div>
        
        <div className="text-sm font-medium text-foreground mb-1">
          {label}
        </div>
        
        <div className="text-xs text-muted-foreground">
          {description}
        </div>
      </CardContent>
    </Card>
  )
}