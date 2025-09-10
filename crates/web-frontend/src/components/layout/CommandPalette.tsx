import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command'
import {
  Home,
  BarChart3,
  Database,
  Layers,
  Bot,
  Settings,
  Search,
  FileText,
  Users,
  // Calendar, // 暂时未使用
  Bell,
  Zap,
} from 'lucide-react'

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

// 导航项配置
const navigationItems = [
  { 
    name: '仪表板', 
    href: '/', 
    icon: Home,
    description: '查看系统概览和关键指标',
    keywords: ['dashboard', 'home', '首页', '概览']
  },
  { 
    name: '查询分析', 
    href: '/query-analytics', 
    icon: BarChart3,
    description: '分析查询性能和优化建议',
    keywords: ['query', 'analytics', '分析', '性能', '查询']
  },
  { 
    name: '数据探索', 
    href: '/data-explorer', 
    icon: Database,
    description: '浏览和探索数据库表结构',
    keywords: ['data', 'explorer', '数据', '探索', '表']
  },
  { 
    name: 'DuckLake管理', 
    href: '/ducklake-manager', 
    icon: Layers,
    description: '管理DuckLake数据湖配置',
    keywords: ['ducklake', 'lake', '数据湖', '管理']
  },
  { 
    name: 'AI助手', 
    href: '/ai-agent', 
    icon: Bot,
    description: '智能查询助手和建议',
    keywords: ['ai', 'assistant', '助手', '智能', 'bot']
  },
  { 
    name: '设置', 
    href: '/settings', 
    icon: Settings,
    description: '系统配置和用户偏好',
    keywords: ['settings', 'config', '设置', '配置']
  },
]

// 快速操作配置
const quickActions = [
  {
    name: '新建查询',
    action: 'new-query',
    icon: FileText,
    description: '创建新的SQL查询',
    keywords: ['new', 'query', '新建', '查询', 'sql']
  },
  {
    name: '查看通知',
    action: 'notifications',
    icon: Bell,
    description: '查看系统通知和警报',
    keywords: ['notifications', 'alerts', '通知', '警报']
  },
  {
    name: '性能监控',
    action: 'performance',
    icon: Zap,
    description: '查看系统性能指标',
    keywords: ['performance', 'monitoring', '性能', '监控']
  },
  {
    name: '用户管理',
    action: 'users',
    icon: Users,
    description: '管理用户和权限',
    keywords: ['users', 'permissions', '用户', '权限']
  },
]

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const [searchQuery, setSearchQuery] = useState('')
  const navigate = useNavigate()

  // 处理导航
  const handleNavigate = (href: string) => {
    navigate(href)
    onOpenChange(false)
    setSearchQuery('')
  }

  // 处理快速操作
  const handleAction = (action: string) => {
    switch (action) {
      case 'new-query':
        navigate('/query-analytics')
        break
      case 'notifications':
        // 这里可以打开通知面板
        console.log('打开通知面板')
        break
      case 'performance':
        navigate('/')
        break
      case 'users':
        navigate('/settings')
        break
      default:
        break
    }
    onOpenChange(false)
    setSearchQuery('')
  }

  // 过滤搜索结果 - 支持模糊搜索
  const filterItems = (items: any[], query: string) => {
    if (!query) return items

    const searchText = query.toLowerCase()

    return items.filter(item => {
      // 精确匹配得分更高
      const exactMatch = item.name.toLowerCase().includes(searchText)
      const descriptionMatch = item.description.toLowerCase().includes(searchText)
      const keywordMatch = item.keywords.some((keyword: string) =>
        keyword.toLowerCase().includes(searchText)
      )

      // 模糊匹配 - 检查字符序列
      const fuzzyMatch = (text: string) => {
        const textLower = text.toLowerCase()
        let queryIndex = 0
        for (let i = 0; i < textLower.length && queryIndex < searchText.length; i++) {
          if (textLower[i] === searchText[queryIndex]) {
            queryIndex++
          }
        }
        return queryIndex === searchText.length
      }

      const fuzzyNameMatch = fuzzyMatch(item.name)
      const fuzzyDescMatch = fuzzyMatch(item.description)

      return exactMatch || descriptionMatch || keywordMatch || fuzzyNameMatch || fuzzyDescMatch
    }).sort((a, b) => {
      // 排序：精确匹配优先
      const aExact = a.name.toLowerCase().includes(searchText)
      const bExact = b.name.toLowerCase().includes(searchText)

      if (aExact && !bExact) return -1
      if (!aExact && bExact) return 1

      return a.name.localeCompare(b.name)
    })
  }

  const filteredNavigation = filterItems(navigationItems, searchQuery)
  const filteredActions = filterItems(quickActions, searchQuery)

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput 
        placeholder="搜索功能、页面或执行操作..." 
        value={searchQuery}
        onValueChange={setSearchQuery}
        className="border-0 focus:ring-0"
      />
      <CommandList className="max-h-[400px]">
        <CommandEmpty>
          <div className="flex flex-col items-center gap-2 py-6">
            <Search className="h-8 w-8 text-muted-foreground" />
            <p className="text-sm text-muted-foreground">未找到相关结果</p>
            <p className="text-xs text-muted-foreground">尝试使用不同的关键词</p>
          </div>
        </CommandEmpty>
        
        {filteredNavigation.length > 0 && (
          <CommandGroup heading="页面导航">
            {filteredNavigation.map((item) => {
              const Icon = item.icon
              return (
                <CommandItem
                  key={item.href}
                  value={`${item.name} ${item.description} ${item.keywords.join(' ')}`}
                  onSelect={() => handleNavigate(item.href)}
                  className="flex items-center gap-3 px-3 py-2 cursor-pointer"
                >
                  <Icon className="h-4 w-4 text-muted-foreground" />
                  <div className="flex flex-col">
                    <span className="font-medium">{item.name}</span>
                    <span className="text-xs text-muted-foreground">{item.description}</span>
                  </div>
                </CommandItem>
              )
            })}
          </CommandGroup>
        )}

        {filteredActions.length > 0 && filteredNavigation.length > 0 && (
          <CommandSeparator />
        )}

        {filteredActions.length > 0 && (
          <CommandGroup heading="快速操作">
            {filteredActions.map((item) => {
              const Icon = item.icon
              return (
                <CommandItem
                  key={item.action}
                  value={`${item.name} ${item.description} ${item.keywords.join(' ')}`}
                  onSelect={() => handleAction(item.action)}
                  className="flex items-center gap-3 px-3 py-2 cursor-pointer"
                >
                  <Icon className="h-4 w-4 text-muted-foreground" />
                  <div className="flex flex-col">
                    <span className="font-medium">{item.name}</span>
                    <span className="text-xs text-muted-foreground">{item.description}</span>
                  </div>
                </CommandItem>
              )
            })}
          </CommandGroup>
        )}
      </CommandList>
    </CommandDialog>
  )
}
