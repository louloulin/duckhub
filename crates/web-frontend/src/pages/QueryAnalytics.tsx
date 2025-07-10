import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Play, Save, History, Zap } from 'lucide-react'

export default function QueryAnalytics() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<any>(null)
  const [loading, setLoading] = useState(false)

  const handleExecuteQuery = async () => {
    if (!query.trim()) return
    
    setLoading(true)
    try {
      // 这里会调用API执行查询
      // const result = await queryAPI.execute(query)
      // setResults(result.data)
      
      // 模拟数据
      setTimeout(() => {
        setResults({
          query_id: 'q_' + Date.now(),
          execution_time_ms: 150,
          row_count: 1250,
          optimized: true,
          cache_hit: false,
          data: [
            { id: 1, name: '示例数据1', value: 100 },
            { id: 2, name: '示例数据2', value: 200 },
            { id: 3, name: '示例数据3', value: 300 },
          ]
        })
        setLoading(false)
      }, 1000)
    } catch (error) {
      console.error('查询执行失败:', error)
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">查询分析</h1>
        <p className="text-muted-foreground">
          执行SQL查询并分析性能指标
        </p>
      </div>

      {/* 查询编辑器 */}
      <Card>
        <CardHeader>
          <CardTitle>SQL查询编辑器</CardTitle>
          <CardDescription>
            输入SQL查询语句，支持DuckDB语法和窗口函数
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <textarea
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="输入您的SQL查询..."
              className="w-full h-40 p-3 border rounded-md font-mono text-sm resize-none focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <div className="flex gap-2">
              <Button onClick={handleExecuteQuery} disabled={loading || !query.trim()}>
                <Play className="h-4 w-4 mr-2" />
                {loading ? '执行中...' : '执行查询'}
              </Button>
              <Button variant="outline">
                <Save className="h-4 w-4 mr-2" />
                保存查询
              </Button>
              <Button variant="outline">
                <History className="h-4 w-4 mr-2" />
                查询历史
              </Button>
              <Button variant="outline">
                <Zap className="h-4 w-4 mr-2" />
                查询优化
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 查询结果 */}
      {results && (
        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">执行时间</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{results.execution_time_ms}ms</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">返回行数</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{results.row_count.toLocaleString()}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">查询优化</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {results.optimized ? '✓' : '✗'}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">缓存命中</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {results.cache_hit ? '✓' : '✗'}
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* 数据表格 */}
      {results && results.data && (
        <Card>
          <CardHeader>
            <CardTitle>查询结果</CardTitle>
            <CardDescription>
              共 {results.row_count} 行数据
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="overflow-x-auto">
              <table className="w-full border-collapse border border-gray-300">
                <thead>
                  <tr className="bg-gray-50">
                    {Object.keys(results.data[0] || {}).map((key) => (
                      <th key={key} className="border border-gray-300 px-4 py-2 text-left">
                        {key}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {results.data.slice(0, 10).map((row: any, index: number) => (
                    <tr key={index} className="hover:bg-gray-50">
                      {Object.values(row).map((value: any, cellIndex: number) => (
                        <td key={cellIndex} className="border border-gray-300 px-4 py-2">
                          {String(value)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
              {results.data.length > 10 && (
                <div className="mt-4 text-center text-sm text-muted-foreground">
                  显示前10行，共{results.row_count}行数据
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
