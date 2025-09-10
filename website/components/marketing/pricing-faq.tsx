"use client"

import { motion } from 'framer-motion'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { Card, CardContent } from '@/components/ui/card'
import { HelpCircle, MessageCircle, Mail, Phone } from 'lucide-react'

interface FAQ {
  question: string
  answer: string
}

interface PricingFAQProps {
  faqs: FAQ[]
}

const contactMethods = [
  {
    icon: MessageCircle,
    title: '在线聊天',
    description: '工作时间内即时响应',
    action: '开始聊天'
  },
  {
    icon: Mail,
    title: '邮件支持',
    description: 'sales@duckhub.com',
    action: '发送邮件'
  },
  {
    icon: Phone,
    title: '电话咨询',
    description: '+86 400-123-4567',
    action: '立即拨打'
  }
]

/**
 * 定价FAQ组件
 * 展示定价相关的常见问题和联系方式
 */
export function PricingFAQ({ faqs }: PricingFAQProps) {
  return (
    <section className="py-24 bg-gradient-to-br from-gray-50 to-white">
      <div className="container mx-auto px-4">
        <div className="max-w-4xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <div className="flex items-center justify-center mb-4">
              <HelpCircle className="w-8 h-8 text-blue-600 mr-3" />
              <h2 className="text-4xl font-bold text-gray-900">
                常见问题
              </h2>
            </div>
            <p className="text-xl text-gray-600 max-w-2xl mx-auto">
              关于定价和功能的常见问题解答
            </p>
          </motion.div>

          {/* FAQ列表 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="mb-16"
          >
            <Card className="border-0 shadow-lg">
              <CardContent className="p-0">
                <Accordion type="single" collapsible className="w-full">
                  {faqs.map((faq, index) => (
                    <AccordionItem 
                      key={index} 
                      value={`item-${index}`}
                      className="border-b border-gray-100 last:border-b-0"
                    >
                      <AccordionTrigger className="px-8 py-6 text-left hover:no-underline hover:bg-gray-50 transition-colors">
                        <span className="text-lg font-semibold text-gray-900 pr-4">
                          {faq.question}
                        </span>
                      </AccordionTrigger>
                      <AccordionContent className="px-8 pb-6">
                        <div className="text-gray-600 leading-relaxed">
                          {faq.answer}
                        </div>
                      </AccordionContent>
                    </AccordionItem>
                  ))}
                </Accordion>
              </CardContent>
            </Card>
          </motion.div>

          {/* 联系方式 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.4 }}
          >
            <div className="text-center mb-8">
              <h3 className="text-2xl font-bold text-gray-900 mb-4">
                还有其他问题？
              </h3>
              <p className="text-gray-600">
                我们的团队随时为您提供帮助
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-6">
              {contactMethods.map((method, index) => {
                const IconComponent = method.icon
                return (
                  <motion.div
                    key={index}
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ delay: 0.5 + index * 0.1 }}
                  >
                    <Card className="h-full hover:shadow-lg transition-all duration-300 cursor-pointer group">
                      <CardContent className="p-6 text-center">
                        <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-xl flex items-center justify-center mx-auto mb-4 group-hover:scale-110 transition-transform">
                          <IconComponent className="w-6 h-6 text-white" />
                        </div>
                        <h4 className="text-lg font-semibold text-gray-900 mb-2">
                          {method.title}
                        </h4>
                        <p className="text-gray-600 mb-4">
                          {method.description}
                        </p>
                        <div className="text-blue-600 font-medium group-hover:text-blue-700 transition-colors">
                          {method.action}
                        </div>
                      </CardContent>
                    </Card>
                  </motion.div>
                )
              })}
            </div>
          </motion.div>

          {/* 底部提示 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.6 }}
            className="text-center mt-12"
          >
            <div className="bg-blue-50 rounded-2xl p-8">
              <h4 className="text-xl font-semibold text-gray-900 mb-4">
                💡 专业建议
              </h4>
              <p className="text-gray-600 max-w-2xl mx-auto leading-relaxed">
                不确定选择哪个版本？我们建议从<strong>开源版</strong>开始体验基础功能，
                然后根据业务需求升级到<strong>专业版</strong>或<strong>企业版</strong>。
                所有版本都支持无缝数据迁移。
              </p>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  )
}