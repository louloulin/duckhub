import { DocsHeader } from '@/components/docs/docs-header'
import { DocsSidebar } from '@/components/docs/docs-sidebar'
import { DocsFooter } from '@/components/docs/docs-footer'

/**
 * 文档页面布局组件
 * 包含文档专用的头部、侧边栏和底部
 */
export default function DocsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="min-h-screen bg-white">
      <DocsHeader />
      
      <div className="flex">
        <DocsSidebar />
        
        <main className="flex-1 min-w-0">
          <div className="max-w-4xl mx-auto px-6 py-8">
            {children}
          </div>
        </main>
      </div>
      
      <DocsFooter />
    </div>
  )
}