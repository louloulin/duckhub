import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit'
import { dashboardAPI } from '../../services/api'

export interface DashboardMetrics {
  total_queries: number
  avg_execution_time: number
  cache_hit_rate: number
  active_connections: number
  data_volume_gb: number
  error_rate: number
}

export interface ChartData {
  timestamp: string
  value: number
  label?: string
}

export interface DashboardState {
  metrics: DashboardMetrics | null
  queryTrends: ChartData[]
  performanceData: ChartData[]
  systemHealth: {
    cpu_usage: number
    memory_usage: number
    disk_usage: number
    network_io: number
  } | null
  loading: boolean
  error: string | null
  refreshInterval: number
}

const initialState: DashboardState = {
  metrics: null,
  queryTrends: [],
  performanceData: [],
  systemHealth: null,
  loading: false,
  error: null,
  refreshInterval: 30000, // 30秒
}

// 异步操作
export const fetchDashboardMetrics = createAsyncThunk(
  'dashboard/fetchMetrics',
  async () => {
    const response = await dashboardAPI.getMetrics()
    return response.data
  }
)

export const fetchQueryTrends = createAsyncThunk(
  'dashboard/fetchQueryTrends',
  async (timeRange: string) => {
    const response = await dashboardAPI.getQueryTrends(timeRange)
    return response.data
  }
)

export const fetchPerformanceData = createAsyncThunk(
  'dashboard/fetchPerformanceData',
  async (timeRange: string) => {
    const response = await dashboardAPI.getPerformanceData(timeRange)
    return response.data
  }
)

export const fetchSystemHealth = createAsyncThunk(
  'dashboard/fetchSystemHealth',
  async () => {
    const response = await dashboardAPI.getSystemHealth()
    return response.data
  }
)

const dashboardSlice = createSlice({
  name: 'dashboard',
  initialState,
  reducers: {
    setRefreshInterval: (state, action: PayloadAction<number>) => {
      state.refreshInterval = action.payload
    },
    clearError: (state) => {
      state.error = null
    },
  },
  extraReducers: (builder) => {
    builder
      // 获取仪表板指标
      .addCase(fetchDashboardMetrics.pending, (state) => {
        state.loading = true
        state.error = null
      })
      .addCase(fetchDashboardMetrics.fulfilled, (state, action) => {
        state.loading = false
        state.metrics = action.payload
      })
      .addCase(fetchDashboardMetrics.rejected, (state, action) => {
        state.loading = false
        state.error = action.error.message || '获取仪表板数据失败'
      })
      // 获取查询趋势
      .addCase(fetchQueryTrends.fulfilled, (state, action) => {
        state.queryTrends = action.payload
      })
      // 获取性能数据
      .addCase(fetchPerformanceData.fulfilled, (state, action) => {
        state.performanceData = action.payload
      })
      // 获取系统健康状态
      .addCase(fetchSystemHealth.fulfilled, (state, action) => {
        state.systemHealth = action.payload
      })
  },
})

export const { setRefreshInterval, clearError } = dashboardSlice.actions
export default dashboardSlice.reducer
