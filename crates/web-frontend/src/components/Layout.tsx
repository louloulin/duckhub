import { useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { CommandPalette } from '@/components/layout/CommandPalette'
import {
  BarChart3,
  Database,
  Bot,
  Settings,
  Menu,
  X,
  Home,
  Search,
  ChevronLeft,
  ChevronRight,
  Bell,
  User,
  Layers,
} from 'lucide-react'

interface LayoutProps {
  children: React.ReactNode
}

const navigation = [
  { name: '仪表板', href: '/', icon: Home },
  { name: '查询分析', href: '/query-analytics', icon: BarChart3 },
  { name: '数据探索', href: '/data-explorer', icon: Database },
  { name: 'DuckLake管理', href: '/ducklake-manager', icon: Layers },
  { name: 'AI助手', href: '/ai-agent', icon: Bot },
  { name: '设置', href: '/settings', icon: Settings },
]

export default function Layout({ children }: LayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const location = useLocation()

  // 键盘快捷键
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl/Cmd + B 切换侧边栏
      if ((event.ctrlKey || event.metaKey) && event.key === 'b') {
        event.preventDefault()
        setSidebarCollapsed(!sidebarCollapsed)
      }

      // Ctrl/Cmd + K 打开命令面板
      if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
        event.preventDefault()
        setCommandPaletteOpen(true)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [sidebarCollapsed])

  return (
    <div className="min-h-screen bg-background">
      {/* 命令面板 */}
      <CommandPalette
        open={commandPaletteOpen}
        onOpenChange={setCommandPaletteOpen}
      />

      {/* 移动端侧边栏 */}
      <div className={cn(
        "fixed inset-0 z-50 lg:hidden",
        sidebarOpen ? "block" : "hidden"
      )}>
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm" onClick={() => setSidebarOpen(false)} />
        <div className="fixed left-0 top-0 h-full w-64 bg-card border-r border-border shadow-xl">
          <div className="flex h-16 items-center justify-between px-4 border-b border-border">
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 supabase-gradient rounded-lg flex items-center justify-center shadow-sm">
                <Database className="h-4 w-4 text-white" />
              </div>
              <h1 className="text-xl font-bold gradient-text">DuckHub</h1>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSidebarOpen(false)}
              className="hover:bg-accent"
            >
              <X className="h-5 w-5" />
            </Button>
          </div>
          <nav className="px-3 py-4">
            <ul className="space-y-1">
              {navigation.map((item) => {
                const Icon = item.icon
                const isActive = location.pathname === item.href
                return (
                  <li key={item.name}>
                    <Link
                      to={item.href}
                      className={cn(
                        "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 relative group",
                        isActive
                          ? "bg-primary/10 text-primary shadow-sm"
                          : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                      )}
                      onClick={() => setSidebarOpen(false)}
                    >
                      {isActive && (
                        <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary rounded-r-full" />
                      )}
                      <Icon className="h-4 w-4 shrink-0" />
                      {item.name}
                    </Link>
                  </li>
                )
              })}
            </ul>
          </nav>
        </div>
      </div>

      {/* 桌面端侧边栏 */}
      <div className={cn(
        "hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:flex-col transition-all duration-300",
        sidebarCollapsed ? "lg:w-16" : "lg:w-64"
      )}>
        <div className="flex grow flex-col overflow-y-auto bg-card border-r border-border shadow-lg">
          {/* 侧边栏头部 */}
          <div className="flex h-16 shrink-0 items-center justify-between px-4 border-b border-border">
            <div className={cn(
              "flex items-center gap-2 transition-all duration-300",
              sidebarCollapsed ? "justify-center" : ""
            )}>
              <div className="w-8 h-8 supabase-gradient rounded-lg flex items-center justify-center shadow-sm">
                <Database className="h-4 w-4 text-white" />
              </div>
              {!sidebarCollapsed && (
                <h1 className="text-xl font-bold gradient-text">
                  DuckHub
                </h1>
              )}
            </div>
            {!sidebarCollapsed && (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
                className="hover:bg-accent transition-colors group relative"
                title="收起侧边栏 (Ctrl+B)"
              >
                <ChevronLeft className="h-4 w-4" />

                {/* 快捷键提示 */}
                <div className="absolute -bottom-8 left-1/2 -translate-x-1/2 px-2 py-1 bg-popover text-popover-foreground text-xs rounded border shadow-md opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 whitespace-nowrap z-50">
                  Ctrl+B
                </div>
              </Button>
            )}
          </div>

          {/* 导航菜单 */}
          <nav className="flex flex-1 flex-col px-3 py-4">
            <ul className="space-y-1">
              {navigation.map((item) => {
                const Icon = item.icon
                const isActive = location.pathname === item.href
                return (
                  <li key={item.name}>
                    <Link
                      to={item.href}
                      className={cn(
                        "flex items-center rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 group relative",
                        sidebarCollapsed ? "justify-center" : "gap-x-3",
                        isActive
                          ? "bg-primary/10 text-primary shadow-sm"
                          : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                      )}
                      title={sidebarCollapsed ? item.name : undefined}
                    >
                      {isActive && !sidebarCollapsed && (
                        <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary rounded-r-full" />
                      )}
                      {isActive && sidebarCollapsed && (
                        <div className="absolute left-1 top-0 bottom-0 w-0.5 bg-primary rounded-full" />
                      )}
                      <Icon className="h-4 w-4 shrink-0" />
                      {!sidebarCollapsed && (
                        <span className="transition-opacity duration-200">
                          {item.name}
                        </span>
                      )}

                      {/* 悬浮提示 */}
                      {sidebarCollapsed && (
                        <div className="absolute left-full ml-3 px-3 py-2 bg-popover text-popover-foreground text-sm rounded-lg border shadow-md opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 whitespace-nowrap z-50">
                          {item.name}
                          <div className="absolute left-0 top-1/2 -translate-y-1/2 -translate-x-1 w-2 h-2 bg-popover border-l border-t rotate-45"></div>
                        </div>
                      )}
                    </Link>
                  </li>
                )
              })}
            </ul>

            {/* 折叠按钮 - 仅在折叠状态显示 */}
            {sidebarCollapsed && (
              <div className="mt-auto pt-4 border-t border-border">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setSidebarCollapsed(false)}
                  className="w-full hover:bg-accent group relative"
                  title="展开侧边栏 (Ctrl+B)"
                >
                  <ChevronRight className="h-4 w-4" />

                  {/* 悬浮提示 */}
                  <div className="absolute left-full ml-3 px-3 py-2 bg-popover text-popover-foreground text-sm rounded-lg border shadow-md opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 whitespace-nowrap z-50">
                    展开侧边栏
                    <div className="absolute left-0 top-1/2 -translate-y-1/2 -translate-x-1 w-2 h-2 bg-popover border-l border-t rotate-45"></div>
                  </div>
                </Button>
              </div>
            )}
          </nav>
        </div>
      </div>

      {/* 主内容区域 */}
      <div className={cn(
        "transition-all duration-300",
        sidebarCollapsed ? "lg:pl-16" : "lg:pl-64"
      )}>
        {/* 顶部导航栏 */}
        <div className="sticky top-0 z-40 border-b border-border bg-background/80 backdrop-blur-md shadow-sm">
          <div className="flex h-16 items-center gap-x-4 px-4 sm:gap-x-6 sm:px-6 lg:px-8">
            <Button
              variant="ghost"
              size="icon"
              className="lg:hidden hover:bg-accent"
              onClick={() => setSidebarOpen(true)}
            >
              <Menu className="h-5 w-5" />
            </Button>

            {/* 面包屑导航 */}
            <div className="hidden sm:flex items-center text-sm text-muted-foreground">
              <span>DuckHub</span>
              <span className="mx-2">/</span>
              <span className="text-foreground font-medium">
                {navigation.find(item => item.href === location.pathname)?.name || '仪表板'}
              </span>
            </div>

            <div className="flex flex-1 gap-x-4 self-stretch lg:gap-x-6">
              <div className="flex flex-1 items-center justify-end sm:justify-start">
                <div className="relative w-full max-w-lg">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground pointer-events-none" />
                  <button
                    onClick={() => setCommandPaletteOpen(true)}
                    className="w-full rounded-lg border border-input bg-background pl-10 pr-12 py-2.5 text-sm text-left text-muted-foreground hover:border-primary hover:bg-accent/50 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all duration-200"
                  >
                    搜索表、查询或数据...
                  </button>
                  <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none">
                    <kbd className="px-2 py-1 text-xs bg-muted rounded border border-border">
                      ⌘K
                    </kbd>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-x-2 lg:gap-x-4">
                {/* 系统状态指示器 */}
                <div className="hidden lg:flex items-center gap-2 px-3 py-1.5 rounded-full bg-success/10 text-success text-xs font-medium">
                  <div className="w-2 h-2 bg-success rounded-full animate-pulse"></div>
                  系统正常
                </div>

                {/* 通知按钮 */}
                <Button
                  variant="ghost"
                  size="icon"
                  className="hover:bg-accent relative"
                >
                  <Bell className="h-4 w-4" />
                  <div className="absolute -top-1 -right-1 w-3 h-3 bg-destructive rounded-full flex items-center justify-center">
                    <div className="w-1.5 h-1.5 bg-white rounded-full"></div>
                  </div>
                </Button>

                {/* 用户头像 */}
                <Button
                  variant="ghost"
                  size="icon"
                  className="hover:bg-accent rounded-full"
                >
                  <div className="w-8 h-8 supabase-gradient rounded-full flex items-center justify-center shadow-sm">
                    <User className="h-4 w-4 text-white" />
                  </div>
                </Button>
              </div>
            </div>
          </div>
        </div>

        {/* 页面内容 */}
        <main className="flex-1 overflow-auto">
          <div className="h-full">
            <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
              <div className="h-full">
                {children}
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  )
}
