import { useState, useEffect } from 'react'
import { WifiOff, RefreshCw, AlertCircle, CheckCircle } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { ConnectionStatus, useRealtimeStatus } from '@/hooks/useRealtime'

interface RealtimeIndicatorProps {
  status: ConnectionStatus
  lastUpdated?: Date | null
  onRefresh?: () => void
  className?: string
  showText?: boolean
  size?: 'sm' | 'md' | 'lg'
}

export function RealtimeIndicator({
  status,
  lastUpdated,
  onRefresh,
  className,
  showText = true,
  size = 'md'
}: RealtimeIndicatorProps) {
  const { isOnline } = useRealtimeStatus()
  const [timeAgo, setTimeAgo] = useState<string>('')

  // 更新时间显示
  useEffect(() => {
    if (!lastUpdated) return

    const updateTimeAgo = () => {
      const now = new Date()
      const diff = now.getTime() - lastUpdated.getTime()
      const seconds = Math.floor(diff / 1000)
      const minutes = Math.floor(seconds / 60)
      const hours = Math.floor(minutes / 60)

      if (seconds < 60) {
        setTimeAgo('刚刚')
      } else if (minutes < 60) {
        setTimeAgo(`${minutes}分钟前`)
      } else if (hours < 24) {
        setTimeAgo(`${hours}小时前`)
      } else {
        setTimeAgo('很久以前')
      }
    }

    updateTimeAgo()
    const interval = setInterval(updateTimeAgo, 30000) // 每30秒更新

    return () => clearInterval(interval)
  }, [lastUpdated])

  // 获取状态配置
  const getStatusConfig = () => {
    if (!isOnline) {
      return {
        icon: WifiOff,
        color: 'text-destructive',
        bgColor: 'bg-destructive/10',
        text: '离线',
        pulse: false
      }
    }

    switch (status) {
      case ConnectionStatus.CONNECTED:
        return {
          icon: CheckCircle,
          color: 'text-success',
          bgColor: 'bg-success/10',
          text: '已连接',
          pulse: true
        }
      case ConnectionStatus.CONNECTING:
      case ConnectionStatus.RECONNECTING:
        return {
          icon: RefreshCw,
          color: 'text-warning',
          bgColor: 'bg-warning/10',
          text: status === ConnectionStatus.CONNECTING ? '连接中' : '重连中',
          pulse: true,
          spin: true
        }
      case ConnectionStatus.ERROR:
        return {
          icon: AlertCircle,
          color: 'text-destructive',
          bgColor: 'bg-destructive/10',
          text: '连接错误',
          pulse: false
        }
      case ConnectionStatus.DISCONNECTED:
      default:
        return {
          icon: WifiOff,
          color: 'text-muted-foreground',
          bgColor: 'bg-muted/10',
          text: '未连接',
          pulse: false
        }
    }
  }

  const config = getStatusConfig()
  const Icon = config.icon

  const sizeClasses = {
    sm: 'h-3 w-3',
    md: 'h-4 w-4',
    lg: 'h-5 w-5'
  }

  const containerSizeClasses = {
    sm: 'gap-1 px-2 py-1 text-xs',
    md: 'gap-2 px-3 py-1.5 text-sm',
    lg: 'gap-2 px-4 py-2 text-base'
  }

  return (
    <div className={cn(
      "flex items-center rounded-full border transition-all duration-200",
      config.bgColor,
      containerSizeClasses[size],
      className
    )}>
      <div className="relative">
        <Icon 
          className={cn(
            sizeClasses[size],
            config.color,
            config.spin && "animate-spin",
            "transition-colors duration-200"
          )}
        />
        {config.pulse && (
          <div className={cn(
            "absolute inset-0 rounded-full animate-ping",
            config.color.replace('text-', 'bg-'),
            "opacity-20"
          )} />
        )}
      </div>
      
      {showText && (
        <div className="flex flex-col">
          <span className={cn("font-medium", config.color)}>
            {config.text}
          </span>
          {lastUpdated && timeAgo && (
            <span className="text-xs text-muted-foreground">
              {timeAgo}更新
            </span>
          )}
        </div>
      )}
      
      {onRefresh && (
        <Button
          variant="ghost"
          size="icon"
          onClick={onRefresh}
          className={cn(
            "ml-1 hover:bg-background/50",
            size === 'sm' ? 'h-5 w-5' : size === 'md' ? 'h-6 w-6' : 'h-7 w-7'
          )}
          title="手动刷新"
        >
          <RefreshCw className={cn(
            size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'
          )} />
        </Button>
      )}
    </div>
  )
}

// 简化版状态指示器
interface SimpleStatusIndicatorProps {
  status: ConnectionStatus
  className?: string
}

export function SimpleStatusIndicator({ status, className }: SimpleStatusIndicatorProps) {
  const { isOnline } = useRealtimeStatus()
  
  if (!isOnline) {
    return (
      <div className={cn("flex items-center gap-1", className)}>
        <div className="w-2 h-2 rounded-full bg-destructive" />
        <span className="text-xs text-muted-foreground">离线</span>
      </div>
    )
  }

  const getStatusColor = () => {
    switch (status) {
      case ConnectionStatus.CONNECTED:
        return 'bg-success'
      case ConnectionStatus.CONNECTING:
      case ConnectionStatus.RECONNECTING:
        return 'bg-warning'
      case ConnectionStatus.ERROR:
        return 'bg-destructive'
      case ConnectionStatus.DISCONNECTED:
      default:
        return 'bg-muted-foreground'
    }
  }

  const getStatusText = () => {
    switch (status) {
      case ConnectionStatus.CONNECTED:
        return '在线'
      case ConnectionStatus.CONNECTING:
        return '连接中'
      case ConnectionStatus.RECONNECTING:
        return '重连中'
      case ConnectionStatus.ERROR:
        return '错误'
      case ConnectionStatus.DISCONNECTED:
      default:
        return '离线'
    }
  }

  return (
    <div className={cn("flex items-center gap-1", className)}>
      <div className={cn(
        "w-2 h-2 rounded-full transition-colors duration-200",
        getStatusColor(),
        (status === ConnectionStatus.CONNECTING || status === ConnectionStatus.RECONNECTING) && "animate-pulse"
      )} />
      <span className="text-xs text-muted-foreground">
        {getStatusText()}
      </span>
    </div>
  )
}

// 网络状态监控组件
export function NetworkStatusMonitor() {
  const { isOnline } = useRealtimeStatus()
  const [showOfflineMessage, setShowOfflineMessage] = useState(false)

  useEffect(() => {
    if (!isOnline) {
      const timer = setTimeout(() => {
        setShowOfflineMessage(true)
      }, 3000) // 3秒后显示离线消息

      return () => clearTimeout(timer)
    } else {
      setShowOfflineMessage(false)
    }
  }, [isOnline])

  if (!showOfflineMessage) return null

  return (
    <div className="fixed top-4 left-1/2 -translate-x-1/2 z-50">
      <div className="bg-destructive text-destructive-foreground px-4 py-2 rounded-lg shadow-lg flex items-center gap-2">
        <WifiOff className="h-4 w-4" />
        <span className="text-sm font-medium">
          网络连接已断开，请检查您的网络设置
        </span>
      </div>
    </div>
  )
}
