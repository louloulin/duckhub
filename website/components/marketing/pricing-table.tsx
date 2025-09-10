"use client"

import { useState } from 'react'
import { motion } from 'framer-motion'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Check, X, Star, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'

interface PricingPlan {
  name: string
  price: string
  period: string
  description: string
  popular: boolean
  features: string[]
  limitations: string[]
  ctaText: string
  ctaHref: string
}

interface ComparisonFeature {
  name: string
  free: boolean | string
  pro: boolean | string
  enterprise: boolean | string
}

interface ComparisonCategory {
  category: string
  features: ComparisonFeature[]
}

interface PricingTableProps {
  plans: PricingPlan[]
  comparisonFeatures: ComparisonCategory[]
}

/**
 * 定价表格组件
 * 展示定价方案和功能对比
 */
export function PricingTable({ plans, comparisonFeatures }: PricingTableProps) {
  const [billingCycle, setBillingCycle] = useState<'monthly' | 'yearly'>('monthly')

  const renderFeatureValue = (value: boolean | string) => {
    if (typeof value === 'boolean') {
      return value ? (
        <Check className="w-5 h-5 text-green-500 mx-auto" />
      ) : (
        <X className="w-5 h-5 text-gray-300 mx-auto" />
      )
    }
    return (
      <span className="text-sm text-gray-700 text-center block">
        {value}
      </span>
    )
  }

  return (
    <section className="py-24 bg-white">
      <div className="container mx-auto px-4">
        {/* 定价卡片 */}
        <div className="mb-20">
          {/* 计费周期切换 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="flex justify-center mb-12"
          >
            <div className="bg-gray-100 rounded-full p-1">
              <button
                onClick={() => setBillingCycle('monthly')}
                className={cn(
                  "px-6 py-2 rounded-full text-sm font-medium transition-all",
                  billingCycle === 'monthly'
                    ? "bg-white text-gray-900 shadow-sm"
                    : "text-gray-600 hover:text-gray-900"
                )}
              >
                按月付费
              </button>
              <button
                onClick={() => setBillingCycle('yearly')}
                className={cn(
                  "px-6 py-2 rounded-full text-sm font-medium transition-all relative",
                  billingCycle === 'yearly'
                    ? "bg-white text-gray-900 shadow-sm"
                    : "text-gray-600 hover:text-gray-900"
                )}
              >
                按年付费
                <Badge className="absolute -top-2 -right-2 bg-green-500 text-white text-xs px-2 py-0.5">
                  省20%
                </Badge>
              </button>
            </div>
          </motion.div>

          {/* 定价卡片网格 */}
          <div className="grid lg:grid-cols-3 gap-8 max-w-7xl mx-auto">
            {plans.map((plan, index) => {
              const adjustedPrice = plan.price !== '免费' && plan.price !== '联系销售' && billingCycle === 'yearly'
                ? `¥${Math.round(parseInt(plan.price.replace('¥', '')) * 0.8)}`
                : plan.price
              
              return (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: index * 0.1 }}
                  className={cn(
                    "relative",
                    plan.popular && "lg:scale-105"
                  )}
                >
                  {plan.popular && (
                    <div className="absolute -top-4 left-1/2 transform -translate-x-1/2">
                      <Badge className="bg-gradient-to-r from-green-500 to-blue-500 text-white px-4 py-1">
                        <Star className="w-3 h-3 mr-1" />
                        最受欢迎
                      </Badge>
                    </div>
                  )}
                  
                  <Card className={cn(
                    "h-full transition-all duration-300 hover:shadow-xl",
                    plan.popular 
                      ? "border-2 border-blue-200 shadow-lg bg-gradient-to-br from-blue-50 to-purple-50" 
                      : "border border-gray-200 hover:border-gray-300"
                  )}>
                    <CardHeader className="text-center pb-8">
                      <h3 className="text-2xl font-bold text-gray-900 mb-2">
                        {plan.name}
                      </h3>
                      <div className="mb-4">
                        <span className="text-4xl font-bold text-gray-900">
                          {adjustedPrice}
                        </span>
                        {plan.price !== '免费' && plan.price !== '联系销售' && (
                          <span className="text-gray-600 ml-2">
                            /{billingCycle === 'monthly' ? '月' : '年'}
                          </span>
                        )}
                      </div>
                      <p className="text-gray-600">
                        {plan.description}
                      </p>
                    </CardHeader>
                    
                    <CardContent className="pt-0">
                      {/* 功能列表 */}
                      <div className="mb-8">
                        <h4 className="font-semibold text-gray-900 mb-4 flex items-center">
                          <Zap className="w-4 h-4 mr-2 text-green-500" />
                          包含功能
                        </h4>
                        <ul className="space-y-3">
                          {plan.features.map((feature, featureIndex) => (
                            <li key={featureIndex} className="flex items-start space-x-3">
                              <Check className="w-5 h-5 text-green-500 flex-shrink-0 mt-0.5" />
                              <span className="text-sm text-gray-700">
                                {feature}
                              </span>
                            </li>
                          ))}
                        </ul>
                      </div>
                      
                      {/* 限制说明 */}
                      {plan.limitations.length > 0 && (
                        <div className="mb-8">
                          <h4 className="font-semibold text-gray-900 mb-4 flex items-center">
                            <X className="w-4 h-4 mr-2 text-gray-400" />
                            不包含
                          </h4>
                          <ul className="space-y-2">
                            {plan.limitations.map((limitation, limitIndex) => (
                              <li key={limitIndex} className="flex items-start space-x-3">
                                <X className="w-4 h-4 text-gray-400 flex-shrink-0 mt-0.5" />
                                <span className="text-sm text-gray-500">
                                  {limitation}
                                </span>
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}
                      
                      {/* CTA按钮 */}
                      <Button 
                        className={cn(
                          "w-full py-3 text-base font-medium",
                          plan.popular
                            ? "bg-gradient-to-r from-green-600 to-blue-600 hover:from-green-700 hover:to-blue-700 text-white shadow-lg"
                            : "bg-gray-900 hover:bg-gray-800 text-white"
                        )}
                        asChild
                      >
                        <a href={plan.ctaHref}>
                          {plan.ctaText}
                        </a>
                      </Button>
                    </CardContent>
                  </Card>
                </motion.div>
              )
            })}
          </div>
        </div>

        {/* 功能对比表 */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="max-w-6xl mx-auto"
        >
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">
              详细功能对比
            </h2>
            <p className="text-xl text-gray-600">
              了解不同版本的具体功能差异
            </p>
          </div>

          <div className="bg-white rounded-2xl shadow-lg overflow-hidden border border-gray-200">
            {/* 表头 */}
            <div className="bg-gray-50 px-6 py-4">
              <div className="grid grid-cols-4 gap-4">
                <div className="font-semibold text-gray-900">功能</div>
                <div className="text-center font-semibold text-gray-900">开源版</div>
                <div className="text-center font-semibold text-gray-900">专业版</div>
                <div className="text-center font-semibold text-gray-900">企业版</div>
              </div>
            </div>

            {/* 功能分类 */}
            {comparisonFeatures.map((category, categoryIndex) => (
              <div key={categoryIndex}>
                {/* 分类标题 */}
                <div className="bg-blue-50 px-6 py-3 border-t border-gray-200">
                  <h3 className="font-semibold text-blue-900">
                    {category.category}
                  </h3>
                </div>
                
                {/* 功能列表 */}
                {category.features.map((feature, featureIndex) => (
                  <div 
                    key={featureIndex}
                    className={cn(
                      "px-6 py-4 border-t border-gray-100",
                      featureIndex % 2 === 0 ? "bg-white" : "bg-gray-50/50"
                    )}
                  >
                    <div className="grid grid-cols-4 gap-4 items-center">
                      <div className="font-medium text-gray-900">
                        {feature.name}
                      </div>
                      <div className="text-center">
                        {renderFeatureValue(feature.free)}
                      </div>
                      <div className="text-center">
                        {renderFeatureValue(feature.pro)}
                      </div>
                      <div className="text-center">
                        {renderFeatureValue(feature.enterprise)}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  )
}