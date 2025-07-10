import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Settings as SettingsIcon, Database, Shield, Bell, Palette } from 'lucide-react'

export default function Settings() {
  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">设置</h1>
        <p className="text-muted-foreground">
          配置系统参数和用户偏好
        </p>
      </div>

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
