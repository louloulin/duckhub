import { Hero } from '@/components/marketing/hero'
import { Features } from '@/components/marketing/features'
import { Stats } from '@/components/marketing/stats'
import { Testimonials } from '@/components/marketing/testimonials'
import { CTA } from '@/components/marketing/cta'
import { TechStack } from '@/components/marketing/tech-stack'
import { UseCases } from '@/components/marketing/use-cases'

/**
 * 首页组件
 * 展示DuckHub的核心价值主张和功能特性
 */
export default function HomePage() {
  return (
    <>
      {/* Hero区域 - 主要价值主张 */}
      <Hero />
      
      {/* 统计数据 - 展示平台优势 */}
      <Stats />
      
      {/* 核心功能特性 */}
      <Features />
      
      {/* 技术栈展示 */}
      <TechStack />
      
      {/* 使用场景 */}
      <UseCases />
      
      {/* 客户证言 */}
      <Testimonials />
      
      {/* 行动号召 */}
      <CTA />
    </>
  )
}