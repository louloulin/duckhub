import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { duckLakeAPI } from '@/services/api'
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
  Database,
  Plus,
  Settings,
  Wifi,
  WifiOff,
  AlertCircle,
  CheckCircle,
  Trash2,
  MoreHorizontal,
  Eye,
  HardDrive,
  Clock,
  Users,
  FileText,
} from 'lucide-react'

interface DuckLakeDatabase {
  id: string
  name: string
  path: string
  status: 'connected' | 'disconnected' | 'error'
  size: string
  tables: number
  lastAccessed: string
  connections: number
  description?: string
}

export default function DatabasePanel() {
  const [databases, setDatabases] = useState<DuckLakeDatabase[]>([])
  const [loading, setLoading] = useState(true)
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [newDatabase, setNewDatabase] = useState({
    name: '',
    path: '',
    description: '',
  })

  // 加载数据库列表
  useEffect(() => {
    const loadDatabases = async () => {
      setLoading(true)
      try {
        const response = await duckLakeAPI.getDatabases()
        setDatabases(response.data.data || [])
      } catch (error) {
        console.error('加载数据库列表时出错:', error)
        // 显示错误状态，不使用mock数据
        setDatabases([])
      } finally {
        setLoading(false)
      }
    }

    loadDatabases()
  }, [])

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'connected':
        return (
          <Badge variant="default" className="bg-green-100 text-green-800 border-green-200">
            <CheckCircle className="h-3 w-3 mr-1" />
            已连接
          </Badge>
        )
      case 'disconnected':
        return (
          <Badge variant="secondary" className="bg-gray-100 text-gray-800">
            <WifiOff className="h-3 w-3 mr-1" />
            已断开
          </Badge>
        )
      case 'error':
        return (
          <Badge variant="destructive">
            <AlertCircle className="h-3 w-3 mr-1" />
            错误
          </Badge>
        )
      default:
        return null
    }
  }

  const handleAddDatabase = async () => {
    if (!newDatabase.name || !newDatabase.path) return

    // 模拟添加数据库
    const newDb: DuckLakeDatabase = {
      id: Date.now().toString(),
      name: newDatabase.name,
      path: newDatabase.path,
      status: 'connected',
      size: '0 MB',
      tables: 0,
      lastAccessed: '刚刚',
      connections: 0,
      description: newDatabase.description,
    }

    setDatabases(prev => [...prev, newDb])
    setNewDatabase({ name: '', path: '', description: '' })
    setShowAddDialog(false)
  }

  const handleDetachDatabase = (id: string) => {
    setDatabases(prev => prev.filter(db => db.id !== id))
  }

  const handleConnectDatabase = (id: string) => {
    setDatabases(prev =>
      prev.map(db =>
        db.id === id ? { ...db, status: 'connected' as const } : db
      )
    )
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
      {/* 数据库列表 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Database className="h-5 w-5" />
                DuckLake 数据库列表
              </CardTitle>
              <CardDescription>
                管理所有DuckLake数据库的连接和配置
              </CardDescription>
            </div>
            <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
              <DialogTrigger asChild>
                <Button className="flex items-center gap-2">
                  <Plus className="h-4 w-4" />
                  附加数据库
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>附加DuckLake数据库</DialogTitle>
                  <DialogDescription>
                    添加一个新的DuckLake数据库到管理系统中
                  </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                  <div className="grid gap-2">
                    <Label htmlFor="name">数据库名称</Label>
                    <Input
                      id="name"
                      value={newDatabase.name}
                      onChange={(e) => setNewDatabase(prev => ({ ...prev, name: e.target.value }))}
                      placeholder="例如: financial_data"
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="path">数据库路径</Label>
                    <Input
                      id="path"
                      value={newDatabase.path}
                      onChange={(e) => setNewDatabase(prev => ({ ...prev, path: e.target.value }))}
                      placeholder="例如: /data/ducklake/financial.db"
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="description">描述 (可选)</Label>
                    <Input
                      id="description"
                      value={newDatabase.description}
                      onChange={(e) => setNewDatabase(prev => ({ ...prev, description: e.target.value }))}
                      placeholder="数据库用途描述"
                    />
                  </div>
                </div>
                <DialogFooter>
                  <Button variant="outline" onClick={() => setShowAddDialog(false)}>
                    取消
                  </Button>
                  <Button onClick={handleAddDatabase}>
                    附加数据库
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>数据库名称</TableHead>
                <TableHead>状态</TableHead>
                <TableHead>大小</TableHead>
                <TableHead>表数量</TableHead>
                <TableHead>连接数</TableHead>
                <TableHead>最后访问</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {databases.map((db) => (
                <TableRow key={db.id}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{db.name}</div>
                      <div className="text-sm text-gray-500">{db.path}</div>
                      {db.description && (
                        <div className="text-xs text-gray-400 mt-1">{db.description}</div>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>{getStatusBadge(db.status)}</TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <HardDrive className="h-4 w-4 text-gray-400" />
                      {db.size}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <FileText className="h-4 w-4 text-gray-400" />
                      {db.tables}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Users className="h-4 w-4 text-gray-400" />
                      {db.connections}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Clock className="h-4 w-4 text-gray-400" />
                      {db.lastAccessed}
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
                        <DropdownMenuItem>
                          <Eye className="mr-2 h-4 w-4" />
                          查看详情
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Settings className="mr-2 h-4 w-4" />
                          配置
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        {db.status === 'disconnected' ? (
                          <DropdownMenuItem onClick={() => handleConnectDatabase(db.id)}>
                            <Wifi className="mr-2 h-4 w-4" />
                            连接
                          </DropdownMenuItem>
                        ) : (
                          <DropdownMenuItem>
                            <WifiOff className="mr-2 h-4 w-4" />
                            断开连接
                          </DropdownMenuItem>
                        )}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          className="text-red-600"
                          onClick={() => handleDetachDatabase(db.id)}
                        >
                          <Trash2 className="mr-2 h-4 w-4" />
                          分离数据库
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
