import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { Provider } from 'react-redux'
import { BrowserRouter } from 'react-router-dom'
import { configureStore } from '@reduxjs/toolkit'
import AIAgent from '../pages/AIAgent'
import { aiAgentAPI } from '../services/api'

// Mock API
jest.mock('../services/api', () => ({
  aiAgentAPI: {
    processNLPQuery: jest.fn(),
    getRecommendations: jest.fn(),
  }
}))

const mockStore = configureStore({
  reducer: {
    // 添加必要的reducers
  }
})

const renderWithProviders = (component: React.ReactElement) => {
  return render(
    <Provider store={mockStore}>
      <BrowserRouter>
        {component}
      </BrowserRouter>
    </Provider>
  )
}

describe('AIAgent 真实化测试', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test('应该正确加载AI推荐数据', async () => {
    const mockRecommendations = [
      {
        id: '1',
        title: '测试推荐',
        description: '这是一个测试推荐',
        priority: 'high',
        category: 'performance',
        sql: 'SELECT * FROM test',
        icon: 'Zap'
      }
    ]

    ;(aiAgentAPI.getRecommendations as jest.Mock).mockResolvedValue({
      data: {
        success: true,
        data: mockRecommendations
      }
    })

    renderWithProviders(<AIAgent />)

    // 等待加载完成
    await waitFor(() => {
      expect(screen.queryByText('正在加载AI建议...')).not.toBeInTheDocument()
    })

    // 验证API被调用
    expect(aiAgentAPI.getRecommendations).toHaveBeenCalledWith({
      context: 'ducklake_management',
      user_preferences: ['performance', 'schema', 'snapshots']
    })
  })

  test('应该正确处理AI智能回复', async () => {
    const mockResponse = {
      data: {
        success: true,
        data: {
          response: '这是AI的回复',
          category: 'general',
          metadata: {
            sql_query: 'SELECT * FROM test'
          }
        }
      }
    }

    ;(aiAgentAPI.processNLPQuery as jest.Mock).mockResolvedValue(mockResponse)
    ;(aiAgentAPI.getRecommendations as jest.Mock).mockResolvedValue({
      data: { success: true, data: [] }
    })

    renderWithProviders(<AIAgent />)

    // 等待组件加载
    await waitFor(() => {
      expect(screen.queryByText('正在加载AI建议...')).not.toBeInTheDocument()
    })

    // 输入消息
    const input = screen.getByPlaceholderText('输入您的问题...')
    fireEvent.change(input, { target: { value: '查询性能如何优化？' } })

    // 点击发送
    const sendButton = screen.getByRole('button', { name: /发送/i })
    fireEvent.click(sendButton)

    // 验证加载状态
    await waitFor(() => {
      expect(screen.getByText('正在分析您的问题...')).toBeInTheDocument()
    })

    // 等待AI回复
    await waitFor(() => {
      expect(screen.getByText('这是AI的回复')).toBeInTheDocument()
    })

    // 验证API被调用
    expect(aiAgentAPI.processNLPQuery).toHaveBeenCalledWith('查询性能如何优化？')
  })

  test('应该正确处理API错误', async () => {
    ;(aiAgentAPI.processNLPQuery as jest.Mock).mockRejectedValue(new Error('API错误'))
    ;(aiAgentAPI.getRecommendations as jest.Mock).mockResolvedValue({
      data: { success: true, data: [] }
    })

    renderWithProviders(<AIAgent />)

    // 等待组件加载
    await waitFor(() => {
      expect(screen.queryByText('正在加载AI建议...')).not.toBeInTheDocument()
    })

    // 输入消息
    const input = screen.getByPlaceholderText('输入您的问题...')
    fireEvent.change(input, { target: { value: '测试错误' } })

    // 点击发送
    const sendButton = screen.getByRole('button', { name: /发送/i })
    fireEvent.click(sendButton)

    // 等待错误处理
    await waitFor(() => {
      expect(screen.getByText('我理解您的需求。请连接网络以获取更智能的AI分析和建议。')).toBeInTheDocument()
    })
  })

  test('应该显示空状态当没有推荐时', async () => {
    ;(aiAgentAPI.getRecommendations as jest.Mock).mockResolvedValue({
      data: { success: true, data: [] }
    })

    renderWithProviders(<AIAgent />)

    // 等待加载完成
    await waitFor(() => {
      expect(screen.queryByText('正在加载AI建议...')).not.toBeInTheDocument()
    })

    // 切换到推荐标签页
    const recommendationsTab = screen.getByRole('tab', { name: /智能建议/i })
    fireEvent.click(recommendationsTab)

    // 验证空状态
    expect(screen.getByText('暂无AI建议，请稍后刷新')).toBeInTheDocument()
  })
})
