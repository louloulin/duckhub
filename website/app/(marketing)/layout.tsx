import { Header } from '@/components/common/header'
import { Footer } from '@/components/common/footer'

/**
 * 营销页面布局组件
 * 包含头部导航和底部信息，用于所有营销相关页面
 */
export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <>
      <Header />
      <main className="flex-1">{children}</main>
      <Footer />
    </>
  )
}