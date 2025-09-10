"use client"

import { motion } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { LucideIcon } from 'lucide-react'
import { cn } from '@/lib/utils'

interface Benefit {
  icon: LucideIcon
  title: string
  description: string
}

interface FeatureHeroProps {
  title: string
  subtitle: string
  description: string
  gradient: string
  icon: LucideIcon
  benefits: Benefit[]
}

/**
 * 功能页面Hero组件
 * 用于展示功能特性的主要信息和价值主张
 */
export function Hero({
  title,
  subtitle,
  description,
  gradient,
  icon: IconComponent,
  benefits
}: FeatureHeroProps) {
  return (
    <section className="relative min-h-[80vh] flex items-center justify-center overflow-hidden">
      {/* 背景渐变 */}
      <div className={cn(
        "absolute inset-0 bg-gradient-to-br opacity-10",
        gradient.replace('from-', 'from-').replace('to-', 'to-')
      )} />
      
      {/* 网格背景 */}
      <div className="absolute inset-0 bg-grid-slate-100 [mask-image:linear-gradient(0deg,white,rgba(255,255,255,0.6))] dark:bg-grid-slate-700/25 dark:[mask-image:linear-gradient(0deg,rgba(255,255,255,0.1),rgba(255,255,255,0.5))]" />
      
      <div className="relative z-10 container mx-auto px-4">
        <div className="max-w-6xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            className="text-center mb-16"
          >
            {/* 图标 */}
            <motion.div
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.2 }}
              className="mb-8"
            >
              <div className={cn(
                "w-20 h-20 rounded-2xl flex items-center justify-center mx-auto shadow-lg bg-gradient-to-br",
                gradient
              )}>
                <IconComponent className="w-10 h-10 text-white" />
              </div>
            </motion.div>
            
            {/* 副标题 */}
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
              className="mb-4"
            >
              <Badge variant="secondary" className="px-4 py-2 text-sm font-medium">
                {subtitle}
              </Badge>
            </motion.div>
            
            {/* 主标题 */}
            <motion.h1 
              className="text-4xl md:text-6xl font-bold mb-6 leading-tight"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4 }}
            >
              <span className={cn(
                "bg-gradient-to-r bg-clip-text text-transparent",
                gradient
              )}>
                {title}
              </span>
            </motion.h1>
            
            {/* 描述 */}
            <motion.p 
              className="text-xl md:text-2xl text-muted-foreground mb-12 max-w-4xl mx-auto leading-relaxed"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5 }}
            >
              {description}
            </motion.p>

            {/* CTA按钮 */}
            <motion.div 
              className="flex flex-col sm:flex-row gap-4 justify-center mb-16"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.6 }}
            >
              <Button 
                size="lg" 
                className={cn(
                  "px-8 py-4 text-lg bg-gradient-to-r text-white shadow-lg hover:shadow-xl transition-all",
                  gradient
                )}
              >
                免费试用
              </Button>
              <Button 
                size="lg" 
                variant="outline" 
                className="px-8 py-4 text-lg border-2 hover:bg-accent"
              >
                查看演示
              </Button>
            </motion.div>
          </motion.div>

          {/* 核心优势 */}
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7 }}
            className="grid md:grid-cols-3 gap-8"
          >
            {benefits.map((benefit, index) => {
              const BenefitIcon = benefit.icon
              return (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.8 + index * 0.1 }}
                  className="text-center"
                >
                  <div className="bg-white/80 backdrop-blur-sm rounded-2xl p-8 shadow-lg hover:shadow-xl transition-all duration-300 border border-white/20">
                    <div className={cn(
                      "w-12 h-12 rounded-xl flex items-center justify-center mx-auto mb-4 bg-gradient-to-br",
                      gradient
                    )}>
                      <BenefitIcon className="w-6 h-6 text-white" />
                    </div>
                    <h3 className="text-lg font-semibold text-gray-900 mb-2">
                      {benefit.title}
                    </h3>
                    <p className="text-sm text-gray-600 leading-relaxed">
                      {benefit.description}
                    </p>
                  </div>
                </motion.div>
              )
            })}
          </motion.div>
        </div>
      </div>
    </section>
  )
}