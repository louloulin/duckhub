import { render, screen, waitFor } from '@testing-library/react'
import { Provider } from 'react-redux'
import { BrowserRouter } from 'react-router-dom'
import { configureStore } from '@reduxjs/toolkit'
import Dashboard from '../pages/Dashboard'
import DataExplorer from '../pages/DataExplorer'
import { dashboardAPI, dataExplorerAPI, duckLakeAPI } from '../services/api'

// Mock APIs
jest.mock('../services/api', () => ({
  dashboardAPI: {
    getMetrics: jest.fn(),
    getQueryTrends: jest.fn(),
    getPerformanceData: jest.fn(),
    getSystemHealth: jest.fn(),
  },
  dataExplorerAPI: {
    getTables: jest.fn(),
    getTableSchema: jest.fn(),
    getTableData: jest.fn(),
  },
  duckLakeAPI: {
    getMetrics: jest.fn(),
    getDatabases: jest.fn(),
  }
}))

const mockStore = configureStore({
  reducer: {
    dashboard: (state = {
      metrics: null,
      queryTrends: [],
      performanceData: [],
      systemHealth: null,
      loading: false,
      error: null,
      refreshInterval: 30000
    }) => state
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

describe('真实数据集成测试', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  describe('Dashboard 真实数据', () => {
    test('应该从真实API加载DuckLake指标', async () => {
      const mockMetrics = {
        active_databases: 3,
        total_snapshots: 127,
        time_travel_queries: 1250,
        schema_evolutions: 15,
        query_performance: [
          { time: '00:00', version: 126, avg_response_time: 245, throughput: 1200 }
        ]
      }

      ;(duckLakeAPI.getMetrics as jest.Mock).mockResolvedValue({
        data: {
          success: true,
          data: mockMetrics
        }
      })

      ;(dashboardAPI.getMetrics as jest.Mock).mockResolvedValue({
        data: {
          total_queries: 1000,
          avg_execution_time: 150,
          cache_hit_rate: 0.85,
          active_connections: 5
        }
      })

      renderWithProviders(<Dashboard />)

      // 验证API调用
      await waitFor(() => {
        expect(duckLakeAPI.getMetrics).toHaveBeenCalledWith('24h')
      })

      // 验证数据显示
      await waitFor(() => {
        expect(screen.getByText('3')).toBeInTheDocument() // 活跃数据库数
        expect(screen.getByText('127')).toBeInTheDocument() // 总快照数
      })
    })

    test('应该正确处理API错误', async () => {
      ;(duckLakeAPI.getMetrics as jest.Mock).mockRejectedValue(new Error('网络错误'))
      ;(dashboardAPI.getMetrics as jest.Mock).mockResolvedValue({
        data: {
          total_queries: 1000,
          avg_execution_time: 150,
          cache_hit_rate: 0.85,
          active_connections: 5
        }
      })

      renderWithProviders(<Dashboard />)

      // 验证错误处理
      await waitFor(() => {
        expect(duckLakeAPI.getMetrics).toHaveBeenCalled()
      })

      // 应该显示错误状态而不是崩溃
      expect(screen.getByText('仪表板')).toBeInTheDocument()
    })
  })

  describe('DataExplorer 真实数据', () => {
    test('应该从真实API加载表数据', async () => {
      const mockTables = [
        {
          name: 'transactions',
          rows: 1250000,
          size: '2.3 GB',
          schema_version: 5,
          last_modified: '2024-01-11 14:30:25',
          description: '交易记录表'
        },
        {
          name: 'users',
          rows: 50000,
          size: '120 MB',
          schema_version: 3,
          last_modified: '2024-01-11 12:15:30',
          description: '用户信息表'
        }
      ]

      ;(dataExplorerAPI.getTables as jest.Mock).mockResolvedValue({
        data: {
          success: true,
          data: mockTables
        }
      })

      renderWithProviders(<DataExplorer />)

      // 验证API调用
      await waitFor(() => {
        expect(dataExplorerAPI.getTables).toHaveBeenCalled()
      })

      // 验证表数据显示
      await waitFor(() => {
        expect(screen.getByText('transactions')).toBeInTheDocument()
        expect(screen.getByText('users')).toBeInTheDocument()
        expect(screen.getByText('1,250,000')).toBeInTheDocument() // 行数格式化
      })
    })

    test('应该显示空状态当没有表时', async () => {
      ;(dataExplorerAPI.getTables as jest.Mock).mockResolvedValue({
        data: {
          success: true,
          data: []
        }
      })

      renderWithProviders(<DataExplorer />)

      await waitFor(() => {
        expect(dataExplorerAPI.getTables).toHaveBeenCalled()
      })

      // 验证空状态显示
      await waitFor(() => {
        expect(screen.getByText(/暂无数据表/)).toBeInTheDocument()
      })
    })
  })

  describe('API错误处理', () => {
    test('应该优雅处理网络错误', async () => {
      ;(dataExplorerAPI.getTables as jest.Mock).mockRejectedValue(new Error('网络连接失败'))

      renderWithProviders(<DataExplorer />)

      await waitFor(() => {
        expect(dataExplorerAPI.getTables).toHaveBeenCalled()
      })

      // 应该显示错误信息而不是崩溃
      await waitFor(() => {
        expect(screen.getByText(/获取表列表失败/)).toBeInTheDocument()
      })
    })

    test('应该处理API返回的错误响应', async () => {
      ;(dataExplorerAPI.getTables as jest.Mock).mockResolvedValue({
        data: {
          success: false,
          message: '权限不足'
        }
      })

      renderWithProviders(<DataExplorer />)

      await waitFor(() => {
        expect(dataExplorerAPI.getTables).toHaveBeenCalled()
      })

      // 验证错误处理
      await waitFor(() => {
        expect(screen.getByText(/获取表列表失败/)).toBeInTheDocument()
      })
    })
  })

  describe('数据刷新机制', () => {
    test('应该支持自动数据刷新', async () => {
      jest.useFakeTimers()

      ;(duckLakeAPI.getMetrics as jest.Mock).mockResolvedValue({
        data: { success: true, data: {} }
      })
      ;(dashboardAPI.getMetrics as jest.Mock).mockResolvedValue({
        data: { total_queries: 1000 }
      })

      renderWithProviders(<Dashboard />)

      // 初始调用
      await waitFor(() => {
        expect(duckLakeAPI.getMetrics).toHaveBeenCalledTimes(1)
      })

      // 模拟30秒后的自动刷新
      jest.advanceTimersByTime(30000)

      await waitFor(() => {
        expect(duckLakeAPI.getMetrics).toHaveBeenCalledTimes(2)
      })

      jest.useRealTimers()
    })
  })
})
