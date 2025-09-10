"use client"

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { motion } from 'framer-motion'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Play, Download, ChevronDown, Database, Zap, Bot, Shield, ArrowRight } from 'lucide-react'
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
      <div className="absolute inset-0 bg-gradient-to-br from-green-50 via-blue-50 to-purple-50 dark:from-green-950/20 dark:via-blue-950/20 dark:to-purple-950/20" />
      
      {/* 动态背景元素 */}
      <div className="absolute inset-0 overflow-hidden">
        <div 
          className="absolute w-96 h-96 bg-green-200 dark:bg-green-800/30 rounded-full mix-blend-multiply dark:mix-blend-screen filter blur-xl opacity-70 animate-blob"
          style={{
            left: `${20 + mousePosition.x * 0.02}%`,
            top: `${20 + mousePosition.y * 0.02}%`,
          }}
        />
        <div 
          className="absolute w-96 h-96 bg-blue-200 dark:bg-blue-800/30 rounded-full mix-blend-multiply dark:mix-blend-screen filter blur-xl opacity-70 animate-blob animation-delay-2000"
          style={{
            right: `${20 - mousePosition.x * 0.02}%`,
            top: `${30 + mousePosition.y * 0.01}%`,
          }}
        />
        <div 
          className="absolute w-96 h-96 bg-purple-200 dark:bg-purple-800/30 rounded-full mix-blend-multiply dark:mix-blend-screen filter blur-xl opacity-70 animate-blob animation-delay-4000"
          style={{
            bottom: `${20 - mousePosition.y * 0.02}%`,
            left: `${30 + mousePosition.x * 0.01}%`,
          }}
        />
      </div>

      {/* 网格背景 */}
      <div className="absolute inset-0 bg-grid-slate-100 dark:bg-grid-slate-700 [mask-image:linear-gradient(0deg,white,rgba(255,255,255,0.6))] dark:[mask-image:linear-gradient(0deg,rgba(255,255,255,0.1),rgba(255,255,255,0.5))]" />

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
            <Badge variant="secondary" className="px-4 py-2 text-sm font-medium bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-green-200 dark:border-green-800">
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
            <Button 
              size="lg" 
              className="bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700 text-white px-8 py-4 text-lg shadow-lg hover:shadow-xl transition-all duration-300"
              asChild
            >
              <Link href="/demo" className="flex items-center space-x-2">
                <Play className="w-5 h-5" />
                <span>观看演示</span>
              </Link>
            </Button>
            <Button 
              size="lg" 
              variant="outline" 
              className="px-8 py-4 text-lg border-2 hover:bg-accent hover:text-accent-foreground transition-all duration-300"
              asChild
            >
              <Link href="/docs/getting-started" className="flex items-center space-x-2">
                <Download className="w-5 h-5" />
                <span>开始使用</span>
                <ArrowRight className="w-4 h-4" />
              </Link>
            </Button>
          </motion.div>

          {/* 核心功能标签 */}
          <motion.div 
            className="flex flex-wrap justify-center gap-6 mb-16"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.6 }}
          >
            {features.map((feature, index) => {
              const Icon = feature.icon
              return (
                <motion.div
                  key={feature.label}
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: 0.7 + index * 0.1 }}
                  className="flex items-center space-x-2 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-full px-4 py-2 border border-gray-200 dark:border-gray-700 hover:shadow-md transition-all duration-300"
                >
                  <div className="w-6 h-6 rounded-full bg-gradient-to-r from-green-500 to-blue-500 flex items-center justify-center">
                    <Icon className="w-3 h-3 text-white" />
                  </div>
                  <span className="text-sm font-medium text-foreground">{feature.label}</span>
                </motion.div>
              )
            })}
          </motion.div>

          {/* 技术标签 */}
          <motion.div 
            className="flex flex-wrap justify-center gap-3 mb-12"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.8 }}
          >
            {['DuckDB', 'DuckLake', 'Rust', 'React', 'AI驱动', '企业级', 'ACID事务', '时间旅行'].map((tag, index) => (
              <motion.div
                key={tag}
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.9 + index * 0.05 }}
              >
                <Badge 
                  variant="secondary" 
                  className="px-3 py-1 text-sm bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-gray-200 dark:border-gray-700 hover:bg-white/80 dark:hover:bg-gray-800/80 transition-all duration-300"
                >
                  {tag}
                </Badge>
              </motion.div>
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
        <div className="flex flex-col items-center space-y-2">
          <span className="text-xs text-muted-foreground">向下滚动</span>
          <ChevronDown className="w-6 h-6 text-muted-foreground" />
        </div>
      </motion.div>
    </section>
  )
}