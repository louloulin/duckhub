import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Settings as SettingsIcon,
  Database,
  Shield,
  Bell,
  Palette,
  Layers,
  Clock,
  GitBranch,
  HardDrive,
  Zap,
  AlertTriangle,
  CheckCircle,
  Save,
  RotateCcw,
} from 'lucide-react'

interface DuckLakeConfig {
  snapshotRetention: {
    enabled: boolean
    retentionDays: number
    maxSnapshots: number
    autoCleanup: boolean
  }
  performance: {
    memoryLimit: string
    threadCount: number
    cacheSize: string
    queryTimeout: number
  }
  security: {
    encryptionEnabled: boolean
    accessLogging: boolean
    auditTrail: boolean
    backupEncryption: boolean
  }
  automation: {
    autoSnapshot: boolean
    snapshotSchedule: string
    performanceMonitoring: boolean
    alertThreshold: number
  }
}

export default function Settings() {
  const [activeTab, setActiveTab] = useState('general')
  const [duckLakeConfig, setDuckLakeConfig] = useState<DuckLakeConfig>({
    snapshotRetention: {
      enabled: true,
      retentionDays: 30,
      maxSnapshots: 100,
      autoCleanup: true,
    },
    performance: {
      memoryLimit: '2GB',
      threadCount: 4,
      cacheSize: '512MB',
      queryTimeout: 30,
    },
    security: {
      encryptionEnabled: true,
      accessLogging: true,
      auditTrail: false,
      backupEncryption: true,
    },
    automation: {
      autoSnapshot: true,
      snapshotSchedule: 'daily',
      performanceMonitoring: true,
      alertThreshold: 80,
    },
  })

  const handleConfigChange = (section: keyof DuckLakeConfig, key: string, value: any) => {
    setDuckLakeConfig(prev => ({
      ...prev,
      [section]: {
        ...prev[section],
        [key]: value,
      },
    }))
  }

  const handleSaveConfig = () => {
    // 保存配置逻辑
    console.log('保存DuckLake配置:', duckLakeConfig)
  }

  const handleResetConfig = () => {
    // 重置配置逻辑
    console.log('重置DuckLake配置')
  }

  return (
    <div className="space-y-6">
      {/* 页面头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-r from-gray-600 to-gray-500 rounded-xl flex items-center justify-center">
              <SettingsIcon className="h-6 w-6 text-white" />
            </div>
            <span className="gradient-text">系统设置</span>
          </h1>
          <p className="text-gray-600 mt-2">
            配置DuckLake数据湖管理系统和用户偏好设置
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={handleResetConfig}>
            <RotateCcw className="h-4 w-4 mr-2" />
            重置配置
          </Button>
          <Button onClick={handleSaveConfig}>
            <Save className="h-4 w-4 mr-2" />
            保存设置
          </Button>
        </div>
      </div>

      {/* 设置标签页 */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="general" className="flex items-center gap-2">
            <Database className="h-4 w-4" />
            通用设置
          </TabsTrigger>
          <TabsTrigger value="ducklake" className="flex items-center gap-2">
            <Layers className="h-4 w-4" />
            DuckLake配置
          </TabsTrigger>
          <TabsTrigger value="security" className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            安全设置
          </TabsTrigger>
          <TabsTrigger value="notifications" className="flex items-center gap-2">
            <Bell className="h-4 w-4" />
            通知设置
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6">

      <div className="grid gap-6 md:grid-cols-2">
        {/* 数据库设置 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              数据库配置
            </CardTitle>
            <CardDescription>
              DuckDB连接和性能参数
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="text-sm font-medium">内存限制</label>
              <input
                type="text"
                defaultValue="2GB"
                className="w-full mt-1 px-3 py-2 border rounded-md"
              />
            </div>
            <div>
              <label className="text-sm font-medium">线程数</label>
              <input
                type="number"
                defaultValue="4"
                className="w-full mt-1 px-3 py-2 border rounded-md"
              />
            </div>
            <div>
              <label className="text-sm font-medium">临时目录</label>
              <input
                type="text"
                defaultValue="/tmp/duckhub"
                className="w-full mt-1 px-3 py-2 border rounded-md"
              />
            </div>
            <Button>保存配置</Button>
          </CardContent>
        </Card>

        {/* 安全设置 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              安全设置
            </CardTitle>
            <CardDescription>
              访问控制和权限管理
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">启用RBAC权限控制</span>
              <div className="w-10 h-6 bg-primary rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">数据脱敏</span>
              <div className="w-10 h-6 bg-primary rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">审计日志</span>
              <div className="w-10 h-6 bg-gray-300 rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 left-1"></div>
              </div>
            </div>
            <Button>更新权限</Button>
          </CardContent>
        </Card>

        {/* 通知设置 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              通知设置
            </CardTitle>
            <CardDescription>
              系统警告和提醒配置
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">查询性能警告</span>
              <div className="w-10 h-6 bg-primary rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">系统资源监控</span>
              <div className="w-10 h-6 bg-primary rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">数据质量报告</span>
              <div className="w-10 h-6 bg-gray-300 rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 left-1"></div>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium">邮件通知地址</label>
              <input
                type="email"
                placeholder="admin@example.com"
                className="w-full mt-1 px-3 py-2 border rounded-md"
              />
            </div>
            <Button>保存设置</Button>
          </CardContent>
        </Card>

        {/* 界面设置 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Palette className="h-5 w-5" />
              界面设置
            </CardTitle>
            <CardDescription>
              主题和显示偏好
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="text-sm font-medium">主题模式</label>
              <select className="w-full mt-1 px-3 py-2 border rounded-md">
                <option>浅色模式</option>
                <option>深色模式</option>
                <option>跟随系统</option>
              </select>
            </div>
            <div>
              <label className="text-sm font-medium">语言</label>
              <select className="w-full mt-1 px-3 py-2 border rounded-md">
                <option>简体中文</option>
                <option>English</option>
              </select>
            </div>
            <div>
              <label className="text-sm font-medium">时区</label>
              <select className="w-full mt-1 px-3 py-2 border rounded-md">
                <option>Asia/Shanghai</option>
                <option>UTC</option>
                <option>America/New_York</option>
              </select>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">显示查询执行计划</span>
              <div className="w-10 h-6 bg-primary rounded-full relative">
                <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
              </div>
            </div>
            <Button>应用设置</Button>
          </CardContent>
        </Card>
      </div>
        </TabsContent>

        {/* DuckLake配置标签页 */}
        <TabsContent value="ducklake" className="space-y-6">
          <div className="grid gap-6 md:grid-cols-2">
            {/* 快照保留策略 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Layers className="h-5 w-5 text-blue-600" />
                  快照保留策略
                </CardTitle>
                <CardDescription>
                  配置快照的自动管理和清理策略
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="snapshot-enabled">启用快照管理</Label>
                  <input
                    type="checkbox"
                    id="snapshot-enabled"
                    checked={duckLakeConfig.snapshotRetention.enabled}
                    onChange={(e) => handleConfigChange('snapshotRetention', 'enabled', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <Label htmlFor="retention-days">保留天数</Label>
                  <Input
                    id="retention-days"
                    type="number"
                    value={duckLakeConfig.snapshotRetention.retentionDays}
                    onChange={(e) => handleConfigChange('snapshotRetention', 'retentionDays', parseInt(e.target.value))}
                    className="mt-1"
                  />
                </div>
                <div>
                  <Label htmlFor="max-snapshots">最大快照数</Label>
                  <Input
                    id="max-snapshots"
                    type="number"
                    value={duckLakeConfig.snapshotRetention.maxSnapshots}
                    onChange={(e) => handleConfigChange('snapshotRetention', 'maxSnapshots', parseInt(e.target.value))}
                    className="mt-1"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="auto-cleanup">自动清理</Label>
                  <input
                    type="checkbox"
                    id="auto-cleanup"
                    checked={duckLakeConfig.snapshotRetention.autoCleanup}
                    onChange={(e) => handleConfigChange('snapshotRetention', 'autoCleanup', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
              </CardContent>
            </Card>

            {/* 性能优化参数 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Zap className="h-5 w-5 text-yellow-600" />
                  性能优化参数
                </CardTitle>
                <CardDescription>
                  调整DuckLake的性能和资源使用
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label htmlFor="memory-limit">内存限制</Label>
                  <Input
                    id="memory-limit"
                    value={duckLakeConfig.performance.memoryLimit}
                    onChange={(e) => handleConfigChange('performance', 'memoryLimit', e.target.value)}
                    className="mt-1"
                  />
                </div>
                <div>
                  <Label htmlFor="thread-count">线程数</Label>
                  <Input
                    id="thread-count"
                    type="number"
                    value={duckLakeConfig.performance.threadCount}
                    onChange={(e) => handleConfigChange('performance', 'threadCount', parseInt(e.target.value))}
                    className="mt-1"
                  />
                </div>
                <div>
                  <Label htmlFor="cache-size">缓存大小</Label>
                  <Input
                    id="cache-size"
                    value={duckLakeConfig.performance.cacheSize}
                    onChange={(e) => handleConfigChange('performance', 'cacheSize', e.target.value)}
                    className="mt-1"
                  />
                </div>
                <div>
                  <Label htmlFor="query-timeout">查询超时 (秒)</Label>
                  <Input
                    id="query-timeout"
                    type="number"
                    value={duckLakeConfig.performance.queryTimeout}
                    onChange={(e) => handleConfigChange('performance', 'queryTimeout', parseInt(e.target.value))}
                    className="mt-1"
                  />
                </div>
              </CardContent>
            </Card>

            {/* 自动化设置 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Clock className="h-5 w-5 text-green-600" />
                  自动化设置
                </CardTitle>
                <CardDescription>
                  配置自动化任务和监控
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="auto-snapshot">自动快照</Label>
                  <input
                    type="checkbox"
                    id="auto-snapshot"
                    checked={duckLakeConfig.automation.autoSnapshot}
                    onChange={(e) => handleConfigChange('automation', 'autoSnapshot', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <Label htmlFor="snapshot-schedule">快照计划</Label>
                  <select
                    id="snapshot-schedule"
                    value={duckLakeConfig.automation.snapshotSchedule}
                    onChange={(e) => handleConfigChange('automation', 'snapshotSchedule', e.target.value)}
                    className="w-full mt-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="hourly">每小时</option>
                    <option value="daily">每日</option>
                    <option value="weekly">每周</option>
                    <option value="monthly">每月</option>
                  </select>
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="performance-monitoring">性能监控</Label>
                  <input
                    type="checkbox"
                    id="performance-monitoring"
                    checked={duckLakeConfig.automation.performanceMonitoring}
                    onChange={(e) => handleConfigChange('automation', 'performanceMonitoring', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <Label htmlFor="alert-threshold">告警阈值 (%)</Label>
                  <Input
                    id="alert-threshold"
                    type="number"
                    min="0"
                    max="100"
                    value={duckLakeConfig.automation.alertThreshold}
                    onChange={(e) => handleConfigChange('automation', 'alertThreshold', parseInt(e.target.value))}
                    className="mt-1"
                  />
                </div>
              </CardContent>
            </Card>

            {/* 配置状态 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <GitBranch className="h-5 w-5 text-purple-600" />
                  配置状态
                </CardTitle>
                <CardDescription>
                  当前DuckLake配置的健康状态
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">快照策略</span>
                    <Badge className="bg-green-100 text-green-800">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      正常
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">性能配置</span>
                    <Badge className="bg-green-100 text-green-800">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      优化
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">自动化任务</span>
                    <Badge className="bg-yellow-100 text-yellow-800">
                      <AlertTriangle className="h-3 w-3 mr-1" />
                      注意
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">存储使用</span>
                    <Badge className="bg-blue-100 text-blue-800">
                      <HardDrive className="h-3 w-3 mr-1" />
                      75%
                    </Badge>
                  </div>
                </div>
                <div className="mt-4 p-3 bg-blue-50 rounded-lg">
                  <p className="text-sm text-blue-800">
                    <CheckCircle className="h-4 w-4 inline mr-1" />
                    DuckLake配置已优化，建议定期检查快照使用情况
                  </p>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* 安全设置标签页 */}
        <TabsContent value="security" className="space-y-6">
          <div className="grid gap-6 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5 text-red-600" />
                  DuckLake安全设置
                </CardTitle>
                <CardDescription>
                  配置数据加密和访问控制
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="encryption-enabled">数据加密</Label>
                  <input
                    type="checkbox"
                    id="encryption-enabled"
                    checked={duckLakeConfig.security.encryptionEnabled}
                    onChange={(e) => handleConfigChange('security', 'encryptionEnabled', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="access-logging">访问日志</Label>
                  <input
                    type="checkbox"
                    id="access-logging"
                    checked={duckLakeConfig.security.accessLogging}
                    onChange={(e) => handleConfigChange('security', 'accessLogging', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="audit-trail">审计跟踪</Label>
                  <input
                    type="checkbox"
                    id="audit-trail"
                    checked={duckLakeConfig.security.auditTrail}
                    onChange={(e) => handleConfigChange('security', 'auditTrail', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="backup-encryption">备份加密</Label>
                  <input
                    type="checkbox"
                    id="backup-encryption"
                    checked={duckLakeConfig.security.backupEncryption}
                    onChange={(e) => handleConfigChange('security', 'backupEncryption', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* 通知设置标签页 */}
        <TabsContent value="notifications" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Bell className="h-5 w-5 text-orange-600" />
                DuckLake通知设置
              </CardTitle>
              <CardDescription>
                配置快照、性能和错误通知
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm">快照创建通知</span>
                <input
                  type="checkbox"
                  defaultChecked
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">性能告警</span>
                <input
                  type="checkbox"
                  defaultChecked
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Schema变更通知</span>
                <input
                  type="checkbox"
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
              </div>
              <div>
                <Label htmlFor="notification-email">通知邮箱</Label>
                <Input
                  id="notification-email"
                  type="email"
                  placeholder="admin@example.com"
                  className="mt-1"
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* 系统信息 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <SettingsIcon className="h-5 w-5" />
            系统信息
          </CardTitle>
          <CardDescription>
            当前系统状态和版本信息
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="p-4 bg-blue-50 rounded-lg">
              <div className="text-sm text-blue-600">DuckHub版本</div>
              <div className="text-lg font-bold text-blue-900">v0.1.0</div>
            </div>
            <div className="p-4 bg-green-50 rounded-lg">
              <div className="text-sm text-green-600">DuckDB版本</div>
              <div className="text-lg font-bold text-green-900">v0.9.2</div>
            </div>
            <div className="p-4 bg-purple-50 rounded-lg">
              <div className="text-sm text-purple-600">运行时间</div>
              <div className="text-lg font-bold text-purple-900">7天 12小时</div>
            </div>
          </div>
          <div className="mt-6 flex gap-2">
            <Button variant="outline">检查更新</Button>
            <Button variant="outline">导出配置</Button>
            <Button variant="outline">重启服务</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
