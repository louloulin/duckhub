import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Bot,
  Send,
  Lightbulb,
  Zap,
  MessageSquare,
  Clock,
  GitBranch,
  Layers,
  Database,
  AlertTriangle,
  CheckCircle,
  TrendingUp,
} from 'lucide-react'

interface BaseMessage {
  id: string
  content: string
  timestamp: string
}

interface UserMessage extends BaseMessage {
  type: 'user'
}

interface AssistantMessage extends BaseMessage {
  type: 'assistant'
  category?: 'general' | 'time_travel' | 'schema' | 'snapshot' | 'performance'
  metadata?: {
    sql_query?: string
    execution_time?: number
    result_count?: number
    time_travel?: {
      suggested_version?: number
      suggested_timestamp?: string
      reasoning?: string
    }
    schema_suggestion?: {
      operation: 'add_column' | 'modify_column' | 'add_index'
      table: string
      details: string
      compatibility: 'safe' | 'breaking' | 'warning'
      impact: 'low' | 'medium' | 'high'
    }
    snapshot_recommendation?: {
      action: 'create' | 'cleanup' | 'optimize'
      reason: string
      estimated_benefit: string
    }
  }
}

type Message = UserMessage | AssistantMessage

export default function AIAgent() {
  const [message, setMessage] = useState('')
  const [activeTab, setActiveTab] = useState('chat')
  const [messages, setMessages] = useState<Message[]>([])

  const handleSendMessage = () => {
    if (!message.trim()) return

    // 添加用户消息
    const userMessage: UserMessage = {
      id: Date.now().toString(),
      type: 'user',
      content: message,
      timestamp: new Date().toISOString(),
    }
    setMessages(prev => [...prev, userMessage])

    // 智能AI回复 - 基于DuckLake功能
    setTimeout(() => {
      const assistantMessage = generateSmartResponse(message)
      setMessages(prev => [...prev, assistantMessage])
    }, 1000)

    setMessage('')
  }

  const generateSmartResponse = (userInput: string): AssistantMessage => {
    const input = userInput.toLowerCase()

    // 时间旅行查询相关
    if (input.includes('历史') || input.includes('时间旅行') || input.includes('版本') || input.includes('之前')) {
      return {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: '我为您推荐一个时间旅行查询方案。基于您的需求，建议查询版本126的数据，该版本在昨天下午创建，包含了完整的交易记录。',
        timestamp: new Date().toISOString(),
        category: 'time_travel',
        metadata: {
          sql_query: 'SELECT * FROM financial_data.transactions AT VERSION 126 WHERE created_at >= \'2024-01-10\'',
          time_travel: {
            suggested_version: 126,
            suggested_timestamp: '2024-01-10 16:20:15',
            reasoning: '版本126包含最稳定的数据集，且Schema兼容性良好'
          }
        }
      }
    }

    // Schema演进相关
    if (input.includes('schema') || input.includes('表结构') || input.includes('字段') || input.includes('列')) {
      return {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: '我分析了您的Schema演进需求。建议为transactions表添加status字段，这是一个向后兼容的安全操作。',
        timestamp: new Date().toISOString(),
        category: 'schema',
        metadata: {
          schema_suggestion: {
            operation: 'add_column',
            table: 'transactions',
            details: 'ALTER TABLE transactions ADD COLUMN status VARCHAR(50) DEFAULT \'pending\'',
            compatibility: 'safe',
            impact: 'low'
          }
        }
      }
    }

    // 快照管理相关
    if (input.includes('快照') || input.includes('备份') || input.includes('版本管理')) {
      return {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: '基于您的数据增长模式，我建议创建一个新的快照。当前数据变化率较高，及时快照可以保护重要数据状态。',
        timestamp: new Date().toISOString(),
        category: 'snapshot',
        metadata: {
          snapshot_recommendation: {
            action: 'create',
            reason: '数据变化率达到15%，建议及时创建快照',
            estimated_benefit: '可节省20%的恢复时间，提升数据安全性'
          }
        }
      }
    }

    // 性能优化相关
    if (input.includes('性能') || input.includes('优化') || input.includes('慢') || input.includes('快')) {
      return {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: '我检测到您的查询性能可以进一步优化。建议为user_id字段添加索引，预计可提升查询速度40%。',
        timestamp: new Date().toISOString(),
        category: 'performance',
        metadata: {
          sql_query: 'CREATE INDEX idx_user_id ON transactions(user_id)',
          execution_time: 85,
          result_count: 1250,
          schema_suggestion: {
            operation: 'add_index',
            table: 'transactions',
            details: 'CREATE INDEX idx_user_id ON transactions(user_id)',
            compatibility: 'safe',
            impact: 'low'
          }
        }
      }
    }

    // 默认通用回复
    return {
      id: (Date.now() + 1).toString(),
      type: 'assistant',
      content: '我理解您的需求。基于DuckLake的强大功能，我可以为您提供数据查询、版本管理和性能优化建议。请告诉我更具体的需求，我会为您制定最佳方案。',
      timestamp: new Date().toISOString(),
      category: 'general',
      metadata: {
        sql_query: 'SELECT * FROM information_schema.tables WHERE table_schema = \'financial_data\'',
        execution_time: 45,
        result_count: 15,
      }
    }
  }

  const quickActions = [
    { label: '时间旅行查询历史数据', icon: Clock, category: 'time_travel' },
    { label: 'Schema演进建议', icon: GitBranch, category: 'schema' },
    { label: '快照管理优化', icon: Layers, category: 'snapshot' },
    { label: '性能优化分析', icon: Zap, category: 'performance' },
    { label: '显示所有交易记录', icon: MessageSquare, category: 'general' },
    { label: '检测异常交易', icon: Lightbulb, category: 'general' },
  ]

  const recommendations = [
    {
      id: '1',
      title: '时间旅行查询优化',
      description: '建议使用版本126查询昨日数据，性能最佳',
      priority: 'high' as const,
      category: 'time_travel',
      sql: 'SELECT * FROM financial_data.transactions AT VERSION 126',
      icon: Clock,
    },
    {
      id: '2',
      title: 'Schema演进建议',
      description: '为transactions表添加status字段，向后兼容',
      priority: 'medium' as const,
      category: 'schema',
      sql: 'ALTER TABLE transactions ADD COLUMN status VARCHAR(50) DEFAULT \'pending\'',
      icon: GitBranch,
    },
    {
      id: '3',
      title: '快照清理策略',
      description: '建议清理30天前的快照，释放存储空间',
      priority: 'medium' as const,
      category: 'snapshot',
      sql: 'SELECT * FROM snapshots WHERE created_at < NOW() - INTERVAL 30 DAY',
      icon: Layers,
    },
    {
      id: '4',
      title: '查询性能优化',
      description: '为user_id字段添加索引，提升查询速度40%',
      priority: 'high' as const,
      category: 'performance',
      sql: 'CREATE INDEX idx_user_id ON transactions(user_id)',
      icon: Zap,
    },
  ]

  return (
    <div className="space-y-6">
      {/* 页面头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-r from-purple-600 to-purple-500 rounded-xl flex items-center justify-center">
              <Bot className="h-6 w-6 text-white" />
            </div>
            <span className="gradient-text">DuckLake AI助手</span>
          </h1>
          <p className="text-gray-600 mt-2">
            专为DuckLake优化的智能助手，提供时间旅行查询、Schema演进和性能优化建议
          </p>
        </div>
      </div>

      {/* AI功能标签页 */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="chat" className="flex items-center gap-2">
            <MessageSquare className="h-4 w-4" />
            智能对话
          </TabsTrigger>
          <TabsTrigger value="recommendations" className="flex items-center gap-2">
            <Lightbulb className="h-4 w-4" />
            智能建议
          </TabsTrigger>
          <TabsTrigger value="insights" className="flex items-center gap-2">
            <TrendingUp className="h-4 w-4" />
            数据洞察
          </TabsTrigger>
        </TabsList>

        <TabsContent value="chat" className="space-y-6">

      <div className="grid gap-6 lg:grid-cols-3">
        {/* 聊天界面 */}
        <div className="lg:col-span-2">
          <Card className="h-[600px] flex flex-col">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Bot className="h-5 w-5" />
                AI对话
              </CardTitle>
              <CardDescription>
                使用自然语言描述您的数据需求
              </CardDescription>
            </CardHeader>
            <CardContent className="flex-1 flex flex-col">
              {/* 消息列表 */}
              <div className="flex-1 overflow-y-auto space-y-4 mb-4">
                {messages.map((msg) => (
                  <div
                    key={msg.id}
                    className={`flex ${msg.type === 'user' ? 'justify-end' : 'justify-start'}`}
                  >
                    <div
                      className={`max-w-[80%] p-3 rounded-lg ${
                        msg.type === 'user'
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted'
                      }`}
                    >
                      <p className="text-sm">{msg.content}</p>
                      {msg.type === 'assistant' && msg.metadata?.sql_query && (
                        <div className="mt-2 p-2 bg-black/10 rounded text-xs font-mono">
                          {msg.metadata.sql_query}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              {/* 输入框 */}
              <div className="flex gap-2">
                <input
                  type="text"
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
                  placeholder="输入您的问题，例如：显示本月交易金额最大的10笔记录"
                  className="flex-1 px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
                />
                <Button onClick={handleSendMessage} disabled={!message.trim()}>
                  <Send className="h-4 w-4" />
                </Button>
              </div>

              {/* 快捷操作 */}
              <div className="mt-4">
                <p className="text-sm text-muted-foreground mb-2">快捷操作：</p>
                <div className="flex flex-wrap gap-2">
                  {quickActions.map((action, index) => {
                    const Icon = action.icon
                    return (
                      <Button
                        key={index}
                        variant="outline"
                        size="sm"
                        onClick={() => setMessage(action.label)}
                      >
                        <Icon className="h-3 w-3 mr-1" />
                        {action.label}
                      </Button>
                    )
                  })}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* 智能推荐 */}
        <div>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Lightbulb className="h-5 w-5" />
                智能推荐
              </CardTitle>
              <CardDescription>
                基于数据分析的优化建议
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {recommendations.map((rec) => (
                  <div
                    key={rec.id}
                    className={`p-3 rounded-lg border ${
                      rec.priority === 'high'
                        ? 'border-red-200 bg-red-50'
                        : rec.priority === 'medium'
                        ? 'border-yellow-200 bg-yellow-50'
                        : 'border-blue-200 bg-blue-50'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <h4 className="font-medium text-sm">{rec.title}</h4>
                        <p className="text-xs text-muted-foreground mt-1">
                          {rec.description}
                        </p>
                      </div>
                      <span
                        className={`px-2 py-1 text-xs rounded ${
                          rec.priority === 'high'
                            ? 'bg-red-100 text-red-800'
                            : rec.priority === 'medium'
                            ? 'bg-yellow-100 text-yellow-800'
                            : 'bg-blue-100 text-blue-800'
                        }`}
                      >
                        {rec.priority === 'high' ? '高' : rec.priority === 'medium' ? '中' : '低'}
                      </span>
                    </div>
                    {rec.sql && (
                      <div className="mt-2">
                        <code className="text-xs bg-black/10 p-1 rounded block">
                          {rec.sql}
                        </code>
                      </div>
                    )}
                    <div className="mt-2 flex gap-1">
                      <Button size="sm" variant="outline" className="text-xs h-6">
                        应用
                      </Button>
                      <Button size="sm" variant="ghost" className="text-xs h-6">
                        忽略
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* AI功能设置 */}
          <Card className="mt-6">
            <CardHeader>
              <CardTitle>AI功能设置</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm">自然语言查询</span>
                  <div className="w-10 h-6 bg-primary rounded-full relative">
                    <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">智能推荐</span>
                  <div className="w-10 h-6 bg-primary rounded-full relative">
                    <div className="w-4 h-4 bg-white rounded-full absolute top-1 right-1"></div>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">自动优化建议</span>
                  <div className="w-10 h-6 bg-gray-300 rounded-full relative">
                    <div className="w-4 h-4 bg-white rounded-full absolute top-1 left-1"></div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
        </TabsContent>

        {/* 智能建议标签页 */}
        <TabsContent value="recommendations" className="space-y-6">
          <div className="grid gap-6 md:grid-cols-2">
            {recommendations.map((rec) => (
              <Card key={rec.id} className="card-hover">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">
                      <rec.icon className="h-5 w-5 text-blue-600" />
                      {rec.title}
                    </CardTitle>
                    <Badge className={
                      rec.priority === 'high' ? 'bg-red-100 text-red-800' :
                      rec.priority === 'medium' ? 'bg-yellow-100 text-yellow-800' :
                      'bg-green-100 text-green-800'
                    }>
                      {rec.priority === 'high' ? '高优先级' :
                       rec.priority === 'medium' ? '中优先级' : '低优先级'}
                    </Badge>
                  </div>
                  <CardDescription>{rec.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-3">
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <p className="text-xs text-gray-600 mb-1">建议SQL:</p>
                      <code className="text-sm font-mono">{rec.sql}</code>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button size="sm">
                        应用建议
                      </Button>
                      <Button size="sm" variant="outline">
                        查看详情
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* 数据洞察标签页 */}
        <TabsContent value="insights" className="space-y-6">
          <div className="grid gap-6 md:grid-cols-3">
            {/* DuckLake健康状态 */}
            <Card className="card-hover">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Database className="h-5 w-5 text-green-600" />
                  DuckLake健康状态
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">数据库连接</span>
                    <Badge className="bg-green-100 text-green-800">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      正常
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">快照完整性</span>
                    <Badge className="bg-green-100 text-green-800">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      100%
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Schema兼容性</span>
                    <Badge className="bg-yellow-100 text-yellow-800">
                      <AlertTriangle className="h-3 w-3 mr-1" />
                      注意
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 性能洞察 */}
            <Card className="card-hover">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <TrendingUp className="h-5 w-5 text-blue-600" />
                  性能洞察
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div>
                    <p className="text-sm text-gray-600">平均查询时间</p>
                    <p className="text-2xl font-bold text-blue-600">95ms</p>
                    <p className="text-xs text-green-600">↓ 12% 较上周</p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-600">时间旅行查询</p>
                    <p className="text-2xl font-bold text-purple-600">1,250</p>
                    <p className="text-xs text-green-600">↑ 8% 较上周</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 使用建议 */}
            <Card className="card-hover">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Lightbulb className="h-5 w-5 text-yellow-600" />
                  使用建议
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="p-3 bg-blue-50 rounded-lg">
                    <p className="text-sm font-medium text-blue-800">时间旅行优化</p>
                    <p className="text-xs text-blue-600">使用版本查询比时间戳查询快30%</p>
                  </div>
                  <div className="p-3 bg-green-50 rounded-lg">
                    <p className="text-sm font-medium text-green-800">快照策略</p>
                    <p className="text-xs text-green-600">建议每日创建快照，保留30天</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
