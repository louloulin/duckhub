import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Bot, Send, Lightbulb, Zap, MessageSquare } from 'lucide-react'

export default function AIAgent() {
  const [message, setMessage] = useState('')
  const [messages, setMessages] = useState([
    {
      id: '1',
      type: 'assistant' as const,
      content: '您好！我是DuckHub AI助手，可以帮助您进行自然语言查询、数据分析和获取智能推荐。有什么我可以帮助您的吗？',
      timestamp: new Date().toISOString(),
    }
  ])

  const handleSendMessage = () => {
    if (!message.trim()) return

    // 添加用户消息
    const userMessage = {
      id: Date.now().toString(),
      type: 'user' as const,
      content: message,
      timestamp: new Date().toISOString(),
    }
    setMessages(prev => [...prev, userMessage])

    // 模拟AI回复
    setTimeout(() => {
      const aiMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant' as const,
        content: `我理解您想要查询"${message}"。让我为您生成相应的SQL查询并执行分析...`,
        timestamp: new Date().toISOString(),
        metadata: {
          sql_query: 'SELECT * FROM transactions WHERE amount > 1000 ORDER BY created_at DESC LIMIT 10',
          recommendations: [
            {
              id: 'rec1',
              title: '查询优化建议',
              description: '建议在amount列上添加索引以提高查询性能',
              priority: 'medium' as const,
            }
          ]
        }
      }
      setMessages(prev => [...prev, aiMessage])
    }, 1000)

    setMessage('')
  }

  const quickActions = [
    { label: '显示所有交易记录', icon: MessageSquare },
    { label: '分析用户行为趋势', icon: Zap },
    { label: '检测异常交易', icon: Lightbulb },
    { label: '生成月度报告', icon: Bot },
  ]

  const recommendations = [
    {
      id: '1',
      title: '查询性能优化',
      description: '建议为transactions表的user_id列添加索引',
      priority: 'high' as const,
      sql: 'CREATE INDEX idx_transactions_user_id ON transactions(user_id)',
    },
    {
      id: '2',
      title: '数据质量检查',
      description: '发现amount列存在负值，建议进行数据清洗',
      priority: 'medium' as const,
      sql: 'SELECT * FROM transactions WHERE amount < 0',
    },
    {
      id: '3',
      title: '时间序列分析',
      description: '可以对交易数据进行时间序列分析以发现趋势',
      priority: 'low' as const,
      sql: 'SELECT DATE(created_at) as date, COUNT(*) as daily_count FROM transactions GROUP BY DATE(created_at)',
    },
  ]

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">AI助手</h1>
        <p className="text-muted-foreground">
          使用自然语言进行数据查询和分析
        </p>
      </div>

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
                      {msg.metadata?.sql_query && (
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
    </div>
  )
}
