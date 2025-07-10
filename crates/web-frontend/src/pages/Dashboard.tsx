import { useEffect } from 'react'
import { useDispatch, useSelector } from 'react-redux'
import { RootState, AppDispatch } from '@/store'
import {
  fetchDashboardMetrics,
  fetchQueryTrends,
  fetchPerformanceData,
  fetchSystemHealth,
} from '@/store/slices/dashboardSlice'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { formatNumber, formatDuration, formatPercentage } from '@/lib/utils'
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from 'recharts'
import {
  Activity,
  Database,
  Clock,
  TrendingUp,
  Users,
  AlertCircle,
} from 'lucide-react'

export default function Dashboard() {
  const dispatch = useDispatch<AppDispatch>()
  const { metrics, queryTrends, performanceData, systemHealth, loading } = useSelector(
    (state: RootState) => state.dashboard
  )

  useEffect(() => {
    // 初始加载数据
    dispatch(fetchDashboardMetrics())
    dispatch(fetchQueryTrends('24h'))
    dispatch(fetchPerformanceData('24h'))
    dispatch(fetchSystemHealth())

    // 设置定时刷新
    const interval = setInterval(() => {
      dispatch(fetchDashboardMetrics())
      dispatch(fetchSystemHealth())
    }, 30000) // 30秒刷新一次

    return () => clearInterval(interval)
  }, [dispatch])

  const metricCards = [
    {
      title: '总查询数',
      value: metrics?.total_queries || 0,
      icon: Database,
      description: '今日执行的查询总数',
      trend: '+12%',
    },
    {
      title: '平均执行时间',
      value: formatDuration(metrics?.avg_execution_time || 0),
      icon: Clock,
      description: '查询平均响应时间',
      trend: '-5%',
    },
    {
      title: '缓存命中率',
      value: formatPercentage(metrics?.cache_hit_rate || 0),
      icon: TrendingUp,
      description: '查询缓存命中率',
      trend: '+8%',
    },
    {
      title: '活跃连接',
      value: metrics?.active_connections || 0,
      icon: Users,
      description: '当前活跃数据库连接',
      trend: '稳定',
    },
  ]

  const systemHealthData = systemHealth ? [
    { name: 'CPU', value: systemHealth.cpu_usage, color: '#8884d8' },
    { name: '内存', value: systemHealth.memory_usage, color: '#82ca9d' },
    { name: '磁盘', value: systemHealth.disk_usage, color: '#ffc658' },
    { name: '网络', value: systemHealth.network_io, color: '#ff7300' },
  ] : []

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">仪表板</h1>
        <p className="text-muted-foreground">
          DuckHub金融数据平台实时监控和分析
        </p>
      </div>

      {/* 指标卡片 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {metricCards.map((metric) => {
          const Icon = metric.icon
          return (
            <Card key={metric.title}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {metric.title}
                </CardTitle>
                <Icon className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{metric.value}</div>
                <p className="text-xs text-muted-foreground">
                  {metric.description}
                </p>
                <div className="mt-2 flex items-center text-xs">
                  <span className="text-green-600">{metric.trend}</span>
                  <span className="ml-1 text-muted-foreground">较昨日</span>
                </div>
              </CardContent>
            </Card>
          )
        })}
      </div>

      {/* 图表区域 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        {/* 查询趋势图 */}
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>查询趋势</CardTitle>
            <CardDescription>
              过去24小时的查询执行趋势
            </CardDescription>
          </CardHeader>
          <CardContent className="pl-2">
            <ResponsiveContainer width="100%" height={350}>
              <AreaChart data={queryTrends}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis
                  dataKey="timestamp"
                  stroke="#888888"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis
                  stroke="#888888"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(value) => formatNumber(value)}
                />
                <Tooltip />
                <Area
                  type="monotone"
                  dataKey="value"
                  stroke="#8884d8"
                  fill="#8884d8"
                  fillOpacity={0.6}
                />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* 系统健康状态 */}
        <Card className="col-span-3">
          <CardHeader>
            <CardTitle>系统健康状态</CardTitle>
            <CardDescription>
              实时系统资源使用情况
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={350}>
              <PieChart>
                <Pie
                  data={systemHealthData}
                  cx="50%"
                  cy="50%"
                  innerRadius={60}
                  outerRadius={120}
                  paddingAngle={5}
                  dataKey="value"
                >
                  {systemHealthData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip formatter={(value) => `${value}%`} />
              </PieChart>
            </ResponsiveContainer>
            <div className="mt-4 grid grid-cols-2 gap-4">
              {systemHealthData.map((item) => (
                <div key={item.name} className="flex items-center">
                  <div
                    className="h-3 w-3 rounded-full mr-2"
                    style={{ backgroundColor: item.color }}
                  />
                  <span className="text-sm">{item.name}: {item.value}%</span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 性能分析 */}
      <div className="grid gap-4 md:grid-cols-2">
        {/* 查询性能分布 */}
        <Card>
          <CardHeader>
            <CardTitle>查询性能分布</CardTitle>
            <CardDescription>
              不同执行时间范围的查询分布
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={performanceData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis
                  dataKey="label"
                  stroke="#888888"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis
                  stroke="#888888"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <Tooltip />
                <Bar dataKey="value" fill="#8884d8" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* 系统警告 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <AlertCircle className="h-5 w-5" />
              系统警告
            </CardTitle>
            <CardDescription>
              需要关注的系统状态和建议
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-start gap-3 p-3 rounded-lg bg-yellow-50 border border-yellow-200">
                <AlertCircle className="h-4 w-4 text-yellow-600 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-yellow-800">
                    查询缓存使用率较低
                  </p>
                  <p className="text-xs text-yellow-700">
                    建议优化查询缓存策略以提高性能
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 rounded-lg bg-blue-50 border border-blue-200">
                <Activity className="h-4 w-4 text-blue-600 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-blue-800">
                    数据量增长趋势
                  </p>
                  <p className="text-xs text-blue-700">
                    数据存储量持续增长，建议考虑数据归档策略
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 rounded-lg bg-green-50 border border-green-200">
                <TrendingUp className="h-4 w-4 text-green-600 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-green-800">
                    系统运行稳定
                  </p>
                  <p className="text-xs text-green-700">
                    所有核心服务运行正常，性能指标良好
                  </p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
