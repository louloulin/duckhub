import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit'
import { queryAPI } from '../../services/api'

export interface QueryResult {
  query_id: string
  execution_time_ms: number
  row_count: number
  optimized: boolean
  cache_hit: boolean
  execution_plan?: string
  data: Record<string, any>[]
}

export interface QueryState {
  currentQuery: string
  results: QueryResult | null
  history: QueryResult[]
  loading: boolean
  error: string | null
}

const initialState: QueryState = {
  currentQuery: '',
  results: null,
  history: [],
  loading: false,
  error: null,
}

// 异步操作
export const executeQuery = createAsyncThunk(
  'query/execute',
  async (sql: string) => {
    const response = await queryAPI.execute(sql)
    return response.data
  }
)

export const getQueryHistory = createAsyncThunk(
  'query/getHistory',
  async () => {
    const response = await queryAPI.getHistory()
    return response.data
  }
)

const querySlice = createSlice({
  name: 'query',
  initialState,
  reducers: {
    setCurrentQuery: (state, action: PayloadAction<string>) => {
      state.currentQuery = action.payload
    },
    clearResults: (state) => {
      state.results = null
      state.error = null
    },
    clearError: (state) => {
      state.error = null
    },
  },
  extraReducers: (builder) => {
    builder
      // 执行查询
      .addCase(executeQuery.pending, (state) => {
        state.loading = true
        state.error = null
      })
      .addCase(executeQuery.fulfilled, (state, action) => {
        state.loading = false
        state.results = action.payload
        state.history.unshift(action.payload)
        // 保持历史记录在100条以内
        if (state.history.length > 100) {
          state.history = state.history.slice(0, 100)
        }
      })
      .addCase(executeQuery.rejected, (state, action) => {
        state.loading = false
        state.error = action.error.message || '查询执行失败'
      })
      // 获取历史记录
      .addCase(getQueryHistory.fulfilled, (state, action) => {
        state.history = action.payload
      })
  },
})

export const { setCurrentQuery, clearResults, clearError } = querySlice.actions
export default querySlice.reducer
