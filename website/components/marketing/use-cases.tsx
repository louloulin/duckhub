"use client"

import { motion } from 'framer-motion'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  TrendingUp, 
  Shield, 
  BarChart3, 
  Users, 
  Clock, 
  Database, 
  ArrowRight,
  CheckCircle,
  Building2,
  CreditCard,
  PieChart,
  Target
} from 'lucide-react'
import Link from 'next/link'
import { cn } from '@/lib/utils'

/**
 * 使用场景展示组件
 * 展示DuckHub在不同业务场景中的应用
 */
export function UseCases() {
  const useCases = [
    {
      id: 1,
      title: "实时风险监控",
      description: "金融机构风险管理的核心场景",
      icon: Shield,
      gradient: "from-red-500 to-orange-500",
      bgColor: "bg-red-50 dark:bg-red-950/20",
      industry: "银行业",
      challenge: "传统数据仓库无法满足实时风险监控的性能要求，数据更新延迟导致风险暴露",
      solution: "DuckHub的时间旅行查询和ACID事务确保数据一致性，高性能引擎实现毫秒级风险计算",
      benefits: [
        "风险计算延迟降低95%",
        "数据一致性100%保证",
        "支持复杂风险模型",
        "实时告警和通知"
      ],
      metrics: {
        performance: "<50ms",
        accuracy: "99.99%",
        availability: "99.9%",
        coverage: "全业务",
        conversion: "N/A",
        cost: "N/A",
        efficiency: "N/A",
        compliance: "N/A"
      }
    },
    {
      id: 2,
      title: "智能投研分析",
      description: "证券公司投资研究的智能化升级",
      icon: TrendingUp,
      gradient: "from-blue-500 to-cyan-500",
      bgColor: "bg-blue-50 dark:bg-blue-950/20",
      industry: "证券业",
      challenge: "投研分析师需要处理海量市场数据，传统工具效率低下，难以快速响应市场变化",
      solution: "AI智能助手支持自然语言查询，自动生成投研报告，DuckLake确保历史数据完整性",
      benefits: [
        "分析效率提升300%",
        "自动化报告生成",
        "历史数据回溯",
        "多维度数据关联"
      ],
      metrics: {
        performance: "10x",
        accuracy: "95%",
        coverage: "全市场",
        availability: "N/A",
        conversion: "N/A",
        cost: "N/A",
        efficiency: "N/A",
        compliance: "N/A"
      }
    },
    {
      id: 3,
      title: "客户行为分析",
      description: "保险公司精准营销和风控",
      icon: Users,
      gradient: "from-green-500 to-emerald-500",
      bgColor: "bg-green-50 dark:bg-green-950/20",
      industry: "保险业",
      challenge: "客户数据分散在多个系统，难以形成统一视图，营销效果和风控精度有限",
      solution: "统一数据湖架构整合多源数据，AI分析挖掘客户行为模式，支持个性化服务",
      benefits: [
        "客户转化率提升40%",
        "风险识别准确率90%",
        "营销成本降低30%",
        "客户满意度提升"
      ],
      metrics: {
        conversion: "+40%",
        accuracy: "90%",
        cost: "-30%",
        coverage: "全客户群",
        performance: "N/A",
        availability: "N/A",
        efficiency: "N/A",
        compliance: "N/A"
      }
    },
    {
      id: 4,
      title: "合规报告自动化",
      description: "金融机构监管合规的自动化解决方案",
      icon: BarChart3,
      gradient: "from-purple-500 to-pink-500",
      bgColor: "bg-purple-50 dark:bg-purple-950/20",
      industry: "监管合规",
      challenge: "监管报告制作耗时耗力，人工操作容易出错，难以满足日益严格的合规要求",
      solution: "自动化数据收集和报告生成，审计追踪确保合规性，支持多种监管标准",
      benefits: [
        "报告生成时间减少80%",
        "人工错误率降至0.1%",
        "支持多种监管标准",
        "完整审计追踪"
      ],
      metrics: {
        efficiency: "+400%",
        accuracy: "99.9%",
        compliance: "100%",
        performance: "N/A",
        availability: "N/A",
        coverage: "N/A",
        conversion: "N/A",
        cost: "N/A"
      }
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
            应用场景
          </Badge>
          <h2 className="text-4xl md:text-5xl font-bold text-foreground mb-4">
            真实业务场景
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
            了解DuckHub如何在不同金融业务场景中发挥价值，助力企业数字化转型
          </p>
        </motion.div>

        <div className="grid lg:grid-cols-2 gap-8">
          {useCases.map((useCase, index) => (
            <motion.div
              key={useCase.id}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
            >
              <UseCaseCard useCase={useCase} />
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
          <div className="bg-gradient-to-r from-green-50 to-blue-50 dark:from-green-950/10 dark:to-blue-950/10 rounded-2xl p-8 border border-green-100 dark:border-green-900/20">
            <h3 className="text-2xl font-bold text-foreground mb-4">
              您的业务场景是什么？
            </h3>
            <p className="text-muted-foreground mb-6 max-w-2xl mx-auto">
              我们的专家团队将为您量身定制解决方案，让DuckHub完美适配您的业务需求
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Button size="lg" asChild>
                <Link href="/contact" className="flex items-center space-x-2">
                  <span>咨询解决方案</span>
                  <ArrowRight className="w-4 h-4" />
                </Link>
              </Button>
              <Button size="lg" variant="outline" asChild>
                <Link href="/demo" className="flex items-center space-x-2">
                  <span>预约演示</span>
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
 * 使用场景卡片组件
 */
interface UseCaseCardProps {
  useCase: {
    id: number
    title: string
    description: string
    icon: React.ComponentType<{ className?: string }>
    gradient: string
    bgColor: string
    industry: string
    challenge: string
    solution: string
    benefits: string[]
    metrics: Record<string, string>
  }
}

function UseCaseCard({ useCase }: UseCaseCardProps) {
  const Icon = useCase.icon
  
  return (
    <Card className={cn("h-full hover:shadow-xl transition-all duration-300 border-0", useCase.bgColor)}>
      <CardHeader className="pb-4">
        <div className="flex items-start justify-between mb-4">
          <div className="flex items-center space-x-3">
            <div className={cn(
              "w-12 h-12 rounded-xl flex items-center justify-center shadow-lg",
              `bg-gradient-to-r ${useCase.gradient}`
            )}>
              <Icon className="w-6 h-6 text-white" />
            </div>
            <div>
              <CardTitle className="text-xl font-semibold text-foreground">
                {useCase.title}
              </CardTitle>
              <CardDescription className="text-muted-foreground">
                {useCase.description}
              </CardDescription>
            </div>
          </div>
          <Badge variant="outline" className="text-xs">
            {useCase.industry}
          </Badge>
        </div>
      </CardHeader>
      
      <CardContent className="pt-0 space-y-6">
        {/* 挑战 */}
        <div>
          <h4 className="text-sm font-semibold text-foreground mb-2 flex items-center space-x-2">
            <Target className="w-4 h-4 text-red-500" />
            <span>业务挑战</span>
          </h4>
          <p className="text-sm text-muted-foreground leading-relaxed">
            {useCase.challenge}
          </p>
        </div>
        
        {/* 解决方案 */}
        <div>
          <h4 className="text-sm font-semibold text-foreground mb-2 flex items-center space-x-2">
            <Database className="w-4 h-4 text-blue-500" />
            <span>DuckHub方案</span>
          </h4>
          <p className="text-sm text-muted-foreground leading-relaxed">
            {useCase.solution}
          </p>
        </div>
        
        {/* 业务价值 */}
        <div>
          <h4 className="text-sm font-semibold text-foreground mb-3 flex items-center space-x-2">
            <CheckCircle className="w-4 h-4 text-green-500" />
            <span>业务价值</span>
          </h4>
          <div className="grid grid-cols-1 gap-2">
            {useCase.benefits.map((benefit, index) => (
              <div key={index} className="flex items-center space-x-2">
                <div className="w-1.5 h-1.5 bg-green-500 rounded-full flex-shrink-0" />
                <span className="text-sm text-muted-foreground">{benefit}</span>
              </div>
            ))}
          </div>
        </div>
        
        {/* 关键指标 */}
        <div className="bg-white/50 dark:bg-gray-800/50 rounded-lg p-4">
          <h4 className="text-sm font-semibold text-foreground mb-3">关键指标</h4>
          <div className="grid grid-cols-3 gap-4">
            {Object.entries(useCase.metrics).map(([key, value]) => (
              <div key={key} className="text-center">
                <div className="text-lg font-bold text-foreground">{value}</div>
                <div className="text-xs text-muted-foreground capitalize">{key}</div>
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}