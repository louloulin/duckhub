"use client"

import { motion } from 'framer-motion'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { LucideIcon, CheckCircle } from 'lucide-react'
import { cn } from '@/lib/utils'

interface Feature {
  icon: LucideIcon
  title: string
  description: string
  details: string[]
}

interface FeatureDetailsProps {
  title: string
  subtitle: string
  features: Feature[]
}

/**
 * 功能详情组件
 * 展示功能特性的详细信息和特点
 */
export function FeatureDetails({ title, subtitle, features }: FeatureDetailsProps) {
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
            {title}
          </h2>
          <p className="text-xl text-gray-600 max-w-3xl mx-auto">
            {subtitle}
          </p>
        </motion.div>

        <div className="grid lg:grid-cols-2 gap-12">
          {features.map((feature, index) => {
            const IconComponent = feature.icon
            const isEven = index % 2 === 0
            
            return (
              <motion.div
                key={index}
                initial={{ opacity: 0, x: isEven ? -30 : 30 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.2 }}
              >
                <Card className="h-full hover:shadow-lg transition-all duration-300 border-0 bg-gradient-to-br from-gray-50 to-white">
                  <CardContent className="p-8">
                    <div className="flex items-start space-x-6">
                      {/* 图标 */}
                      <div className="flex-shrink-0">
                        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-blue-500 to-purple-500 flex items-center justify-center shadow-lg">
                          <IconComponent className="w-8 h-8 text-white" />
                        </div>
                      </div>
                      
                      {/* 内容 */}
                      <div className="flex-1">
                        <h3 className="text-2xl font-bold text-gray-900 mb-3">
                          {feature.title}
                        </h3>
                        <p className="text-gray-600 mb-6 leading-relaxed">
                          {feature.description}
                        </p>
                        
                        {/* 特性列表 */}
                        <div className="space-y-3">
                          {feature.details.map((detail, detailIndex) => {
                            const [title, description] = detail.split(' - ')
                            return (
                              <motion.div
                                key={detailIndex}
                                initial={{ opacity: 0, x: -10 }}
                                whileInView={{ opacity: 1, x: 0 }}
                                viewport={{ once: true }}
                                transition={{ delay: (index * 0.2) + (detailIndex * 0.1) }}
                                className="flex items-start space-x-3"
                              >
                                <CheckCircle className="w-5 h-5 text-green-500 flex-shrink-0 mt-0.5" />
                                <div>
                                  <span className="font-semibold text-gray-900">
                                    {title}
                                  </span>
                                  {description && (
                                    <span className="text-gray-600 ml-1">
                                      - {description}
                                    </span>
                                  )}
                                </div>
                              </motion.div>
                            )
                          })}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </motion.div>
            )
          })}
        </div>
        
        {/* 底部CTA */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.6 }}
          className="text-center mt-16"
        >
          <div className="bg-gradient-to-r from-blue-50 to-purple-50 rounded-2xl p-8">
            <h3 className="text-2xl font-bold text-gray-900 mb-4">
              准备好体验这些强大功能了吗？
            </h3>
            <p className="text-gray-600 mb-6 max-w-2xl mx-auto">
              立即开始使用DuckHub，体验企业级数据平台的强大能力
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Badge 
                variant="secondary" 
                className="px-6 py-2 text-sm bg-white/80 hover:bg-white transition-colors cursor-pointer"
              >
                🚀 快速部署
              </Badge>
              <Badge 
                variant="secondary" 
                className="px-6 py-2 text-sm bg-white/80 hover:bg-white transition-colors cursor-pointer"
              >
                📚 完整文档
              </Badge>
              <Badge 
                variant="secondary" 
                className="px-6 py-2 text-sm bg-white/80 hover:bg-white transition-colors cursor-pointer"
              >
                🛠️ 技术支持
              </Badge>
            </div>
          </div>
        </motion.div>
      </div>