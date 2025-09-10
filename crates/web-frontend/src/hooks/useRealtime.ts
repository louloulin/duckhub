import { useState, useEffect, useRef, useCallback } from 'react'

// 实时连接状态
export enum ConnectionStatus {
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  ERROR = 'error'
}

// 实时数据更新配置
interface RealtimeConfig {
  endpoint?: string
  pollInterval?: number
  maxRetries?: number
  retryDelay?: number
  enableWebSocket?: boolean
  enablePolling?: boolean
}

// 实时数据钩子返回值
interface RealtimeHook<T> {
  data: T | null
  status: ConnectionStatus
  error: string | null
  lastUpdated: Date | null
  retryCount: number
  refresh: () => Promise<void>
  connect: () => void
  disconnect: () => void
}

// 默认配置
const DEFAULT_CONFIG: Required<RealtimeConfig> = {
  endpoint: '/api/v1',
  pollInterval: 30000, // 30秒轮询
  maxRetries: 5,
  retryDelay: 2000, // 2秒重试延迟
  enableWebSocket: true,
  enablePolling: true
}

/**
 * 实时数据更新钩子
 * 支持WebSocket和轮询两种方式
 */
export function useRealtime<T>(
  dataFetcher: () => Promise<T>,
  config: RealtimeConfig = {}
): RealtimeHook<T> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config }
  
  // 状态管理
  const [data, setData] = useState<T | null>(null)
  const [status, setStatus] = useState<ConnectionStatus>(ConnectionStatus.DISCONNECTED)
  const [error, setError] = useState<string | null>(null)
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null)
  const [retryCount, setRetryCount] = useState(0)
  
  // 引用管理
  const wsRef = useRef<WebSocket | null>(null)
  const pollTimerRef = useRef<NodeJS.Timeout | null>(null)
  const retryTimerRef = useRef<NodeJS.Timeout | null>(null)
  const mountedRef = useRef(true)

  // 清理定时器
  const clearTimers = useCallback(() => {
    if (pollTimerRef.current) {
      clearInterval(pollTimerRef.current)
      pollTimerRef.current = null
    }
    if (retryTimerRef.current) {
      clearTimeout(retryTimerRef.current)
      retryTimerRef.current = null
    }
  }, [])

  // 获取数据
  const fetchData = useCallback(async () => {
    if (!mountedRef.current) return

    try {
      const result = await dataFetcher()
      if (mountedRef.current) {
        setData(result)
        setError(null)
        setLastUpdated(new Date())
        setRetryCount(0)
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err.message : '数据获取失败')
        console.error('实时数据获取错误:', err)
      }
    }
  }, [dataFetcher])

  // WebSocket连接管理
  const connectWebSocket = useCallback(() => {
    if (!finalConfig.enableWebSocket || wsRef.current?.readyState === WebSocket.OPEN) {
      return
    }

    try {
      setStatus(ConnectionStatus.CONNECTING)
      
      // 构建WebSocket URL
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      const wsUrl = `${protocol}//${window.location.host}${finalConfig.endpoint}/ws`
      
      const ws = new WebSocket(wsUrl)
      wsRef.current = ws

      ws.onopen = () => {
        if (mountedRef.current) {
          setStatus(ConnectionStatus.CONNECTED)
          setError(null)
          setRetryCount(0)
          console.log('WebSocket连接已建立')
        }
      }

      ws.onmessage = (event) => {
        if (!mountedRef.current) return
        
        try {
          const message = JSON.parse(event.data)
          if (message.type === 'data_update') {
            fetchData()
          }
        } catch (err) {
          console.error('WebSocket消息解析错误:', err)
        }
      }

      ws.onclose = () => {
        if (mountedRef.current) {
          setStatus(ConnectionStatus.DISCONNECTED)
          wsRef.current = null
          
          // 自动重连
          if (retryCount < finalConfig.maxRetries) {
            setStatus(ConnectionStatus.RECONNECTING)
            retryTimerRef.current = setTimeout(() => {
              setRetryCount(prev => prev + 1)
              connectWebSocket()
            }, finalConfig.retryDelay)
          }
        }
      }

      ws.onerror = (error) => {
        if (mountedRef.current) {
          setStatus(ConnectionStatus.ERROR)
          setError('WebSocket连接错误')
          console.error('WebSocket错误:', error)
        }
      }

    } catch (err) {
      if (mountedRef.current) {
        setStatus(ConnectionStatus.ERROR)
        setError('WebSocket连接失败')
        console.error('WebSocket连接失败:', err)
      }
    }
  }, [finalConfig, retryCount, fetchData])

  // 轮询管理
  const startPolling = useCallback(() => {
    if (!finalConfig.enablePolling) return

    clearTimers()
    
    // 立即获取一次数据
    fetchData()
    
    // 设置定时轮询
    pollTimerRef.current = setInterval(() => {
      fetchData()
    }, finalConfig.pollInterval)
  }, [finalConfig.enablePolling, finalConfig.pollInterval, fetchData, clearTimers])

  // 手动刷新
  const refresh = useCallback(async () => {
    await fetchData()
  }, [fetchData])

  // 连接
  const connect = useCallback(() => {
    if (finalConfig.enableWebSocket) {
      connectWebSocket()
    }
    if (finalConfig.enablePolling) {
      startPolling()
    }
  }, [finalConfig.enableWebSocket, finalConfig.enablePolling, connectWebSocket, startPolling])

  // 断开连接
  const disconnect = useCallback(() => {
    clearTimers()
    
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    
    setStatus(ConnectionStatus.DISCONNECTED)
  }, [clearTimers])

  // 组件挂载时自动连接
  useEffect(() => {
    mountedRef.current = true
    connect()

    return () => {
      mountedRef.current = false
      disconnect()
    }
  }, [connect, disconnect])

  // 组件卸载时清理
  useEffect(() => {
    return () => {
      mountedRef.current = false
      clearTimers()
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [clearTimers])

  return {
    data,
    status,
    error,
    lastUpdated,
    retryCount,
    refresh,
    connect,
    disconnect
  }
}

/**
 * 实时状态指示器钩子
 */
export function useRealtimeStatus() {
  const [isOnline, setIsOnline] = useState(navigator.onLine)
  const [lastSeen, setLastSeen] = useState<Date>(new Date())

  useEffect(() => {
    const handleOnline = () => {
      setIsOnline(true)
      setLastSeen(new Date())
    }
    
    const handleOffline = () => {
      setIsOnline(false)
    }

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    // 定期更新最后在线时间
    const interval = setInterval(() => {
      if (navigator.onLine) {
        setLastSeen(new Date())
      }
    }, 30000)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
      clearInterval(interval)
    }
  }, [])

  return { isOnline, lastSeen }
}
