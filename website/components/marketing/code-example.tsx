"use client"

import { useState } from 'react'
import { motion } from 'framer-motion'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Copy, Check, Play, Code } from 'lucide-react'
import { cn } from '@/lib/utils'

interface CodeExample {
  title: string
  language: string
  code: string
}

interface CodeExampleProps {
  title: string
  subtitle: string
  examples: CodeExample[]
}

/**
 * 代码示例组件
 * 展示功能的代码示例和使用方法
 */
export function CodeExample({ title, subtitle, examples }: CodeExampleProps) {
  const [activeTab, setActiveTab] = useState(0)
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null)

  const copyToClipboard = async (code: string, index: number) => {
    try {
      await navigator.clipboard.writeText(code)
      setCopiedIndex(index)
      setTimeout(() => setCopiedIndex(null), 2000)
    } catch (err) {
      console.error('Failed to copy code:', err)
    }
  }

  const getLanguageColor = (language: string) => {
    const colors = {
      sql: 'from-blue-500 to-blue-600',
      javascript: 'from-yellow-500 to-orange-500',
      typescript: 'from-blue-600 to-blue-700',
      python: 'from-green-500 to-green-600',
      text: 'from-gray-500 to-gray-600',
      json: 'from-purple-500 to-purple-600'
    }
    return colors[language as keyof typeof colors] || 'from-gray-500 to-gray-600'
  }

  return (
    <section className="py-24 bg-gradient-to-br from-gray-50 to-white">
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

        <div className="max-w-6xl mx-auto">
          {/* 标签页导航 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="flex flex-wrap justify-center gap-2 mb-8"
          >
            {examples.map((example, index) => (
              <Button
                key={index}
                variant={activeTab === index ? "default" : "outline"}
                onClick={() => setActiveTab(index)}
                className={cn(
                  "px-6 py-3 text-sm font-medium transition-all",
                  activeTab === index
                    ? "bg-gradient-to-r from-blue-600 to-purple-600 text-white shadow-lg"
                    : "hover:bg-accent"
                )}
              >
                <Code className="w-4 h-4 mr-2" />
                {example.title}
              </Button>
            ))}
          </motion.div>

          {/* 代码展示区域 */}
          <motion.div
            key={activeTab}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
          >
            <Card className="overflow-hidden shadow-2xl border-0">
              <div className="bg-gradient-to-r from-gray-800 to-gray-900 px-6 py-4 flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="flex space-x-2">
                    <div className="w-3 h-3 rounded-full bg-red-500"></div>
                    <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                    <div className="w-3 h-3 rounded-full bg-green-500"></div>
                  </div>
                  <Badge 
                    className={cn(
                      "px-3 py-1 text-xs font-medium text-white bg-gradient-to-r",
                      getLanguageColor(examples[activeTab].language)
                    )}
                  >
                    {examples[activeTab].language.toUpperCase()}
                  </Badge>
                </div>
                
                <div className="flex items-center space-x-2">
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => copyToClipboard(examples[activeTab].code, activeTab)}
                    className="text-gray-300 hover:text-white hover:bg-gray-700"
                  >
                    {copiedIndex === activeTab ? (
                      <Check className="w-4 h-4" />
                    ) : (
                      <Copy className="w-4 h-4" />
                    )}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="text-gray-300 hover:text-white hover:bg-gray-700"
                  >
                    <Play className="w-4 h-4" />
                  </Button>
                </div>
              </div>
              
              <CardContent className="p-0">
                <div className="bg-gray-900 text-gray-100 p-6 overflow-x-auto">
                  <pre className="text-sm leading-relaxed">
                    <code className="language-{examples[activeTab].language}">
                      {examples[activeTab].code}
                    </code>
                  </pre>
                </div>
              </CardContent>
            </Card>
          </motion.div>

          {/* 代码说明 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.3 }}
            className="mt-8 text-center"
          >
            <div className="bg-blue-50 rounded-2xl p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                💡 代码说明
              </h3>
              <p className="text-gray-600 max-w-3xl mx-auto">
                以上示例展示了{examples[activeTab].title}的具体实现方式。
                您可以直接复制代码到您的项目中使用，或者根据实际需求进行调整。
              </p>
            </div>
          </motion.div>

          {/* 相关链接 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.4 }}
            className="flex flex-wrap justify-center gap-4 mt-8"
          >
            <Button variant="outline" className="px-6 py-2">
              📖 查看完整文档
            </Button>
            <Button variant="outline" className="px-6 py-2">
              🎮 在线试用
            </Button>
            <Button variant="outline" className="px-6 py-2">
              💬 技术支持
            </Button>
          </motion.div>
        </div>
      </div>
    </section>
  )
}