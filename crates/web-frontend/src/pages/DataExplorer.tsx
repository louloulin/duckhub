import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
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
  Database,
  Table as TableIcon,
  BarChart3,
  TrendingUp,
  GitBranch,
  Plus,
  Minus,
  Edit,
  History,
  AlertTriangle,
  CheckCircle,
  Clock,
  User,
  FileText,
  Eye,
  Trash2,
  Download,
  Search,
} from 'lucide-react'

interface SchemaColumn {
  column: string
  type: string
  nullable: boolean
  key: string
  default_value?: string
  comment?: string
}

interface SchemaVersion {
  version: number
  timestamp: string
  author: string
  description: string
  changes: SchemaChange[]
  compatibility: 'backward' | 'forward' | 'breaking' | 'full'
}

interface SchemaChange {
  type: 'add_column' | 'drop_column' | 'modify_column' | 'add_index' | 'drop_index'
  table: string
  column?: string
  old_definition?: string
  new_definition?: string
  description: string
  impact: 'low' | 'medium' | 'high'
}

interface TableInfo {
  name: string
  rows: number
  size: string
  schema_version: number
  last_modified: string
  description?: string
}

export default function DataExplorer() {
  const [selectedTable, setSelectedTable] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState('tables')

  const [showAddColumnDialog, setShowAddColumnDialog] = useState(false)
  const [selectedSchemaVersions, setSelectedSchemaVersions] = useState<number[]>([])
  const [newColumn, setNewColumn] = useState({
    name: '',
    type: 'VARCHAR(255)',
    nullable: true,
    default_value: '',
    comment: '',
  })

  // 模拟表数据
  const tables: TableInfo[] = [
    {
      name: 'transactions',
      rows: 1250000,
      size: '2.3 GB',
      schema_version: 5,
      last_modified: '2024-01-11 14:30:25',
      description: '交易记录表'
    },
    {
      name: 'users',
      rows: 45000,
      size: '120 MB',
      schema_version: 3,
      last_modified: '2024-01-10 09:15:10',
      description: '用户信息表'
    },
    {
      name: 'products',
      rows: 8500,
      size: '45 MB',
      schema_version: 2,
      last_modified: '2024-01-09 16:20:30',
      description: '产品信息表'
    },
    {
      name: 'orders',
      rows: 890000,
      size: '1.8 GB',
      schema_version: 4,
      last_modified: '2024-01-11 12:45:15',
      description: '订单记录表'
    },
  ]

  const tableSchema: SchemaColumn[] = selectedTable ? [
    { column: 'id', type: 'BIGINT', nullable: false, key: 'PRIMARY', comment: '主键ID' },
    { column: 'user_id', type: 'BIGINT', nullable: false, key: 'FOREIGN', comment: '用户ID' },
    { column: 'amount', type: 'DECIMAL(10,2)', nullable: false, key: '', comment: '交易金额' },
    { column: 'status', type: 'VARCHAR(50)', nullable: false, key: '', default_value: 'pending', comment: '交易状态' },
    { column: 'created_at', type: 'TIMESTAMP', nullable: false, key: '', default_value: 'CURRENT_TIMESTAMP', comment: '创建时间' },
    { column: 'updated_at', type: 'TIMESTAMP', nullable: true, key: '', comment: '更新时间' },
  ] : []

  // 模拟Schema演进历史
  const schemaVersions: SchemaVersion[] = [
    {
      version: 5,
      timestamp: '2024-01-11 14:30:25',
      author: 'admin',
      description: '添加交易状态字段',
      compatibility: 'backward',
      changes: [
        {
          type: 'add_column',
          table: 'transactions',
          column: 'status',
          new_definition: 'VARCHAR(50) NOT NULL DEFAULT "pending"',
          description: '添加交易状态字段，支持pending/completed/failed状态',
          impact: 'low'
        }
      ]
    },
    {
      version: 4,
      timestamp: '2024-01-10 16:20:15',
      author: 'developer',
      description: '修改金额字段精度',
      compatibility: 'breaking',
      changes: [
        {
          type: 'modify_column',
          table: 'transactions',
          column: 'amount',
          old_definition: 'DECIMAL(8,2)',
          new_definition: 'DECIMAL(10,2)',
          description: '增加金额字段精度以支持更大金额',
          impact: 'medium'
        }
      ]
    },
    {
      version: 3,
      timestamp: '2024-01-09 10:15:30',
      author: 'dba',
      description: '添加索引优化查询性能',
      compatibility: 'full',
      changes: [
        {
          type: 'add_index',
          table: 'transactions',
          column: 'user_id',
          new_definition: 'INDEX idx_user_id (user_id)',
          description: '为user_id字段添加索引',
          impact: 'low'
        }
      ]
    },
  ]

  // 辅助函数
  const getCompatibilityBadge = (compatibility: string) => {
    const configs = {
      backward: { label: '向后兼容', className: 'bg-green-100 text-green-800' },
      forward: { label: '向前兼容', className: 'bg-blue-100 text-blue-800' },
      breaking: { label: '破坏性变更', className: 'bg-red-100 text-red-800' },
      full: { label: '完全兼容', className: 'bg-purple-100 text-purple-800' },
    }
    const config = configs[compatibility as keyof typeof configs] || configs.backward
    return (
      <Badge className={config.className}>
        {config.label}
      </Badge>
    )
  }

  const getImpactBadge = (impact: string) => {
    const configs = {
      low: { label: '低影响', className: 'bg-green-100 text-green-800' },
      medium: { label: '中等影响', className: 'bg-yellow-100 text-yellow-800' },
      high: { label: '高影响', className: 'bg-red-100 text-red-800' },
    }
    const config = configs[impact as keyof typeof configs] || configs.low
    return (
      <Badge variant="outline" className={config.className}>
        {config.label}
      </Badge>
    )
  }

  const getChangeTypeIcon = (type: string) => {
    switch (type) {
      case 'add_column':
        return <Plus className="h-4 w-4 text-green-600" />
      case 'drop_column':
        return <Minus className="h-4 w-4 text-red-600" />
      case 'modify_column':
        return <Edit className="h-4 w-4 text-blue-600" />
      case 'add_index':
        return <Plus className="h-4 w-4 text-purple-600" />
      case 'drop_index':
        return <Minus className="h-4 w-4 text-orange-600" />
      default:
        return <FileText className="h-4 w-4 text-gray-600" />
    }
  }

  const handleAddColumn = () => {
    // 模拟添加列
    console.log('添加列:', newColumn)
    setShowAddColumnDialog(false)
    setNewColumn({
      name: '',
      type: 'VARCHAR(255)',
      nullable: true,
      default_value: '',
      comment: '',
    })
  }

  const toggleSchemaVersionSelection = (version: number) => {
    setSelectedSchemaVersions(prev => {
      if (prev.includes(version)) {
        return prev.filter(v => v !== version)
      } else if (prev.length < 2) {
        return [...prev, version]
      } else {
        return [prev[1], version] // 替换第一个选择
      }
    })
  }

  return (
    <div className="space-y-6">
      {/* 页面头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-r from-green-600 to-green-500 rounded-xl flex items-center justify-center">
              <Database className="h-6 w-6 text-white" />
            </div>
            <span className="gradient-text">数据探索</span>
          </h1>
          <p className="text-gray-600 mt-2">
            浏览数据库表结构、数据内容和Schema演进历史
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" className="flex items-center gap-2">
            <Search className="h-4 w-4" />
            搜索表
          </Button>
          <Button className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            数据分析
          </Button>
        </div>
      </div>

      {/* 主要内容标签页 */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="tables" className="flex items-center gap-2">
            <TableIcon className="h-4 w-4" />
            数据表
          </TabsTrigger>
          <TabsTrigger value="schema" className="flex items-center gap-2">
            <GitBranch className="h-4 w-4" />
            Schema演进
          </TabsTrigger>
          <TabsTrigger value="analysis" className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            数据分析
          </TabsTrigger>
        </TabsList>

        <TabsContent value="tables" className="space-y-6">

          <div className="grid gap-6 md:grid-cols-3">
            {/* 表列表 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Database className="h-5 w-5" />
                  数据表
                </CardTitle>
                <CardDescription>
                  选择要探索的数据表
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {tables.map((table) => (
                    <div
                      key={table.name}
                      className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                        selectedTable === table.name
                          ? 'bg-blue-50 border-blue-200'
                          : 'hover:bg-gray-50'
                      }`}
                      onClick={() => setSelectedTable(table.name)}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <TableIcon className="h-4 w-4" />
                          <span className="font-medium">{table.name}</span>
                        </div>
                        <Badge variant="outline" className="text-xs">
                          v{table.schema_version}
                        </Badge>
                      </div>
                      <div className="mt-1 text-xs text-gray-500">
                        {table.rows.toLocaleString()} 行 • {table.size}
                      </div>
                      {table.description && (
                        <div className="mt-1 text-xs text-gray-400">
                          {table.description}
                        </div>
                      )}
                      <div className="mt-1 text-xs text-gray-400">
                        最后修改: {table.last_modified}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* 表结构 */}
            <Card className="md:col-span-2">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <TableIcon className="h-5 w-5" />
                      表结构
                      {selectedTable && <span className="text-muted-foreground">- {selectedTable}</span>}
                    </CardTitle>
                    <CardDescription>
                      {selectedTable ? '查看表的列定义和约束' : '选择一个表查看其结构'}
                    </CardDescription>
                  </div>
                  {selectedTable && (
                    <div className="flex items-center gap-2">
                      <Dialog open={showAddColumnDialog} onOpenChange={setShowAddColumnDialog}>
                        <DialogTrigger asChild>
                          <Button size="sm" className="flex items-center gap-2">
                            <Plus className="h-4 w-4" />
                            添加列
                          </Button>
                        </DialogTrigger>
                        <DialogContent>
                          <DialogHeader>
                            <DialogTitle>添加新列</DialogTitle>
                            <DialogDescription>
                              为 {selectedTable} 表添加新的列
                            </DialogDescription>
                          </DialogHeader>
                          <div className="grid gap-4 py-4">
                            <div className="grid gap-2">
                              <Label htmlFor="columnName">列名</Label>
                              <Input
                                id="columnName"
                                value={newColumn.name}
                                onChange={(e) => setNewColumn(prev => ({ ...prev, name: e.target.value }))}
                                placeholder="例如: email"
                              />
                            </div>
                            <div className="grid gap-2">
                              <Label htmlFor="columnType">数据类型</Label>
                              <select
                                id="columnType"
                                value={newColumn.type}
                                onChange={(e) => setNewColumn(prev => ({ ...prev, type: e.target.value }))}
                                className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                              >
                                <option value="VARCHAR(255)">VARCHAR(255)</option>
                                <option value="TEXT">TEXT</option>
                                <option value="INT">INT</option>
                                <option value="BIGINT">BIGINT</option>
                                <option value="DECIMAL(10,2)">DECIMAL(10,2)</option>
                                <option value="BOOLEAN">BOOLEAN</option>
                                <option value="TIMESTAMP">TIMESTAMP</option>
                                <option value="DATE">DATE</option>
                              </select>
                            </div>
                            <div className="flex items-center space-x-2">
                              <input
                                type="checkbox"
                                id="nullable"
                                checked={newColumn.nullable}
                                onChange={(e) => setNewColumn(prev => ({ ...prev, nullable: e.target.checked }))}
                                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                              />
                              <Label htmlFor="nullable">允许为空</Label>
                            </div>
                            <div className="grid gap-2">
                              <Label htmlFor="defaultValue">默认值</Label>
                              <Input
                                id="defaultValue"
                                value={newColumn.default_value}
                                onChange={(e) => setNewColumn(prev => ({ ...prev, default_value: e.target.value }))}
                                placeholder="例如: NULL, '', 0"
                              />
                            </div>
                            <div className="grid gap-2">
                              <Label htmlFor="comment">注释</Label>
                              <Input
                                id="comment"
                                value={newColumn.comment}
                                onChange={(e) => setNewColumn(prev => ({ ...prev, comment: e.target.value }))}
                                placeholder="列的描述信息"
                              />
                            </div>
                          </div>
                          <DialogFooter>
                            <Button variant="outline" onClick={() => setShowAddColumnDialog(false)}>
                              取消
                            </Button>
                            <Button onClick={handleAddColumn}>
                              添加列
                            </Button>
                          </DialogFooter>
                        </DialogContent>
                      </Dialog>

                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setActiveTab('schema')}
                        className="flex items-center gap-2"
                      >
                        <History className="h-4 w-4" />
                        演进历史
                      </Button>
                    </div>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                {selectedTable ? (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>列名</TableHead>
                        <TableHead>数据类型</TableHead>
                        <TableHead>可空</TableHead>
                        <TableHead>默认值</TableHead>
                        <TableHead>约束</TableHead>
                        <TableHead>注释</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {tableSchema.map((column, index) => (
                        <TableRow key={index}>
                          <TableCell className="font-mono font-medium">
                            {column.column}
                          </TableCell>
                          <TableCell>
                            <Badge variant="outline">{column.type}</Badge>
                          </TableCell>
                          <TableCell>
                            {column.nullable ? (
                              <CheckCircle className="h-4 w-4 text-green-600" />
                            ) : (
                              <AlertTriangle className="h-4 w-4 text-red-600" />
                            )}
                          </TableCell>
                          <TableCell className="font-mono text-sm">
                            {column.default_value || '-'}
                          </TableCell>
                          <TableCell>
                            {column.key && (
                              <Badge className={
                                column.key === 'PRIMARY' ? 'bg-blue-100 text-blue-800' :
                                column.key === 'FOREIGN' ? 'bg-purple-100 text-purple-800' :
                                'bg-gray-100 text-gray-800'
                              }>
                                {column.key}
                              </Badge>
                            )}
                          </TableCell>
                          <TableCell className="text-sm text-gray-600">
                            {column.comment || '-'}
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex items-center gap-1 justify-end">
                              <Button variant="ghost" size="sm">
                                <Edit className="h-3 w-3" />
                              </Button>
                              <Button variant="ghost" size="sm">
                                <Trash2 className="h-3 w-3" />
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                ) : (
                  <div className="text-center py-8 text-muted-foreground">
                    请从左侧选择一个表查看其结构
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Schema演进标签页 */}
        <TabsContent value="schema" className="space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <GitBranch className="h-5 w-5" />
                    Schema演进历史
                  </CardTitle>
                  <CardDescription>
                    查看数据库Schema的变更历史和兼容性分析
                  </CardDescription>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    disabled={selectedSchemaVersions.length !== 2}
                    className="flex items-center gap-2"
                  >
                    <GitBranch className="h-4 w-4" />
                    比较版本 {selectedSchemaVersions.length > 0 && `(${selectedSchemaVersions.length})`}
                  </Button>
                  <Button className="flex items-center gap-2">
                    <Plus className="h-4 w-4" />
                    新建变更
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {schemaVersions.map((version) => (
                  <div
                    key={version.version}
                    className={`p-4 border rounded-lg transition-colors ${
                      selectedSchemaVersions.includes(version.version)
                        ? 'bg-blue-50 border-blue-200'
                        : 'hover:bg-gray-50'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3">
                        <input
                          type="checkbox"
                          checked={selectedSchemaVersions.includes(version.version)}
                          onChange={() => toggleSchemaVersionSelection(version.version)}
                          className="mt-1 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                        />
                        <div className="flex-1">
                          <div className="flex items-center gap-3 mb-2">
                            <Badge variant="outline" className="font-mono">
                              v{version.version}
                            </Badge>
                            {getCompatibilityBadge(version.compatibility)}
                            <div className="flex items-center gap-1 text-sm text-gray-500">
                              <User className="h-3 w-3" />
                              {version.author}
                            </div>
                            <div className="flex items-center gap-1 text-sm text-gray-500">
                              <Clock className="h-3 w-3" />
                              {version.timestamp}
                            </div>
                          </div>

                          <h3 className="font-medium text-gray-900 mb-2">{version.description}</h3>

                          <div className="space-y-2">
                            {version.changes.map((change, index) => (
                              <div key={index} className="flex items-center gap-3 p-2 bg-white rounded border">
                                {getChangeTypeIcon(change.type)}
                                <div className="flex-1">
                                  <div className="flex items-center gap-2 mb-1">
                                    <span className="text-sm font-medium">{change.table}</span>
                                    {change.column && (
                                      <Badge variant="outline" className="text-xs">
                                        {change.column}
                                      </Badge>
                                    )}
                                    {getImpactBadge(change.impact)}
                                  </div>
                                  <p className="text-sm text-gray-600">{change.description}</p>
                                  {change.old_definition && change.new_definition && (
                                    <div className="mt-1 text-xs">
                                      <div className="text-red-600">- {change.old_definition}</div>
                                      <div className="text-green-600">+ {change.new_definition}</div>
                                    </div>
                                  )}
                                  {change.new_definition && !change.old_definition && (
                                    <div className="mt-1 text-xs text-green-600">
                                      + {change.new_definition}
                                    </div>
                                  )}
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center gap-1">
                        <Button variant="ghost" size="sm">
                          <Eye className="h-3 w-3" />
                        </Button>
                        <Button variant="ghost" size="sm">
                          <Download className="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 数据分析标签页 */}
        <TabsContent value="analysis" className="space-y-6">
          <div className="grid gap-6 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>数据质量分析</CardTitle>
                <CardDescription>
                  分析数据完整性和质量指标
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-blue-50 rounded-lg">
                      <div className="text-sm text-blue-600">数据完整性</div>
                      <div className="text-2xl font-bold text-blue-900">98.5%</div>
                    </div>
                    <div className="p-3 bg-green-50 rounded-lg">
                      <div className="text-sm text-green-600">数据一致性</div>
                      <div className="text-2xl font-bold text-green-900">99.2%</div>
                    </div>
                  </div>
                  <Button className="w-full">
                    <BarChart3 className="mr-2 h-4 w-4" />
                    生成详细报告
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Schema兼容性分析</CardTitle>
                <CardDescription>
                  分析Schema变更的兼容性影响
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-green-50 rounded-lg">
                      <div className="text-sm text-green-600">兼容变更</div>
                      <div className="text-2xl font-bold text-green-900">85%</div>
                    </div>
                    <div className="p-3 bg-red-50 rounded-lg">
                      <div className="text-sm text-red-600">破坏性变更</div>
                      <div className="text-2xl font-bold text-red-900">15%</div>
                    </div>
                  </div>
                  <Button className="w-full" variant="outline">
                    <TrendingUp className="mr-2 h-4 w-4" />
                    兼容性趋势分析
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>

      {/* 数据预览和分析 */}
      {selectedTable && (
        <div className="grid gap-6 md:grid-cols-2">
          {/* 数据预览 */}
          <Card>
            <CardHeader>
              <CardTitle>数据预览</CardTitle>
              <CardDescription>
                {selectedTable} 表的前几行数据
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="overflow-x-auto">
                  <table className="w-full border-collapse border border-gray-300 text-sm">
                    <thead>
                      <tr className="bg-gray-50">
                        <th className="border border-gray-300 px-2 py-1 text-left">id</th>
                        <th className="border border-gray-300 px-2 py-1 text-left">user_id</th>
                        <th className="border border-gray-300 px-2 py-1 text-left">amount</th>
                        <th className="border border-gray-300 px-2 py-1 text-left">status</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr className="hover:bg-gray-50">
                        <td className="border border-gray-300 px-2 py-1">1001</td>
                        <td className="border border-gray-300 px-2 py-1">2001</td>
                        <td className="border border-gray-300 px-2 py-1">1250.00</td>
                        <td className="border border-gray-300 px-2 py-1">completed</td>
                      </tr>
                      <tr className="hover:bg-gray-50">
                        <td className="border border-gray-300 px-2 py-1">1002</td>
                        <td className="border border-gray-300 px-2 py-1">2002</td>
                        <td className="border border-gray-300 px-2 py-1">890.50</td>
                        <td className="border border-gray-300 px-2 py-1">pending</td>
                      </tr>
                      <tr className="hover:bg-gray-50">
                        <td className="border border-gray-300 px-2 py-1">1003</td>
                        <td className="border border-gray-300 px-2 py-1">2003</td>
                        <td className="border border-gray-300 px-2 py-1">2100.75</td>
                        <td className="border border-gray-300 px-2 py-1">completed</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
                <div className="flex gap-2">
                  <Button size="sm" variant="outline">
                    <BarChart3 className="h-4 w-4 mr-2" />
                    生成统计
                  </Button>
                  <Button size="sm" variant="outline">
                    <TrendingUp className="h-4 w-4 mr-2" />
                    趋势分析
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* 表统计信息 */}
          <Card>
            <CardHeader>
              <CardTitle>统计信息</CardTitle>
              <CardDescription>
                {selectedTable} 表的数据分布和质量指标
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="p-3 bg-blue-50 rounded-lg">
                    <div className="text-sm text-blue-600">总行数</div>
                    <div className="text-2xl font-bold text-blue-900">1,250,000</div>
                  </div>
                  <div className="p-3 bg-green-50 rounded-lg">
                    <div className="text-sm text-green-600">数据完整性</div>
                    <div className="text-2xl font-bold text-green-900">98.5%</div>
                  </div>
                  <div className="p-3 bg-yellow-50 rounded-lg">
                    <div className="text-sm text-yellow-600">重复记录</div>
                    <div className="text-2xl font-bold text-yellow-900">0.2%</div>
                  </div>
                  <div className="p-3 bg-purple-50 rounded-lg">
                    <div className="text-sm text-purple-600">最后更新</div>
                    <div className="text-sm font-bold text-purple-900">2小时前</div>
                  </div>
                </div>
                <div className="space-y-2">
                  <h4 className="font-medium">数据质量建议</h4>
                  <ul className="text-sm text-muted-foreground space-y-1">
                    <li>• 建议为 user_id 列添加索引以提高查询性能</li>
                    <li>• amount 列存在少量异常值，建议进行数据清洗</li>
                    <li>• created_at 列可用于时间序列分析</li>
                  </ul>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
