import { useState, useEffect, useRef } from 'react'
import { Search, Clock, X, ArrowRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'

// 搜索结果类型
export interface SearchResult {
  id: string
  type: 'table' | 'query' | 'user' | 'config' | 'dashboard'
  title: string
  description?: string
  path?: string
  metadata?: Record<string, any>
  score?: number
}

// 搜索历史项
export interface SearchHistoryItem {
  id: string
  query: string
  timestamp: Date
  resultCount: number
}

// 搜索筛选器
export interface SearchFilter {
  type?: string[]
  dateRange?: {
    start: Date
    end: Date
  }
  status?: string[]
  tags?: string[]
}

// 搜索建议
export interface SearchSuggestion {
  id: string
  text: string
  type: 'recent' | 'popular' | 'suggestion'
  count?: number
}

interface SearchBoxProps {
  placeholder?: string
  onSearch?: (query: string, filters?: SearchFilter) => Promise<SearchResult[]>
  onResultSelect?: (result: SearchResult) => void
  className?: string
  showFilters?: boolean
  showHistory?: boolean
  maxResults?: number
}

export function SearchBox({
  placeholder = "搜索表、查询、用户或配置...",
  onSearch,
  onResultSelect,
  className,
  showHistory = true,
  maxResults = 10
}: SearchBoxProps) {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [history, setHistory] = useState<SearchHistoryItem[]>([])
  const [,] = useState<SearchSuggestion[]>([]) // suggestions和setSuggestions暂时未使用
  const [filters] = useState<SearchFilter>({}) // setFilters暂时未使用
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(-1)

  const inputRef = useRef<HTMLInputElement>(null)
  const resultsRef = useRef<HTMLDivElement>(null)

  // 从localStorage加载搜索历史
  useEffect(() => {
    const savedHistory = localStorage.getItem('duckhub-search-history')
    if (savedHistory) {
      try {
        const parsed = JSON.parse(savedHistory)
        setHistory(parsed.map((item: any) => ({
          ...item,
          timestamp: new Date(item.timestamp)
        })))
      } catch (error) {
        console.error('加载搜索历史失败:', error)
      }
    }
  }, [])

  // 保存搜索历史到localStorage
  const saveToHistory = (searchQuery: string, resultCount: number) => {
    if (!searchQuery.trim()) return

    const newItem: SearchHistoryItem = {
      id: Date.now().toString(),
      query: searchQuery,
      timestamp: new Date(),
      resultCount
    }

    const updatedHistory = [newItem, ...history.filter(item => item.query !== searchQuery)]
      .slice(0, 20) // 最多保存20条历史

    setHistory(updatedHistory)
    localStorage.setItem('duckhub-search-history', JSON.stringify(updatedHistory))
  }

  // 执行搜索
  const performSearch = async (searchQuery: string) => {
    if (!onSearch || !searchQuery.trim()) {
      setResults([])
      return
    }

    setIsLoading(true)
    try {
      const searchResults = await onSearch(searchQuery, filters)
      setResults(searchResults.slice(0, maxResults))
      saveToHistory(searchQuery, searchResults.length)
    } catch (error) {
      console.error('搜索失败:', error)
      setResults([])
    } finally {
      setIsLoading(false)
    }
  }

  // 处理输入变化
  const handleInputChange = (value: string) => {
    setQuery(value)
    setSelectedIndex(-1)

    if (value.trim()) {
      // 防抖搜索
      const timeoutId = setTimeout(() => {
        performSearch(value)
      }, 300)

      return () => clearTimeout(timeoutId)
    } else {
      setResults([])
    }
  }

  // 处理键盘导航
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isOpen) return

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setSelectedIndex(prev => 
          prev < results.length - 1 ? prev + 1 : prev
        )
        break
      case 'ArrowUp':
        e.preventDefault()
        setSelectedIndex(prev => prev > 0 ? prev - 1 : -1)
        break
      case 'Enter':
        e.preventDefault()
        if (selectedIndex >= 0 && results[selectedIndex]) {
          handleResultSelect(results[selectedIndex])
        } else if (query.trim()) {
          performSearch(query)
        }
        break
      case 'Escape':
        setIsOpen(false)
        inputRef.current?.blur()
        break
    }
  }

  // 处理结果选择
  const handleResultSelect = (result: SearchResult) => {
    onResultSelect?.(result)
    setIsOpen(false)
    setQuery('')
    setResults([])
  }

  // 处理历史项选择
  const handleHistorySelect = (historyItem: SearchHistoryItem) => {
    setQuery(historyItem.query)
    performSearch(historyItem.query)
  }

  // 清除搜索历史
  const clearHistory = () => {
    setHistory([])
    localStorage.removeItem('duckhub-search-history')
  }

  // 获取搜索建议
  const getSearchSuggestions = () => {
    if (query.trim()) return []

    const recentSearches = history.slice(0, 5).map(item => ({
      id: item.id,
      text: item.query,
      type: 'recent' as const,
      count: item.resultCount
    }))

    return recentSearches
  }

  const currentSuggestions = getSearchSuggestions()
  const showResults = isOpen && (results.length > 0 || currentSuggestions.length > 0 || query.trim())

  return (
    <div className={cn("relative w-full max-w-lg", className)}>
      <div className="relative">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          ref={inputRef}
          type="text"
          placeholder={placeholder}
          value={query}
          onChange={(e) => handleInputChange(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={() => setIsOpen(true)}
          className="pl-10 pr-12"
        />
        {query && (
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              setQuery('')
              setResults([])
              setIsOpen(false)
            }}
            className="absolute right-1 top-1/2 h-6 w-6 -translate-y-1/2"
          >
            <X className="h-3 w-3" />
          </Button>
        )}
      </div>

      {/* 搜索结果下拉框 */}
      {showResults && (
        <div 
          ref={resultsRef}
          className="absolute top-full left-0 right-0 mt-1 bg-popover border rounded-lg shadow-lg z-50 max-h-96 overflow-y-auto"
        >
          {/* 搜索结果 */}
          {results.length > 0 && (
            <div className="p-2">
              <div className="text-xs text-muted-foreground mb-2 px-2">
                搜索结果 ({results.length})
              </div>
              {results.map((result, index) => (
                <div
                  key={result.id}
                  onClick={() => handleResultSelect(result)}
                  className={cn(
                    "flex items-center gap-3 p-2 rounded-md cursor-pointer transition-colors",
                    selectedIndex === index ? "bg-accent" : "hover:bg-accent/50"
                  )}
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{result.title}</span>
                      <Badge variant="secondary" className="text-xs">
                        {result.type}
                      </Badge>
                    </div>
                    {result.description && (
                      <p className="text-sm text-muted-foreground mt-1">
                        {result.description}
                      </p>
                    )}
                  </div>
                  <ArrowRight className="h-4 w-4 text-muted-foreground" />
                </div>
              ))}
            </div>
          )}

          {/* 搜索建议和历史 */}
          {currentSuggestions.length > 0 && results.length === 0 && (
            <div className="p-2">
              <div className="flex items-center justify-between mb-2 px-2">
                <span className="text-xs text-muted-foreground">最近搜索</span>
                {showHistory && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={clearHistory}
                    className="text-xs h-auto p-1"
                  >
                    清除
                  </Button>
                )}
              </div>
              {currentSuggestions.map((suggestion) => (
                <div
                  key={suggestion.id}
                  onClick={() => handleHistorySelect(history.find(h => h.id === suggestion.id)!)}
                  className="flex items-center gap-3 p-2 rounded-md cursor-pointer hover:bg-accent/50 transition-colors"
                >
                  <Clock className="h-4 w-4 text-muted-foreground" />
                  <div className="flex-1">
                    <span className="text-sm">{suggestion.text}</span>
                    {suggestion.count !== undefined && (
                      <span className="text-xs text-muted-foreground ml-2">
                        {suggestion.count} 个结果
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* 无结果 */}
          {query.trim() && results.length === 0 && !isLoading && (
            <div className="p-4 text-center text-muted-foreground">
              <Search className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p className="text-sm">未找到相关结果</p>
              <p className="text-xs mt-1">尝试使用不同的关键词</p>
            </div>
          )}

          {/* 加载状态 */}
          {isLoading && (
            <div className="p-4 text-center">
              <div className="animate-spin h-5 w-5 border-2 border-primary border-t-transparent rounded-full mx-auto" />
              <p className="text-sm text-muted-foreground mt-2">搜索中...</p>
            </div>
          )}
        </div>
      )}

      {/* 点击外部关闭 */}
      {isOpen && (
        <div 
          className="fixed inset-0 z-40" 
          onClick={() => setIsOpen(false)}
        />
      )}
    </div>
  )
}
