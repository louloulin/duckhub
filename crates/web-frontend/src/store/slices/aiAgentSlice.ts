import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit'
import { aiAgentAPI } from '../../services/api'

export interface ChatMessage {
  id: string
  type: 'user' | 'assistant' | 'system'
  content: string
  timestamp: string
  metadata?: {
    sql_query?: string
    query_result?: any
    recommendations?: Recommendation[]
  }
}

export interface Recommendation {
  id: string
  type: string
  title: string
  description: string
  sql_query?: string
  priority: 'low' | 'medium' | 'high' | 'critical'
  confidence: number
  tags: string[]
}

export interface AIAgentState {
  messages: ChatMessage[]
  currentSession: string
  recommendations: Recommendation[]
  loading: boolean
  error: string | null
  nlpEnabled: boolean
  autoSuggestions: boolean
}

const initialState: AIAgentState = {
  messages: [],
  currentSession: '',
  recommendations: [],
  loading: false,
  error: null,
  nlpEnabled: true,
  autoSuggestions: true,
}

// 异步操作
export const sendMessage = createAsyncThunk(
  'aiAgent/sendMessage',
  async ({ sessionId, message }: { sessionId: string; message: string }) => {
    const response = await aiAgentAPI.sendMessage(sessionId, message)
    return response.data
  }
)

export const processNLPQuery = createAsyncThunk(
  'aiAgent/processNLPQuery',
  async (query: string) => {
    const response = await aiAgentAPI.processNLPQuery(query)
    return response.data
  }
)

export const getRecommendations = createAsyncThunk(
  'aiAgent/getRecommendations',
  async (context: any) => {
    const response = await aiAgentAPI.getRecommendations(context)
    return response.data
  }
)

export const createNewSession = createAsyncThunk(
  'aiAgent/createNewSession',
  async () => {
    const response = await aiAgentAPI.createSession()
    return response.data
  }
)

const aiAgentSlice = createSlice({
  name: 'aiAgent',
  initialState,
  reducers: {
    addMessage: (state, action: PayloadAction<ChatMessage>) => {
      state.messages.push(action.payload)
    },
    clearMessages: (state) => {
      state.messages = []
    },
    setCurrentSession: (state, action: PayloadAction<string>) => {
      state.currentSession = action.payload
    },
    toggleNLP: (state) => {
      state.nlpEnabled = !state.nlpEnabled
    },
    toggleAutoSuggestions: (state) => {
      state.autoSuggestions = !state.autoSuggestions
    },
    clearError: (state) => {
      state.error = null
    },
  },
  extraReducers: (builder) => {
    builder
      // 发送消息
      .addCase(sendMessage.pending, (state) => {
        state.loading = true
        state.error = null
      })
      .addCase(sendMessage.fulfilled, (state, action) => {
        state.loading = false
        // 添加AI回复消息
        const aiMessage: ChatMessage = {
          id: Date.now().toString(),
          type: 'assistant',
          content: action.payload.content,
          timestamp: new Date().toISOString(),
          metadata: {
            sql_query: action.payload.sql_query,
            query_result: action.payload.query_result,
            recommendations: action.payload.recommendations,
          },
        }
        state.messages.push(aiMessage)
      })
      .addCase(sendMessage.rejected, (state, action) => {
        state.loading = false
        state.error = action.error.message || '发送消息失败'
      })
      // 处理NLP查询
      .addCase(processNLPQuery.fulfilled, (state, action) => {
        const aiMessage: ChatMessage = {
          id: Date.now().toString(),
          type: 'assistant',
          content: action.payload.content,
          timestamp: new Date().toISOString(),
          metadata: {
            sql_query: action.payload.sql_query,
            query_result: action.payload.query_result,
          },
        }
        state.messages.push(aiMessage)
      })
      // 获取推荐
      .addCase(getRecommendations.fulfilled, (state, action) => {
        state.recommendations = action.payload.recommendations || []
      })
      // 创建新会话
      .addCase(createNewSession.fulfilled, (state, action) => {
        state.currentSession = action.payload.session_id
        state.messages = []
      })
  },
})

export const {
  addMessage,
  clearMessages,
  setCurrentSession,
  toggleNLP,
  toggleAutoSuggestions,
  clearError,
} = aiAgentSlice.actions

export default aiAgentSlice.reducer
