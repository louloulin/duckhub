import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Layers,
  Clock,
  Search,
  MoreHorizontal,
  Eye,
  GitCompare,
  Download,
  Trash2,
  Tag,
  HardDrive,
  FileText,
  Plus,
  BarChart3,
  List,
  Camera,
  AlertTriangle,
  ArrowRight,
  ArrowLeft,
  User,
  Database,
} from 'lucide-react'

interface Snapshot {
  id: string
  version: number
  timestamp: string
  database: string
  size: string
  tables: number
  description?: string
  tags: string[]
  changes: number
  author: string
  type: 'manual' | 'automatic' | 'scheduled'
  parentVersion?: number
  checksum: string
  metadata: {
    rowCount: number
    schemaVersion: number
    compressionRatio: number
  }
}

interface SnapshotComparison {
  baseSnapshot: Snapshot
  targetSnapshot: Snapshot
  differences: {
    tablesAdded: string[]
    tablesRemoved: string[]
    tablesModified: string[]
    rowsAdded: number
    rowsRemoved: number
    rowsModified: number
    schemaChanges: number
  }
}

export default function SnapshotBrowser() {
  const [snapshots, setSnapshots] = useState<Snapshot[]>([])
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedDatabase, setSelectedDatabase] = useState('all')
  const [viewMode, setViewMode] = useState<'list' | 'timeline'>('list')
  const [selectedSnapshots, setSelectedSnapshots] = useState<string[]>([])
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showCompareDialog, setShowCompareDialog] = useState(false)
  const [showDetailDialog, setShowDetailDialog] = useState(false)
  const [selectedSnapshot, setSelectedSnapshot] = useState<Snapshot | null>(null)
  const [comparison, setComparison] = useState<SnapshotComparison | null>(null)
  const [newSnapshot, setNewSnapshot] = useState({
    description: '',
    tags: '',
    database: 'financial_data',
  })

  // 模拟数据加载
  useEffect(() => {
    const loadSnapshots = async () => {
      setLoading(true)
      try {
        const response = await fetch('/api/v1/ducklake/databases/financial_data/snapshots', {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        })

        if (response.ok) {
          const data = await response.json()
          setSnapshots(data.data || [])
        } else {
          console.error('获取快照列表失败')
          setSnapshots([])
        }
      } catch (error) {
        console.error('加载快照列表时出错:', error)
        // 不再使用fallback数据，直接设置为空数组
        setSnapshots([])
      } finally {
        setLoading(false)
      }
    }

    loadSnapshots()
  }, [])

  const filteredSnapshots = snapshots.filter(snapshot => {
    const matchesSearch = snapshot.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         snapshot.database.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         snapshot.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()))
    
    const matchesDatabase = selectedDatabase === 'all' || snapshot.database === selectedDatabase
    
    return matchesSearch && matchesDatabase
  })

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    const diffDays = Math.floor(diffHours / 24)

    if (diffHours < 1) {
      const diffMinutes = Math.floor(diffMs / (1000 * 60))
      return `${diffMinutes}分钟前`
    } else if (diffHours < 24) {
      return `${diffHours}小时前`
    } else if (diffDays < 7) {
      return `${diffDays}天前`
    } else {
      return timestamp
    }
  }

  const getTagColor = (tag: string) => {
    const colors: Record<string, string> = {
      daily: 'bg-blue-100 text-blue-800',
      production: 'bg-green-100 text-green-800',
      backup: 'bg-gray-100 text-gray-800',
      analytics: 'bg-purple-100 text-purple-800',
      scheduled: 'bg-orange-100 text-orange-800',
      eod: 'bg-red-100 text-red-800',
      trading: 'bg-indigo-100 text-indigo-800',
      manual: 'bg-yellow-100 text-yellow-800',
    }
    return colors[tag] || 'bg-gray-100 text-gray-800'
  }

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'manual':
        return <User className="h-3 w-3" />
      case 'automatic':
        return <Database className="h-3 w-3" />
      case 'scheduled':
        return <Clock className="h-3 w-3" />
      default:
        return <Camera className="h-3 w-3" />
    }
  }

  const getTypeBadge = (type: string) => {
    const configs = {
      manual: { label: '手动', className: 'bg-blue-100 text-blue-800' },
      automatic: { label: '自动', className: 'bg-green-100 text-green-800' },
      scheduled: { label: '定时', className: 'bg-orange-100 text-orange-800' },
    }
    const config = configs[type as keyof typeof configs] || configs.manual
    return (
      <Badge variant="outline" className={config.className}>
        {getTypeIcon(type)}
        <span className="ml-1">{config.label}</span>
      </Badge>
    )
  }

  const handleCreateSnapshot = async () => {
    if (!newSnapshot.description) return

    const snapshot: Snapshot = {
      id: Date.now().toString(),
      version: Math.max(...snapshots.map(s => s.version)) + 1,
      timestamp: new Date().toISOString().slice(0, 19).replace('T', ' '),
      database: newSnapshot.database,
      size: '0 MB',
      tables: 0,
      description: newSnapshot.description,
      tags: newSnapshot.tags.split(',').map(t => t.trim()).filter(Boolean),
      changes: 0,
      author: 'current_user',
      type: 'manual',
      checksum: `sha256:${Math.random().toString(36).substring(2)}...`,
      metadata: {
        rowCount: 0,
        schemaVersion: 1,
        compressionRatio: 1.0,
      },
    }

    setSnapshots(prev => [snapshot, ...prev])
    setNewSnapshot({ description: '', tags: '', database: 'financial_data' })
    setShowCreateDialog(false)
  }

  const handleDeleteSnapshot = (id: string) => {
    setSnapshots(prev => prev.filter(s => s.id !== id))
  }

  const handleCompareSnapshots = async () => {
    if (selectedSnapshots.length !== 2) return

    const [baseId, targetId] = selectedSnapshots
    const baseSnapshot = snapshots.find(s => s.id === baseId)
    const targetSnapshot = snapshots.find(s => s.id === targetId)

    if (!baseSnapshot || !targetSnapshot) return

    try {
      // 调用后端API进行快照比较
      const response = await fetch('/api/v1/ducklake/snapshots/compare', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify({
          base_snapshot_id: baseId,
          target_snapshot_id: targetId,
        }),
      })

      if (response.ok) {
        const comparisonData = await response.json()
        setComparison(comparisonData.data)
      } else {
        // 如果API不可用，使用计算的比较结果
        const calculatedComparison: SnapshotComparison = {
          baseSnapshot,
          targetSnapshot,
          differences: {
            tablesAdded: [],
            tablesRemoved: [],
            tablesModified: [],
            rowsAdded: Math.max(0, targetSnapshot.metadata.rowCount - baseSnapshot.metadata.rowCount),
            rowsRemoved: Math.max(0, baseSnapshot.metadata.rowCount - targetSnapshot.metadata.rowCount),
            rowsModified: Math.min(baseSnapshot.metadata.rowCount, targetSnapshot.metadata.rowCount),
            schemaChanges: Math.abs(targetSnapshot.metadata.schemaVersion - baseSnapshot.metadata.schemaVersion),
          },
        }
        setComparison(calculatedComparison)
      }
      setShowCompareDialog(true)
    } catch (error) {
      console.error('快照比较失败:', error)
    }
  }

  const toggleSnapshotSelection = (id: string) => {
    setSelectedSnapshots(prev => {
      if (prev.includes(id)) {
        return prev.filter(sid => sid !== id)
      } else if (prev.length < 2) {
        return [...prev, id]
      } else {
        return [prev[1], id] // 替换第一个选择
      }
    })
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
      {/* 头部控制区 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Layers className="h-5 w-5" />
                快照浏览器
              </CardTitle>
              <CardDescription>
                浏览和管理所有数据快照，支持时间旅行查询和快照比较
              </CardDescription>
            </div>
            <div className="flex items-center gap-3">
              {/* 视图切换 */}
              <div className="flex items-center gap-1 bg-gray-100 rounded-lg p-1">
                <Button
                  variant={viewMode === 'list' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setViewMode('list')}
                  className="flex items-center gap-1"
                >
                  <List className="h-4 w-4" />
                  列表
                </Button>
                <Button
                  variant={viewMode === 'timeline' ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setViewMode('timeline')}
                  className="flex items-center gap-1"
                >
                  <BarChart3 className="h-4 w-4" />
                  时间线
                </Button>
              </div>

              {/* 比较按钮 */}
              <Button
                variant="outline"
                onClick={handleCompareSnapshots}
                disabled={selectedSnapshots.length !== 2}
                className="flex items-center gap-2"
              >
                <GitCompare className="h-4 w-4" />
                比较快照 {selectedSnapshots.length > 0 && `(${selectedSnapshots.length})`}
              </Button>

              {/* 创建快照按钮 */}
              <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                <DialogTrigger asChild>
                  <Button className="flex items-center gap-2">
                    <Plus className="h-4 w-4" />
                    创建快照
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>创建新快照</DialogTitle>
                    <DialogDescription>
                      为选定的数据库创建一个新的数据快照
                    </DialogDescription>
                  </DialogHeader>
                  <div className="grid gap-4 py-4">
                    <div className="grid gap-2">
                      <Label htmlFor="database">数据库</Label>
                      <select
                        id="database"
                        value={newSnapshot.database}
                        onChange={(e) => setNewSnapshot(prev => ({ ...prev, database: e.target.value }))}
                        className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      >
                        <option value="financial_data">financial_data</option>
                        <option value="analytics_warehouse">analytics_warehouse</option>
                        <option value="backup_archive">backup_archive</option>
                      </select>
                    </div>
                    <div className="grid gap-2">
                      <Label htmlFor="description">描述</Label>
                      <Input
                        id="description"
                        value={newSnapshot.description}
                        onChange={(e) => setNewSnapshot(prev => ({ ...prev, description: e.target.value }))}
                        placeholder="快照描述..."
                      />
                    </div>
                    <div className="grid gap-2">
                      <Label htmlFor="tags">标签 (用逗号分隔)</Label>
                      <Input
                        id="tags"
                        value={newSnapshot.tags}
                        onChange={(e) => setNewSnapshot(prev => ({ ...prev, tags: e.target.value }))}
                        placeholder="例如: backup, manual, important"
                      />
                    </div>
                  </div>
                  <DialogFooter>
                    <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                      取消
                    </Button>
                    <Button onClick={handleCreateSnapshot}>
                      创建快照
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {/* 搜索和过滤 */}
          <div className="flex gap-4 mb-6">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
                <Input
                  placeholder="搜索快照..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <select
              value={selectedDatabase}
              onChange={(e) => setSelectedDatabase(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">所有数据库</option>
              <option value="financial_data">financial_data</option>
              <option value="analytics_warehouse">analytics_warehouse</option>
              <option value="backup_archive">backup_archive</option>
            </select>
          </div>

          {/* 快照视图 */}
          {viewMode === 'list' ? (
            /* 列表视图 */
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12">选择</TableHead>
                  <TableHead>版本</TableHead>
                  <TableHead>数据库</TableHead>
                  <TableHead>时间</TableHead>
                  <TableHead>类型</TableHead>
                  <TableHead>大小</TableHead>
                  <TableHead>变更</TableHead>
                  <TableHead>标签</TableHead>
                  <TableHead>描述</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredSnapshots.map((snapshot) => (
                  <TableRow
                    key={snapshot.id}
                    className={selectedSnapshots.includes(snapshot.id) ? 'bg-blue-50' : ''}
                  >
                    <TableCell>
                      <input
                        type="checkbox"
                        checked={selectedSnapshots.includes(snapshot.id)}
                        onChange={() => toggleSnapshotSelection(snapshot.id)}
                        className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                      />
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="font-mono">
                          v{snapshot.version}
                        </Badge>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="font-medium">{snapshot.database}</div>
                      <div className="text-sm text-gray-500 flex items-center gap-1">
                        <FileText className="h-3 w-3" />
                        {snapshot.tables} 表
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <Clock className="h-4 w-4 text-gray-400" />
                        <div>
                          <div className="text-sm">{formatTimestamp(snapshot.timestamp)}</div>
                          <div className="text-xs text-gray-500">{snapshot.timestamp}</div>
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      {getTypeBadge(snapshot.type)}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <HardDrive className="h-4 w-4 text-gray-400" />
                        {snapshot.size}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="secondary" className="text-xs">
                        {snapshot.changes.toLocaleString()} 条
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {snapshot.tags.slice(0, 2).map((tag) => (
                          <Badge
                            key={tag}
                            variant="secondary"
                            className={`text-xs ${getTagColor(tag)}`}
                          >
                            {tag}
                          </Badge>
                        ))}
                        {snapshot.tags.length > 2 && (
                          <Badge variant="secondary" className="text-xs">
                            +{snapshot.tags.length - 2}
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="max-w-xs truncate text-sm text-gray-600">
                        {snapshot.description}
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" className="h-8 w-8 p-0">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuLabel>操作</DropdownMenuLabel>
                          <DropdownMenuItem onClick={() => {
                            setSelectedSnapshot(snapshot)
                            setShowDetailDialog(true)
                          }}>
                            <Eye className="mr-2 h-4 w-4" />
                            查看详情
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => toggleSnapshotSelection(snapshot.id)}>
                            <GitCompare className="mr-2 h-4 w-4" />
                            选择比较
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <Download className="mr-2 h-4 w-4" />
                            导出数据
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem>
                            <Tag className="mr-2 h-4 w-4" />
                            管理标签
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            className="text-red-600"
                            onClick={() => handleDeleteSnapshot(snapshot.id)}
                          >
                            <Trash2 className="mr-2 h-4 w-4" />
                            删除快照
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            /* 时间线视图 */
            <div className="space-y-4">
              {filteredSnapshots.map((snapshot, index) => (
                <div key={snapshot.id} className="relative">
                  {/* 时间线连接线 */}
                  {index < filteredSnapshots.length - 1 && (
                    <div className="absolute left-6 top-16 w-0.5 h-20 bg-gray-200"></div>
                  )}

                  <div className={`flex items-start gap-4 p-4 border rounded-lg hover:bg-gray-50 transition-colors ${
                    selectedSnapshots.includes(snapshot.id) ? 'bg-blue-50 border-blue-200' : 'border-gray-200'
                  }`}>
                    {/* 时间线节点 */}
                    <div className="flex-shrink-0 w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center">
                      <Layers className="h-6 w-6 text-blue-600" />
                    </div>

                    {/* 快照信息 */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-3 mb-2">
                        <input
                          type="checkbox"
                          checked={selectedSnapshots.includes(snapshot.id)}
                          onChange={() => toggleSnapshotSelection(snapshot.id)}
                          className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                        />
                        <Badge variant="outline" className="font-mono">
                          v{snapshot.version}
                        </Badge>
                        {getTypeBadge(snapshot.type)}
                        <span className="text-sm text-gray-500">
                          {snapshot.database}
                        </span>
                      </div>

                      <h3 className="font-medium text-gray-900 mb-1">{snapshot.description}</h3>

                      <div className="flex items-center gap-4 text-sm text-gray-500 mb-2">
                        <div className="flex items-center gap-1">
                          <User className="h-4 w-4" />
                          {snapshot.author}
                        </div>
                        <div className="flex items-center gap-1">
                          <Clock className="h-4 w-4" />
                          {formatTimestamp(snapshot.timestamp)}
                        </div>
                        <div className="flex items-center gap-1">
                          <HardDrive className="h-4 w-4" />
                          {snapshot.size}
                        </div>
                      </div>

                      {/* 标签 */}
                      <div className="flex flex-wrap gap-1 mb-2">
                        {snapshot.tags.map((tag) => (
                          <Badge
                            key={tag}
                            variant="secondary"
                            className={`text-xs ${getTagColor(tag)}`}
                          >
                            {tag}
                          </Badge>
                        ))}
                      </div>

                      {/* 统计信息 */}
                      <div className="flex items-center gap-4 text-sm">
                        <span className="text-blue-600">
                          {snapshot.changes.toLocaleString()} 条变更
                        </span>
                        <span className="text-gray-500">
                          {snapshot.metadata.rowCount.toLocaleString()} 行数据
                        </span>
                        <span className="text-gray-500">
                          {snapshot.tables} 个表
                        </span>
                      </div>
                    </div>

                    {/* 操作按钮 */}
                    <div className="flex items-center gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          setSelectedSnapshot(snapshot)
                          setShowDetailDialog(true)
                        }}
                      >
                        查看详情
                      </Button>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="outline" size="sm">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem>
                            <Download className="mr-2 h-4 w-4" />
                            导出数据
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <Tag className="mr-2 h-4 w-4" />
                            管理标签
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            className="text-red-600"
                            onClick={() => handleDeleteSnapshot(snapshot.id)}
                          >
                            <Trash2 className="mr-2 h-4 w-4" />
                            删除快照
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {filteredSnapshots.length === 0 && (
            <div className="text-center py-8 text-gray-500">
              <Layers className="h-12 w-12 mx-auto mb-4 text-gray-300" />
              <p>没有找到匹配的快照</p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* 快照详情对话框 */}
      <Dialog open={showDetailDialog} onOpenChange={setShowDetailDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Layers className="h-5 w-5" />
              快照详情 - v{selectedSnapshot?.version}
            </DialogTitle>
            <DialogDescription>
              查看快照的详细信息和元数据
            </DialogDescription>
          </DialogHeader>
          {selectedSnapshot && (
            <div className="space-y-6">
              {/* 基本信息 */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-sm font-medium text-gray-600">版本号</Label>
                  <p className="text-lg font-mono">v{selectedSnapshot.version}</p>
                </div>
                <div>
                  <Label className="text-sm font-medium text-gray-600">数据库</Label>
                  <p className="text-lg">{selectedSnapshot.database}</p>
                </div>
                <div>
                  <Label className="text-sm font-medium text-gray-600">创建时间</Label>
                  <p>{selectedSnapshot.timestamp}</p>
                </div>
                <div>
                  <Label className="text-sm font-medium text-gray-600">创建者</Label>
                  <p className="flex items-center gap-1">
                    {getTypeIcon(selectedSnapshot.type)}
                    {selectedSnapshot.author}
                  </p>
                </div>
                <div>
                  <Label className="text-sm font-medium text-gray-600">快照大小</Label>
                  <p>{selectedSnapshot.size}</p>
                </div>
                <div>
                  <Label className="text-sm font-medium text-gray-600">表数量</Label>
                  <p>{selectedSnapshot.tables} 个表</p>
                </div>
              </div>

              {/* 描述 */}
              <div>
                <Label className="text-sm font-medium text-gray-600">描述</Label>
                <p className="mt-1 p-3 bg-gray-50 rounded-md">{selectedSnapshot.description}</p>
              </div>

              {/* 标签 */}
              <div>
                <Label className="text-sm font-medium text-gray-600">标签</Label>
                <div className="flex flex-wrap gap-2 mt-1">
                  {selectedSnapshot.tags.map((tag) => (
                    <Badge
                      key={tag}
                      variant="secondary"
                      className={getTagColor(tag)}
                    >
                      {tag}
                    </Badge>
                  ))}
                </div>
              </div>

              {/* 元数据 */}
              <div>
                <Label className="text-sm font-medium text-gray-600">元数据</Label>
                <div className="mt-2 grid grid-cols-3 gap-4 p-4 bg-gray-50 rounded-md">
                  <div>
                    <p className="text-sm text-gray-500">数据行数</p>
                    <p className="font-medium">{selectedSnapshot.metadata.rowCount.toLocaleString()}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-500">Schema版本</p>
                    <p className="font-medium">v{selectedSnapshot.metadata.schemaVersion}</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-500">压缩率</p>
                    <p className="font-medium">{(selectedSnapshot.metadata.compressionRatio * 100).toFixed(1)}%</p>
                  </div>
                </div>
              </div>

              {/* 校验和 */}
              <div>
                <Label className="text-sm font-medium text-gray-600">校验和</Label>
                <p className="mt-1 font-mono text-sm bg-gray-50 p-2 rounded">{selectedSnapshot.checksum}</p>
              </div>

              {/* 变更统计 */}
              <div>
                <Label className="text-sm font-medium text-gray-600">变更统计</Label>
                <div className="mt-2 flex items-center gap-4">
                  <Badge variant="outline" className="bg-blue-50 text-blue-700">
                    {selectedSnapshot.changes.toLocaleString()} 条变更
                  </Badge>
                  {selectedSnapshot.parentVersion && (
                    <span className="text-sm text-gray-500">
                      基于版本 v{selectedSnapshot.parentVersion}
                    </span>
                  )}
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowDetailDialog(false)}>
              关闭
            </Button>
            <Button>
              <Download className="mr-2 h-4 w-4" />
              导出快照
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 快照比较对话框 */}
      <Dialog open={showCompareDialog} onOpenChange={setShowCompareDialog}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <GitCompare className="h-5 w-5" />
              快照比较
            </DialogTitle>
            <DialogDescription>
              比较两个快照之间的差异
            </DialogDescription>
          </DialogHeader>
          {comparison && (
            <div className="space-y-6">
              {/* 比较概览 */}
              <div className="grid grid-cols-2 gap-6">
                <div className="p-4 border rounded-lg">
                  <div className="flex items-center gap-2 mb-2">
                    <ArrowLeft className="h-4 w-4 text-gray-400" />
                    <span className="font-medium">基准快照</span>
                  </div>
                  <div className="space-y-1">
                    <p className="font-mono">v{comparison.baseSnapshot.version}</p>
                    <p className="text-sm text-gray-600">{comparison.baseSnapshot.description}</p>
                    <p className="text-xs text-gray-500">{comparison.baseSnapshot.timestamp}</p>
                  </div>
                </div>
                <div className="p-4 border rounded-lg">
                  <div className="flex items-center gap-2 mb-2">
                    <ArrowRight className="h-4 w-4 text-gray-400" />
                    <span className="font-medium">目标快照</span>
                  </div>
                  <div className="space-y-1">
                    <p className="font-mono">v{comparison.targetSnapshot.version}</p>
                    <p className="text-sm text-gray-600">{comparison.targetSnapshot.description}</p>
                    <p className="text-xs text-gray-500">{comparison.targetSnapshot.timestamp}</p>
                  </div>
                </div>
              </div>

              {/* 差异统计 */}
              <div>
                <h3 className="font-medium mb-3">差异统计</h3>
                <div className="grid grid-cols-3 gap-4">
                  <Card>
                    <CardContent className="p-4">
                      <div className="flex items-center gap-2">
                        <Plus className="h-4 w-4 text-green-600" />
                        <span className="text-sm font-medium">新增数据</span>
                      </div>
                      <p className="text-lg font-bold text-green-600 mt-1">
                        {comparison.differences.rowsAdded.toLocaleString()}
                      </p>
                    </CardContent>
                  </Card>
                  <Card>
                    <CardContent className="p-4">
                      <div className="flex items-center gap-2">
                        <Trash2 className="h-4 w-4 text-red-600" />
                        <span className="text-sm font-medium">删除数据</span>
                      </div>
                      <p className="text-lg font-bold text-red-600 mt-1">
                        {comparison.differences.rowsRemoved.toLocaleString()}
                      </p>
                    </CardContent>
                  </Card>
                  <Card>
                    <CardContent className="p-4">
                      <div className="flex items-center gap-2">
                        <FileText className="h-4 w-4 text-blue-600" />
                        <span className="text-sm font-medium">修改数据</span>
                      </div>
                      <p className="text-lg font-bold text-blue-600 mt-1">
                        {comparison.differences.rowsModified.toLocaleString()}
                      </p>
                    </CardContent>
                  </Card>
                </div>
              </div>

              {/* 表变更 */}
              <div>
                <h3 className="font-medium mb-3">表结构变更</h3>
                <div className="space-y-3">
                  {comparison.differences.tablesAdded.length > 0 && (
                    <div>
                      <p className="text-sm font-medium text-green-600 mb-1">新增表</p>
                      <div className="flex flex-wrap gap-1">
                        {comparison.differences.tablesAdded.map((table) => (
                          <Badge key={table} className="bg-green-100 text-green-800">
                            {table}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                  {comparison.differences.tablesRemoved.length > 0 && (
                    <div>
                      <p className="text-sm font-medium text-red-600 mb-1">删除表</p>
                      <div className="flex flex-wrap gap-1">
                        {comparison.differences.tablesRemoved.map((table) => (
                          <Badge key={table} className="bg-red-100 text-red-800">
                            {table}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                  {comparison.differences.tablesModified.length > 0 && (
                    <div>
                      <p className="text-sm font-medium text-blue-600 mb-1">修改表</p>
                      <div className="flex flex-wrap gap-1">
                        {comparison.differences.tablesModified.map((table) => (
                          <Badge key={table} className="bg-blue-100 text-blue-800">
                            {table}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </div>

              {/* Schema变更 */}
              {comparison.differences.schemaChanges > 0 && (
                <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4 text-yellow-600" />
                    <span className="font-medium text-yellow-800">Schema变更</span>
                  </div>
                  <p className="text-sm text-yellow-700 mt-1">
                    检测到 {comparison.differences.schemaChanges} 个Schema变更，请注意兼容性
                  </p>
                </div>
              )}
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCompareDialog(false)}>
              关闭
            </Button>
            <Button>
              <Download className="mr-2 h-4 w-4" />
              导出比较报告
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
