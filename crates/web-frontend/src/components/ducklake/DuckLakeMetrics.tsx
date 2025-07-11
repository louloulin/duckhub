import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import {
  Activity,
  Database,
  Clock,
  TrendingUp,
  TrendingDown,
  Zap,
  HardDrive,
  Users,
  GitBranch,
  Layers,
  BarChart3,
} from 'lucide-react'

interface MetricCard {
  title: string
  value: string
  change: string
  trend: 'up' | 'down' | 'stable'
  icon: React.ReactNode
  color: string
}

interface ChartData {
  time: string
  queries: number
  snapshots: number
  transactions: number
}

export default function DuckLakeMetrics() {
  const [metrics, setMetrics] = useState<MetricCard[]>([])
  const [chartData, setChartData] = useState<ChartData[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const loadMetrics = async () => {
      setLoading(true)
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      const mockMetrics: MetricCard[] = [
        {
          title: '时间旅行查询',
          value: '1,234',
          change: '+12.5%',
          trend: 'up',
          icon: <Clock className="h-6 w-6" />,
          color: 'text-blue-600',
        },
        {
          title: '活跃事务',
          value: '45',
          change: '+5.2%',
          trend: 'up',
          icon: <Zap className="h-6 w-6" />,
          color: 'text-green-600',
        },
        {
          title: '快照创建',
          value: '127',
          change: '+8.1%',
          trend: 'up',
          icon: <Layers className="h-6 w-6" />,
          color: 'text-purple-600',
        },
        {
          title: 'Schema演进',
          value: '23',
          change: '-2.3%',
          trend: 'down',
          icon: <GitBranch className="h-6 w-6" />,
          color: 'text-orange-600',
        },
        {
          title: '存储使用',
          value: '8.7 GB',
          change: '+15.8%',
          trend: 'up',
          icon: <HardDrive className="h-6 w-6" />,
          color: 'text-red-600',
        },
        {
          title: '并发连接',
          value: '18',
          change: '+3.4%',
          trend: 'up',
          icon: <Users className="h-6 w-6" />,
          color: 'text-indigo-600',
        },
      ]
      
      const mockChartData: ChartData[] = [
        { time: '00:00', queries: 45, snapshots: 2, transactions: 12 },
        { time: '04:00', queries: 23, snapshots: 1, transactions: 8 },
        { time: '08:00', queries: 89, snapshots: 3, transactions: 25 },
        { time: '12:00', queries: 156, snapshots: 5, transactions: 42 },
        { time: '16:00', queries: 234, snapshots: 4, transactions: 38 },
        { time: '20:00', queries: 178, snapshots: 3, transactions: 29 },
      ]
      
      setMetrics(mockMetrics)
      setChartData(mockChartData)
      setLoading(false)
    }

    loadMetrics()
  }, [])

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up':
        return <TrendingUp className="h-4 w-4 text-green-500" />
      case 'down':
        return <TrendingDown className="h-4 w-4 text-red-500" />
      default:
        return <Activity className="h-4 w-4 text-gray-500" />
    }
  }

  const getTrendColor = (trend: string) => {
    switch (trend) {
      case 'up':
        return 'text-green-600'
      case 'down':
        return 'text-red-600'
      default:
        return 'text-gray-600'
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
      {/* 指标卡片网格 */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {metrics.map((metric, index) => (
          <Card key={index} className="card-hover">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">{metric.title}</p>
                  <p className="text-2xl font-bold text-gray-900">{metric.value}</p>
                </div>
                <div className={`w-12 h-12 bg-gray-100 rounded-lg flex items-center justify-center ${metric.color}`}>
                  {metric.icon}
                </div>
              </div>
              <div className="mt-4 flex items-center text-sm">
                {getTrendIcon(metric.trend)}
                <span className={`ml-1 ${getTrendColor(metric.trend)}`}>
                  {metric.change}
                </span>
                <span className="text-gray-500 ml-1">vs 上周</span>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* 性能图表 */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* 查询趋势图 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="h-5 w-5" />
              24小时活动趋势
            </CardTitle>
            <CardDescription>
              时间旅行查询、快照创建和事务活动
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-64 flex items-end justify-between gap-2 p-4">
              {chartData.map((data, index) => (
                <div key={index} className="flex flex-col items-center gap-2 flex-1">
                  <div className="flex flex-col gap-1 w-full">
                    {/* 查询柱状图 */}
                    <div
                      className="bg-blue-500 rounded-t"
                      style={{
                        height: `${(data.queries / 250) * 100}px`,
                        minHeight: '2px',
                      }}
                      title={`查询: ${data.queries}`}
                    ></div>
                    {/* 快照柱状图 */}
                    <div
                      className="bg-purple-500"
                      style={{
                        height: `${(data.snapshots / 5) * 20}px`,
                        minHeight: '2px',
                      }}
                      title={`快照: ${data.snapshots}`}
                    ></div>
                    {/* 事务柱状图 */}
                    <div
                      className="bg-green-500 rounded-b"
                      style={{
                        height: `${(data.transactions / 50) * 30}px`,
                        minHeight: '2px',
                      }}
                      title={`事务: ${data.transactions}`}
                    ></div>
                  </div>
                  <span className="text-xs text-gray-500">{data.time}</span>
                </div>
              ))}
            </div>
            <div className="flex justify-center gap-6 mt-4 text-sm">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-blue-500 rounded"></div>
                <span>查询</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-purple-500 rounded"></div>
                <span>快照</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-green-500 rounded"></div>
                <span>事务</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* 系统状态 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              系统状态
            </CardTitle>
            <CardDescription>
              DuckLake核心组件运行状态
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {/* 数据库状态 */}
              <div className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                <div className="flex items-center gap-3">
                  <Database className="h-5 w-5 text-blue-600" />
                  <div>
                    <p className="font-medium">数据库引擎</p>
                    <p className="text-sm text-gray-500">DuckDB v0.9.2</p>
                  </div>
                </div>
                <Badge className="bg-green-100 text-green-800">
                  <div className="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                  运行中
                </Badge>
              </div>

              {/* 事务管理器状态 */}
              <div className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                <div className="flex items-center gap-3">
                  <Zap className="h-5 w-5 text-green-600" />
                  <div>
                    <p className="font-medium">事务管理器</p>
                    <p className="text-sm text-gray-500">ACID支持</p>
                  </div>
                </div>
                <Badge className="bg-green-100 text-green-800">
                  <div className="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                  正常
                </Badge>
              </div>

              {/* 快照服务状态 */}
              <div className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                <div className="flex items-center gap-3">
                  <Layers className="h-5 w-5 text-purple-600" />
                  <div>
                    <p className="font-medium">快照服务</p>
                    <p className="text-sm text-gray-500">时间旅行支持</p>
                  </div>
                </div>
                <Badge className="bg-green-100 text-green-800">
                  <div className="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                  活跃
                </Badge>
              </div>

              {/* Schema管理器状态 */}
              <div className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                <div className="flex items-center gap-3">
                  <GitBranch className="h-5 w-5 text-orange-600" />
                  <div>
                    <p className="font-medium">Schema管理器</p>
                    <p className="text-sm text-gray-500">演进支持</p>
                  </div>
                </div>
                <Badge className="bg-green-100 text-green-800">
                  <div className="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                  就绪
                </Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
