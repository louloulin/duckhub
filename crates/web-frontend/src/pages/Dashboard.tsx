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
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
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
  const { metrics, queryTrends, performanceData, systemHealth } = useSelector(
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
    <div className="space-y-8 fade-in">
      {/* 欢迎区域 - 简化版 */}
      <div className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-2xl p-8 border border-blue-100 card-hover">
        <div className="flex items-center justify-between">
          <div className="slide-in-left">
            <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
              <span className="gradient-text">
                Hi, Welcome back
              </span>
              <span className="text-2xl float">👋</span>
            </h1>
            <p className="text-gray-600 mt-2 text-lg">
              DuckHub金融数据平台 - 智能分析与实时监控
            </p>
          </div>
          <div className="flex items-center space-x-4 scale-in">
            <div className="text-sm text-gray-500 glass px-4 py-2 rounded-xl">
              <span className="text-gray-700">最后更新:</span> {new Date().toLocaleTimeString('zh-CN')}
            </div>
          </div>
        </div>
      </div>

      <div className="space-y-8">

        {/* 核心指标卡片 - 现代白色风格 */}
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          {metricCards.map((metric, index) => {
            const Icon = metric.icon
            const trendColor = metric.trend.startsWith('+') ? 'text-green-600' :
                              metric.trend.startsWith('-') ? 'text-red-600' : 'text-gray-600'
            const cardColors = [
              { bg: 'bg-gradient-to-br from-blue-50 to-blue-100', border: 'border-blue-200', icon: 'bg-blue-500', text: 'text-blue-900' },
              { bg: 'bg-gradient-to-br from-green-50 to-green-100', border: 'border-green-200', icon: 'bg-green-500', text: 'text-green-900' },
              { bg: 'bg-gradient-to-br from-yellow-50 to-yellow-100', border: 'border-yellow-200', icon: 'bg-yellow-500', text: 'text-yellow-900' },
              { bg: 'bg-gradient-to-br from-purple-50 to-purple-100', border: 'border-purple-200', icon: 'bg-purple-500', text: 'text-purple-900' }
            ]
            const colors = cardColors[index]

            return (
              <Card
                key={metric.title}
                className={`${colors.bg} ${colors.border} border-2 shadow-lg card-hover scale-in`}
                style={{ animationDelay: `${index * 0.1}s` }}
              >
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
                  <CardTitle className={`text-sm font-semibold ${colors.text}`}>
                    {metric.title}
                  </CardTitle>
                  <div className={`p-3 rounded-xl ${colors.icon} shadow-lg float`}>
                    <Icon className="h-5 w-5 text-white" />
                  </div>
                </CardHeader>
                <CardContent>
                  <div className={`text-3xl font-bold ${colors.text} mb-2`}>
                    {metric.value}
                  </div>
                  <p className="text-sm text-gray-600 mb-4">
                    {metric.description}
                  </p>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center text-sm">
                      <span className={`font-semibold ${trendColor}`}>{metric.trend}</span>
                      <span className="ml-2 text-gray-500">较昨日</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <div className="w-2 h-2 bg-green-500 rounded-full pulse-slow"></div>
                      <span className="text-xs text-gray-500">实时</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </div>

        {/* 主要图表区域 - 现代白色风格 */}
        <div className="grid gap-6 lg:grid-cols-3">
          {/* 查询趋势图 - 占据更大空间 */}
          <Card className="lg:col-span-2 bg-white border border-gray-200 shadow-lg card-hover fade-in">
            <CardHeader className="pb-6">
              <div className="flex items-center justify-between">
                <div className="slide-in-left">
                  <CardTitle className="text-xl font-bold text-gray-900 flex items-center gap-2">
                    <span className="gradient-text">
                      查询趋势分析
                    </span>
                  </CardTitle>
                  <CardDescription className="text-gray-600 mt-2">
                    过去24小时的查询执行趋势 - 实时监控与智能分析
                  </CardDescription>
                </div>
                <div className="flex items-center space-x-4 scale-in">
                  <div className="flex items-center space-x-2 bg-blue-50 px-3 py-1.5 rounded-lg border border-blue-200">
                    <div className="w-3 h-3 bg-blue-500 rounded-full pulse-slow"></div>
                    <span className="text-sm font-medium text-blue-700">查询量</span>
                  </div>
                  <div className="text-sm text-gray-500 glass px-3 py-1.5 rounded-lg">
                    24小时
                  </div>
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-4">
              <ResponsiveContainer width="100%" height={350}>
                <AreaChart data={queryTrends} margin={{ top: 20, right: 30, left: 0, bottom: 0 }}>
                  <defs>
                    <linearGradient id="colorQuery" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
                      <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.05}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
                  <XAxis
                    dataKey="timestamp"
                    stroke="#64748b"
                    fontSize={12}
                    tickLine={false}
                    axisLine={false}
                  />
                  <YAxis
                    stroke="#64748b"
                    fontSize={12}
                    tickLine={false}
                    axisLine={false}
                    tickFormatter={(value) => formatNumber(value)}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'white',
                      border: '1px solid #e2e8f0',
                      borderRadius: '12px',
                      boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
                      color: '#1f2937'
                    }}
                    labelStyle={{ color: '#3b82f6' }}
                  />
                  <Area
                    type="monotone"
                    dataKey="value"
                    stroke="#3b82f6"
                    strokeWidth={2}
                    fill="url(#colorQuery)"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          {/* 系统健康状态 */}
          <Card className="bg-white border border-gray-200 shadow-lg card-hover fade-in">
            <CardHeader className="pb-6">
              <div className="flex items-center justify-between">
                <div className="slide-in-left">
                  <CardTitle className="text-xl font-bold text-gray-900 flex items-center gap-2">
                    <span className="bg-gradient-to-r from-green-600 to-green-500 bg-clip-text text-transparent">
                      系统健康状态
                    </span>
                  </CardTitle>
                  <CardDescription className="text-gray-600 mt-2">
                    实时系统资源使用情况与性能监控
                  </CardDescription>
                </div>
                <div className="flex items-center space-x-2 bg-green-50 px-3 py-1.5 rounded-lg border border-green-200 scale-in">
                  <div className="w-3 h-3 bg-green-500 rounded-full pulse-slow"></div>
                  <span className="text-sm font-semibold text-green-700">系统正常</span>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              {/* 系统资源列表 */}
              <div className="space-y-4">
                {systemHealthData.map((item) => {
                  const percentage = item.value
                  const isHigh = percentage > 80
                  const isMedium = percentage > 60
                  const barColor = isHigh ? 'bg-red-500' : isMedium ? 'bg-yellow-500' : 'bg-green-500'
                  const bgColor = isHigh ? 'bg-red-50' : isMedium ? 'bg-yellow-50' : 'bg-green-50'
                  const borderColor = isHigh ? 'border-red-200' : isMedium ? 'border-yellow-200' : 'border-green-200'
                  const textColor = isHigh ? 'text-red-900' : isMedium ? 'text-yellow-900' : 'text-green-900'

                  return (
                    <div key={item.name} className={`p-4 rounded-xl ${bgColor} border-2 ${borderColor} hover:shadow-md transition-all duration-300`}>
                      <div className="flex items-center justify-between mb-3">
                        <span className={`text-sm font-semibold ${textColor}`}>{item.name}</span>
                        <span className={`text-lg font-bold ${textColor} bg-white px-3 py-1 rounded-lg shadow-sm`}>{percentage}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-3 overflow-hidden">
                        <div
                          className={`h-3 rounded-full ${barColor} transition-all duration-500 shadow-sm`}
                          style={{ width: `${percentage}%` }}
                        ></div>
                      </div>
                    </div>
                  )
                })}
              </div>

              {/* 系统状态总结 */}
              <div className="mt-6 p-4 bg-blue-50 rounded-xl border-2 border-blue-200">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="p-2 bg-blue-500 rounded-lg">
                      <Activity className="h-5 w-5 text-white" />
                    </div>
                    <span className="text-sm font-semibold text-blue-900">系统状态评估</span>
                  </div>
                  <span className="text-lg font-bold text-green-600">优秀</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* 性能分析和最近活动 */}
        <div className="grid gap-6 lg:grid-cols-2">
          {/* 查询性能分布 */}
          <Card className="bg-white border border-gray-200 shadow-lg card-hover fade-in">
            <CardHeader className="pb-6">
              <div className="flex items-center justify-between">
                <div className="slide-in-left">
                  <CardTitle className="text-xl font-bold text-gray-900 flex items-center gap-2">
                    <span className="bg-gradient-to-r from-purple-600 to-purple-500 bg-clip-text text-transparent">
                      查询性能分布
                    </span>
                  </CardTitle>
                  <CardDescription className="text-gray-600 mt-2">
                    不同执行时间范围的查询分布统计与性能分析
                  </CardDescription>
                </div>
                <div className="text-sm text-gray-500 glass px-3 py-1.5 rounded-lg scale-in">
                  实时数据
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={performanceData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
                  <XAxis
                    dataKey="label"
                    stroke="#64748b"
                    fontSize={11}
                    tickLine={false}
                    axisLine={false}
                  />
                  <YAxis
                    stroke="#64748b"
                    fontSize={11}
                    tickLine={false}
                    axisLine={false}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: 'white',
                      border: '1px solid #e2e8f0',
                      borderRadius: '8px',
                      boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)'
                    }}
                  />
                  <Bar
                    dataKey="value"
                    fill="#10b981"
                    radius={[4, 4, 0, 0]}
                  />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          {/* 最近活动 */}
          <Card className="bg-white border border-gray-200 shadow-lg card-hover fade-in">
            <CardHeader className="pb-6">
              <div className="flex items-center justify-between">
                <div className="slide-in-left">
                  <CardTitle className="text-xl font-bold text-gray-900">
                    <span className="bg-gradient-to-r from-indigo-600 to-indigo-500 bg-clip-text text-transparent">
                      最近活动
                    </span>
                  </CardTitle>
                  <CardDescription className="text-gray-600 mt-2">
                    系统状态和重要事件监控
                  </CardDescription>
                </div>
                <div className="text-sm text-gray-500 glass px-3 py-1.5 rounded-lg scale-in">
                  实时更新
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {/* 活动项目 */}
                <div className="flex items-start gap-3 p-3 rounded-xl bg-green-50 border border-green-100 hover:bg-green-100 transition-all duration-300 hover:shadow-md hover:scale-105 fade-in">
                  <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center float">
                    <TrendingUp className="h-4 w-4 text-white" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium text-gray-900">
                        查询性能优化
                      </p>
                      <span className="text-xs text-gray-500">2分钟前</span>
                    </div>
                    <p className="text-xs text-gray-600 mt-1">
                      系统自动优化了查询缓存，性能提升15%
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3 p-3 rounded-xl bg-blue-50 border border-blue-100 hover:bg-blue-100 transition-all duration-300 hover:shadow-md hover:scale-105 fade-in" style={{ animationDelay: '0.1s' }}>
                  <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center float">
                    <Database className="h-4 w-4 text-white" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium text-gray-900">
                        数据同步完成
                      </p>
                      <span className="text-xs text-gray-500">5分钟前</span>
                    </div>
                    <p className="text-xs text-gray-600 mt-1">
                      成功同步了1,234条新记录到数据仓库
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3 p-3 rounded-xl bg-yellow-50 border border-yellow-100 hover:bg-yellow-100 transition-all duration-300 hover:shadow-md hover:scale-105 fade-in" style={{ animationDelay: '0.2s' }}>
                  <div className="w-8 h-8 bg-yellow-500 rounded-full flex items-center justify-center float">
                    <AlertCircle className="h-4 w-4 text-white" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium text-gray-900">
                        缓存使用率提醒
                      </p>
                      <span className="text-xs text-gray-500">10分钟前</span>
                    </div>
                    <p className="text-xs text-gray-600 mt-1">
                      建议优化查询缓存策略以提高性能
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3 p-3 rounded-xl bg-gray-50 border border-gray-100 hover:bg-gray-100 transition-all duration-300 hover:shadow-md hover:scale-105 fade-in" style={{ animationDelay: '0.3s' }}>
                  <div className="w-8 h-8 bg-gray-500 rounded-full flex items-center justify-center float">
                    <Activity className="h-4 w-4 text-white" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium text-gray-900">
                        系统健康检查
                      </p>
                      <span className="text-xs text-gray-500">15分钟前</span>
                    </div>
                    <p className="text-xs text-gray-600 mt-1">
                      所有核心服务运行正常，性能指标良好
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
