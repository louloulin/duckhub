import axios from 'axios'

// 创建axios实例
const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器
api.interceptors.request.use(
  (config) => {
    // 可以在这里添加认证token
    const token = localStorage.getItem('auth_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器
api.interceptors.response.use(
  (response) => {
    return response
  },
  (error) => {
    if (error.response?.status === 401) {
      // 处理认证失败
      localStorage.removeItem('auth_token')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

// 查询API
export const queryAPI = {
  execute: (sql: string) => api.post('/query/execute', { sql }),
  getHistory: () => api.get('/query/history'),
  getStats: () => api.get('/query/stats'),
  optimize: (sql: string) => api.post('/query/optimize', { sql }),
}

// 仪表板API
export const dashboardAPI = {
  getMetrics: () => api.get('/dashboard/metrics'),
  getQueryTrends: (timeRange: string) => api.get(`/dashboard/query-trends?range=${timeRange}`),
  getPerformanceData: (timeRange: string) => api.get(`/dashboard/performance?range=${timeRange}`),
  getSystemHealth: () => api.get('/dashboard/system-health'),
}

// AI Agent API
export const aiAgentAPI = {
  sendMessage: (sessionId: string, message: string) => 
    api.post('/ai-agent/chat', { session_id: sessionId, message }),
  processNLPQuery: (query: string) => 
    api.post('/ai-agent/nlp-query', { query }),
  getRecommendations: (context: any) => 
    api.post('/ai-agent/recommendations', { context }),
  createSession: () => 
    api.post('/ai-agent/session'),
  getSessionHistory: (sessionId: string) => 
    api.get(`/ai-agent/session/${sessionId}/history`),
}

// 数据探索API
export const dataExplorerAPI = {
  getTables: () => api.get('/data/tables'),
  getTableSchema: (tableName: string) => api.get(`/data/tables/${tableName}/schema`),
  getTableData: (tableName: string, limit?: number, offset?: number) => 
    api.get(`/data/tables/${tableName}/data`, { params: { limit, offset } }),
  getTableStats: (tableName: string) => api.get(`/data/tables/${tableName}/stats`),
}

// 时间序列分析API
export const timeSeriesAPI = {
  analyze: (table: string, timeColumn: string, valueColumn: string) =>
    api.post('/analytics/time-series/analyze', { table, time_column: timeColumn, value_column: valueColumn }),
  getTrends: (table: string, timeColumn: string, valueColumn: string, period: string) =>
    api.post('/analytics/time-series/trends', { table, time_column: timeColumn, value_column: valueColumn, period }),
  getSeasonality: (table: string, timeColumn: string, valueColumn: string) =>
    api.post('/analytics/time-series/seasonality', { table, time_column: timeColumn, value_column: valueColumn }),
}

// 窗口函数API
export const windowFunctionAPI = {
  generateRanking: (table: string, rankColumn: string, partitionColumns: string[]) =>
    api.post('/analytics/window/ranking', { table, rank_column: rankColumn, partition_columns: partitionColumns }),
  generateMovingAverage: (table: string, valueColumn: string, windowSize: number, orderColumn: string) =>
    api.post('/analytics/window/moving-average', { table, value_column: valueColumn, window_size: windowSize, order_column: orderColumn }),
  generateCumulativeStats: (table: string, valueColumn: string, orderColumn: string) =>
    api.post('/analytics/window/cumulative-stats', { table, value_column: valueColumn, order_column: orderColumn }),
}

// 系统管理API
export const systemAPI = {
  getHealth: () => api.get('/system/health'),
  getMetrics: () => api.get('/system/metrics'),
  getConfig: () => api.get('/system/config'),
  updateConfig: (config: any) => api.put('/system/config', config),
}

export default api
