import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Database, Table, BarChart3, TrendingUp } from 'lucide-react'

export default function DataExplorer() {
  const [selectedTable, setSelectedTable] = useState<string | null>(null)

  // 模拟表数据
  const tables = [
    { name: 'transactions', rows: 1250000, size: '2.3 GB' },
    { name: 'users', rows: 45000, size: '120 MB' },
    { name: 'products', rows: 8500, size: '45 MB' },
    { name: 'orders', rows: 890000, size: '1.8 GB' },
  ]

  const tableSchema = selectedTable ? [
    { column: 'id', type: 'BIGINT', nullable: false, key: 'PRIMARY' },
    { column: 'user_id', type: 'BIGINT', nullable: false, key: 'FOREIGN' },
    { column: 'amount', type: 'DECIMAL(10,2)', nullable: false, key: '' },
    { column: 'created_at', type: 'TIMESTAMP', nullable: false, key: '' },
    { column: 'status', type: 'VARCHAR(20)', nullable: false, key: '' },
  ] : []

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">数据探索</h1>
        <p className="text-muted-foreground">
          浏览数据库表结构和数据内容
        </p>
      </div>

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
                      ? 'bg-primary text-primary-foreground'
                      : 'hover:bg-accent'
                  }`}
                  onClick={() => setSelectedTable(table.name)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Table className="h-4 w-4" />
                      <span className="font-medium">{table.name}</span>
                    </div>
                  </div>
                  <div className="mt-1 text-xs opacity-75">
                    {table.rows.toLocaleString()} 行 • {table.size}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* 表结构 */}
        <Card className="md:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Table className="h-5 w-5" />
              表结构
              {selectedTable && <span className="text-muted-foreground">- {selectedTable}</span>}
            </CardTitle>
            <CardDescription>
              {selectedTable ? '查看表的列定义和约束' : '选择一个表查看其结构'}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {selectedTable ? (
              <div className="overflow-x-auto">
                <table className="w-full border-collapse border border-gray-300">
                  <thead>
                    <tr className="bg-gray-50">
                      <th className="border border-gray-300 px-4 py-2 text-left">列名</th>
                      <th className="border border-gray-300 px-4 py-2 text-left">数据类型</th>
                      <th className="border border-gray-300 px-4 py-2 text-left">可空</th>
                      <th className="border border-gray-300 px-4 py-2 text-left">约束</th>
                    </tr>
                  </thead>
                  <tbody>
                    {tableSchema.map((column, index) => (
                      <tr key={index} className="hover:bg-gray-50">
                        <td className="border border-gray-300 px-4 py-2 font-mono">
                          {column.column}
                        </td>
                        <td className="border border-gray-300 px-4 py-2">
                          {column.type}
                        </td>
                        <td className="border border-gray-300 px-4 py-2">
                          {column.nullable ? '是' : '否'}
                        </td>
                        <td className="border border-gray-300 px-4 py-2">
                          {column.key && (
                            <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded">
                              {column.key}
                            </span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                请从左侧选择一个表查看其结构
              </div>
            )}
          </CardContent>
        </Card>
      </div>

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
