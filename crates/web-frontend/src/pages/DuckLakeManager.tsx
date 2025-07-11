import { useState } from 'react'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Database,
  Plus,
  Activity,
  Clock,
  CheckCircle,
  RefreshCw,
  GitBranch,
  History,
  Layers,
} from 'lucide-react'
import DatabasePanel from '@/components/ducklake/DatabasePanel'
import SnapshotBrowser from '@/components/ducklake/SnapshotBrowser'
import VersionControl from '@/components/ducklake/VersionControl'
import DuckLakeMetrics from '@/components/ducklake/DuckLakeMetrics'

export default function DuckLakeManager() {
  const [activeTab, setActiveTab] = useState('databases')
  const [refreshing, setRefreshing] = useState(false)

  // 模拟数据刷新
  const handleRefresh = async () => {
    setRefreshing(true)
    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))
    setRefreshing(false)
  }

  return (
    <div className="space-y-6 fade-in">
      {/* 页面头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-r from-blue-600 to-blue-500 rounded-xl flex items-center justify-center">
              <Database className="h-6 w-6 text-white" />
            </div>
            <span className="gradient-text">DuckLake 管理中心</span>
          </h1>
          <p className="text-gray-600 mt-2">
            管理DuckLake数据库、快照、版本控制和监控
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Button
            variant="outline"
            onClick={handleRefresh}
            disabled={refreshing}
            className="flex items-center gap-2"
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
            刷新
          </Button>
          <Button className="flex items-center gap-2">
            <Plus className="h-4 w-4" />
            新建数据库
          </Button>
        </div>
      </div>

      {/* 快速统计卡片 */}
      <div className="grid gap-6 md:grid-cols-4">
        <Card className="card-hover">
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">活跃数据库</p>
                <p className="text-2xl font-bold text-gray-900">3</p>
              </div>
              <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
                <Database className="h-6 w-6 text-green-600" />
              </div>
            </div>
            <div className="mt-4 flex items-center text-sm">
              <CheckCircle className="h-4 w-4 text-green-500 mr-1" />
              <span className="text-green-600">全部在线</span>
            </div>
          </CardContent>
        </Card>

        <Card className="card-hover">
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">总快照数</p>
                <p className="text-2xl font-bold text-gray-900">127</p>
              </div>
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                <Layers className="h-6 w-6 text-blue-600" />
              </div>
            </div>
            <div className="mt-4 flex items-center text-sm">
              <Clock className="h-4 w-4 text-blue-500 mr-1" />
              <span className="text-blue-600">最新: 2分钟前</span>
            </div>
          </CardContent>
        </Card>

        <Card className="card-hover">
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">时间旅行查询</p>
                <p className="text-2xl font-bold text-gray-900">45</p>
              </div>
              <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
                <History className="h-6 w-6 text-purple-600" />
              </div>
            </div>
            <div className="mt-4 flex items-center text-sm">
              <Activity className="h-4 w-4 text-purple-500 mr-1" />
              <span className="text-purple-600">今日查询</span>
            </div>
          </CardContent>
        </Card>

        <Card className="card-hover">
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Schema演进</p>
                <p className="text-2xl font-bold text-gray-900">8</p>
              </div>
              <div className="w-12 h-12 bg-orange-100 rounded-lg flex items-center justify-center">
                <GitBranch className="h-6 w-6 text-orange-600" />
              </div>
            </div>
            <div className="mt-4 flex items-center text-sm">
              <Activity className="h-4 w-4 text-orange-500 mr-1" />
              <span className="text-orange-600">本周操作</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 主要功能标签页 */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="databases" className="flex items-center gap-2">
            <Database className="h-4 w-4" />
            数据库管理
          </TabsTrigger>
          <TabsTrigger value="snapshots" className="flex items-center gap-2">
            <Layers className="h-4 w-4" />
            快照浏览器
          </TabsTrigger>
          <TabsTrigger value="versions" className="flex items-center gap-2">
            <GitBranch className="h-4 w-4" />
            版本控制
          </TabsTrigger>
          <TabsTrigger value="metrics" className="flex items-center gap-2">
            <Activity className="h-4 w-4" />
            监控指标
          </TabsTrigger>
        </TabsList>

        <TabsContent value="databases" className="space-y-6">
          <DatabasePanel />
        </TabsContent>

        <TabsContent value="snapshots" className="space-y-6">
          <SnapshotBrowser />
        </TabsContent>

        <TabsContent value="versions" className="space-y-6">
          <VersionControl />
        </TabsContent>

        <TabsContent value="metrics" className="space-y-6">
          <DuckLakeMetrics />
        </TabsContent>
      </Tabs>
    </div>
  )
}
