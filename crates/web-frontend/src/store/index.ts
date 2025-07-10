import { configureStore } from '@reduxjs/toolkit'
import querySlice from './slices/querySlice'
import dashboardSlice from './slices/dashboardSlice'
import aiAgentSlice from './slices/aiAgentSlice'

export const store = configureStore({
  reducer: {
    query: querySlice,
    dashboard: dashboardSlice,
    aiAgent: aiAgentSlice,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST'],
      },
    }),
})

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
