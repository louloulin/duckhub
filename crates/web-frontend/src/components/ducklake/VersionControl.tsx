import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  GitBranch,
  Clock,
  User,
  FileText,
  Plus,
  Minus,
  Edit,
  History,
  RotateCcw,
  GitCommit,
} from 'lucide-react'

interface VersionChange {
  id: string
  version: number
  timestamp: string
  author: string
  operation: 'create' | 'update' | 'delete' | 'schema_change'
  table: string
  description: string
  changes: {
    added: number
    modified: number
    deleted: number
  }
}

export default function VersionControl() {
  const [versions, setVersions] = useState<VersionChange[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedDatabase, setSelectedDatabase] = useState('financial_data')

  useEffect(() => {
    const loadVersions = async () => {
      setLoading(true)
      try {
        const response = await fetch('/api/v1/ducklake/versions', {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        })

        if (response.ok) {
          const data = await response.json()
          setVersions(data.data || [])
        } else {
          console.error('获取版本列表失败')
          setVersions([])
        }
      } catch (error) {
        console.error('加载版本列表时出错:', error)
        // 不再使用fallback数据，直接设置为空数组
        setVersions([])
      } finally {
        setLoading(false)
      }
    }

    loadVersions()
  }, [selectedDatabase])

  const getOperationIcon = (operation: string) => {
    switch (operation) {
      case 'create':
        return <Plus className="h-4 w-4 text-green-600" />
      case 'update':
        return <Edit className="h-4 w-4 text-blue-600" />
      case 'delete':
        return <Minus className="h-4 w-4 text-red-600" />
      case 'schema_change':
        return <GitBranch className="h-4 w-4 text-purple-600" />
      default:
        return <FileText className="h-4 w-4 text-gray-600" />
    }
  }

  const getOperationBadge = (operation: string) => {
    const configs = {
      create: { label: '新增', className: 'bg-green-100 text-green-800 border-green-200' },
      update: { label: '更新', className: 'bg-blue-100 text-blue-800 border-blue-200' },
      delete: { label: '删除', className: 'bg-red-100 text-red-800 border-red-200' },
      schema_change: { label: 'Schema', className: 'bg-purple-100 text-purple-800 border-purple-200' },
    }
    
    const config = configs[operation as keyof typeof configs] || configs.update
    
    return (
      <Badge variant="outline" className={config.className}>
        {getOperationIcon(operation)}
        <span className="ml-1">{config.label}</span>
      </Badge>
    )
  }

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    
    if (diffHours < 1) {
      const diffMinutes = Math.floor(diffMs / (1000 * 60))
      return `${diffMinutes}分钟前`
    } else if (diffHours < 24) {
      return `${diffHours}小时前`
    } else {
      const diffDays = Math.floor(diffHours / 24)
      return `${diffDays}天前`
    }
  }

  if (loading) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="flex items-center justify-center h-32">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <GitBranch className="h-5 w-5" />
                版本控制中心
              </CardTitle>
              <CardDescription>
                查看数据版本历史和变更记录
              </CardDescription>
            </div>
            <div className="flex items-center gap-3">
              <select
                value={selectedDatabase}
                onChange={(e) => setSelectedDatabase(e.target.value)}
                className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="financial_data">financial_data</option>
                <option value="analytics_warehouse">analytics_warehouse</option>
                <option value="backup_archive">backup_archive</option>
              </select>
              <Button variant="outline" className="flex items-center gap-2">
                <History className="h-4 w-4" />
                查看完整历史
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {/* 版本时间线 */}
          <div className="space-y-4">
            {versions.map((version, index) => (
              <div key={version.id} className="relative">
                {/* 时间线连接线 */}
                {index < versions.length - 1 && (
                  <div className="absolute left-6 top-12 w-0.5 h-16 bg-gray-200"></div>
                )}
                
                <div className="flex items-start gap-4 p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors">
                  {/* 版本号圆圈 */}
                  <div className="flex-shrink-0 w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center">
                    <GitCommit className="h-6 w-6 text-blue-600" />
                  </div>
                  
                  {/* 版本信息 */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <Badge variant="outline" className="font-mono">
                        v{version.version}
                      </Badge>
                      {getOperationBadge(version.operation)}
                      <span className="text-sm text-gray-500">
                        {version.table}
                      </span>
                    </div>
                    
                    <p className="text-gray-900 mb-2">{version.description}</p>
                    
                    <div className="flex items-center gap-4 text-sm text-gray-500">
                      <div className="flex items-center gap-1">
                        <User className="h-4 w-4" />
                        {version.author}
                      </div>
                      <div className="flex items-center gap-1">
                        <Clock className="h-4 w-4" />
                        {formatTimestamp(version.timestamp)}
                      </div>
                    </div>
                    
                    {/* 变更统计 */}
                    <div className="flex items-center gap-4 mt-3 text-sm">
                      {version.changes.added > 0 && (
                        <span className="text-green-600 flex items-center gap-1">
                          <Plus className="h-3 w-3" />
                          {version.changes.added.toLocaleString()} 新增
                        </span>
                      )}
                      {version.changes.modified > 0 && (
                        <span className="text-blue-600 flex items-center gap-1">
                          <Edit className="h-3 w-3" />
                          {version.changes.modified.toLocaleString()} 修改
                        </span>
                      )}
                      {version.changes.deleted > 0 && (
                        <span className="text-red-600 flex items-center gap-1">
                          <Minus className="h-3 w-3" />
                          {version.changes.deleted.toLocaleString()} 删除
                        </span>
                      )}
                    </div>
                  </div>
                  
                  {/* 操作按钮 */}
                  <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm">
                      查看详情
                    </Button>
                    <Button variant="outline" size="sm" className="flex items-center gap-1">
                      <RotateCcw className="h-3 w-3" />
                      回滚
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
          
          {versions.length === 0 && (
            <div className="text-center py-8 text-gray-500">
              <GitBranch className="h-12 w-12 mx-auto mb-4 text-gray-300" />
              <p>暂无版本历史记录</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
