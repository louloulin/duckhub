import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { aiAgentAPI } from '@/services/api'
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
  category?: 'general' | 'time_travel' | 'schema' | 'snapshot' | 'performance' | 'loading' | 'error'
  metadata?: {
    sql_query?: string
    execution_time?: number
    result_count?: number
    error?: string
    note?: string
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
  const [quickActions, setQuickActions] = useState<any[]>([])
  const [recommendations, setRecommendations] = useState<any[]>([])
  const [loading, setLoading] = useState(true)

  // 加载AI助手数据
  useEffect(() => {
    const loadAIData = async () => {
      setLoading(true)
      try {
        // 获取AI推荐
        const recommendationsResponse = await aiAgentAPI.getRecommendations({
          context: 'ducklake_management',
          user_preferences: ['performance', 'schema', 'snapshots']
        })

        if (recommendationsResponse.data.success) {
          setRecommendations(recommendationsResponse.data.data || [])
        }

        // 设置快速操作（这些可以是静态的，因为它们是UI功能）
        setQuickActions([
          { label: '时间旅行查询历史数据', icon: Clock, category: 'time_travel' },
          { label: 'Schema演进建议', icon: GitBranch, category: 'schema' },
          { label: '快照管理优化', icon: Layers, category: 'snapshot' },
          { label: '性能优化分析', icon: Zap, category: 'performance' },
          { label: '显示所有交易记录', icon: MessageSquare, category: 'general' },
          { label: '检测异常交易', icon: Lightbulb, category: 'general' },
        ])

      } catch (error) {
        console.error('加载AI数据失败:', error)
        // 使用备用数据
        setQuickActions([
          { label: '时间旅行查询历史数据', icon: Clock, category: 'time_travel' },
          { label: 'Schema演进建议', icon: GitBranch, category: 'schema' },
          { label: '快照管理优化', icon: Layers, category: 'snapshot' },
          { label: '性能优化分析', icon: Zap, category: 'performance' },
        ])
        setRecommendations([])
      } finally {
        setLoading(false)
      }
    }

    loadAIData()
  }, [])

  const handleSendMessage = async () => {
    if (!message.trim()) return

    const userInput = message
    setMessage('') // 立即清空输入框

    // 添加用户消息
    const userMessage: UserMessage = {
      id: Date.now().toString(),
      type: 'user',
      content: userInput,
      timestamp: new Date().toISOString(),
    }
    setMessages(prev => [...prev, userMessage])

    // 添加加载状态消息
    const loadingMessage: AssistantMessage = {
      id: (Date.now() + 1).toString(),
      type: 'assistant',
      content: '正在分析您的问题...',
      timestamp: new Date().toISOString(),
      category: 'loading',
      metadata: {}
    }
    setMessages(prev => [...prev, loadingMessage])

    try {
      // 调用真实的AI智能回复
      const assistantMessage = await generateSmartResponse(userInput)

      // 替换加载消息为真实回复
      setMessages(prev =>
        prev.map(msg =>
          msg.id === loadingMessage.id ? assistantMessage : msg
        )
      )
    } catch (error) {
      console.error('AI回复失败:', error)

      // 使用备用回复替换加载消息
      const fallbackMessage = generateFallbackResponse(userInput)
      setMessages(prev =>
        prev.map(msg =>
          msg.id === loadingMessage.id ? fallbackMessage : msg
        )
      )
    }
  }

  // 真实的AI智能回复生成函数
  const generateSmartResponse = async (userInput: string): Promise<AssistantMessage> => {
    try {
      // 调用真实的AI Agent API
      const response = await aiAgentAPI.processNLPQuery(userInput)

      if (response.data.success) {
        return {
          id: (Date.now() + 1).toString(),
          type: 'assistant',
          content: response.data.data.response,
          timestamp: new Date().toISOString(),
          category: response.data.data.category || 'general',
          metadata: response.data.data.metadata || {}
        }
      } else {
        throw new Error(response.data.message || 'AI处理失败')
      }
    } catch (error) {
      console.error('AI智能回复生成失败:', error)

      // 错误时返回基础回复
      return {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: '抱歉，我暂时无法处理您的请求。请稍后重试或联系管理员。',
        timestamp: new Date().toISOString(),
        category: 'error',
        metadata: {
          error: error instanceof Error ? error.message : '未知错误'
        }
      }
    }
  }

  // 备用的本地智能回复（当API不可用时使用）
  const generateFallbackResponse = (userInput: string): AssistantMessage => {
    const input = userInput.toLowerCase()

    // 时间旅行查询相关
    if (input.includes('历史') || input.includes('时间旅行') || input.includes('版本') || input.includes('之前')) {
      return {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: '我为您推荐一个时间旅行查询方案。基于您的需求，建议查询历史版本的数据。',
        timestamp: new Date().toISOString(),
        category: 'time_travel',
        metadata: {
          sql_query: 'SELECT * FROM financial_data.transactions AT VERSION (SELECT MAX(version) - 1 FROM snapshots)',
          note: '这是离线模式的基础建议，请连接网络获取更精确的分析'
        }
      }
    }

    // 基础的离线回复
    return {
      id: (Date.now() + 1).toString(),
      type: 'assistant',
      content: '我理解您的需求。请连接网络以获取更智能的AI分析和建议。',
      timestamp: new Date().toISOString(),
      category: 'general',
      metadata: {
        note: '离线模式 - 请连接网络获取完整的AI功能'
      }
    }
  }

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
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="text-center">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
                <p className="text-gray-600">正在加载AI建议...</p>
              </div>
            </div>
          ) : (
            <div className="grid gap-6 md:grid-cols-2">
              {recommendations.length === 0 ? (
                <div className="col-span-2 text-center py-12">
                  <Bot className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                  <p className="text-gray-600">暂无AI建议，请稍后刷新</p>
                </div>
              ) : (
                recommendations.map((rec) => (
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
                ))
              )}
            </div>
          )}
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
