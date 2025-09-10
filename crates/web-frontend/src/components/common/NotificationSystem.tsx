import { useState, useEffect } from 'react'
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'

export interface Notification {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  description?: string
  duration?: number
  action?: {
    label: string
    onClick: () => void
  }
}

interface NotificationItemProps {
  notification: Notification
  onDismiss: (id: string) => void
}

function NotificationItem({ notification, onDismiss }: NotificationItemProps) {
  const [isVisible, setIsVisible] = useState(false)
  const [isExiting, setIsExiting] = useState(false)

  useEffect(() => {
    // 进入动画
    const timer = setTimeout(() => setIsVisible(true), 50)
    return () => clearTimeout(timer)
  }, [])

  useEffect(() => {
    if (notification.duration && notification.duration > 0) {
      const timer = setTimeout(() => {
        handleDismiss()
      }, notification.duration)
      return () => clearTimeout(timer)
    }
  }, [notification.duration])

  const handleDismiss = () => {
    setIsExiting(true)
    setTimeout(() => {
      onDismiss(notification.id)
    }, 200)
  }

  const icons = {
    success: CheckCircle,
    error: AlertCircle,
    warning: AlertTriangle,
    info: Info,
  }

  const Icon = icons[notification.type]

  const typeStyles = {
    success: 'border-success/20 bg-success/10 text-success',
    error: 'border-destructive/20 bg-destructive/10 text-destructive',
    warning: 'border-warning/20 bg-warning/10 text-warning',
    info: 'border-info/20 bg-info/10 text-info',
  }

  return (
    <div
      className={cn(
        "relative flex w-full max-w-sm items-start gap-3 rounded-lg border p-4 shadow-lg backdrop-blur-sm transition-all duration-200",
        typeStyles[notification.type],
        isVisible && !isExiting ? "translate-x-0 opacity-100" : "translate-x-full opacity-0",
        isExiting && "translate-x-full opacity-0"
      )}
    >
      <Icon className="h-5 w-5 shrink-0 mt-0.5" />
      
      <div className="flex-1 space-y-1">
        <h4 className="text-sm font-medium text-foreground">
          {notification.title}
        </h4>
        {notification.description && (
          <p className="text-sm text-muted-foreground">
            {notification.description}
          </p>
        )}
        {notification.action && (
          <Button
            variant="ghost"
            size="sm"
            onClick={notification.action.onClick}
            className="h-auto p-0 text-xs font-medium hover:bg-transparent"
          >
            {notification.action.label}
          </Button>
        )}
      </div>
      
      <Button
        variant="ghost"
        size="icon"
        onClick={handleDismiss}
        className="h-6 w-6 shrink-0 hover:bg-background/50"
      >
        <X className="h-4 w-4" />
      </Button>
    </div>
  )
}

interface NotificationContainerProps {
  notifications: Notification[]
  onDismiss: (id: string) => void
}

export function NotificationContainer({ notifications, onDismiss }: NotificationContainerProps) {
  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2">
      {notifications.map((notification) => (
        <NotificationItem
          key={notification.id}
          notification={notification}
          onDismiss={onDismiss}
        />
      ))}
    </div>
  )
}

// 通知管理 Hook
export function useNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([])

  const addNotification = (notification: Omit<Notification, 'id'>) => {
    const id = Math.random().toString(36).substr(2, 9)
    const newNotification: Notification = {
      ...notification,
      id,
      duration: notification.duration ?? 5000, // 默认5秒
    }
    
    setNotifications(prev => [...prev, newNotification])
    return id
  }

  const dismissNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id))
  }

  const clearAllNotifications = () => {
    setNotifications([])
  }

  // 便捷方法
  const success = (title: string, description?: string, options?: Partial<Notification>) => {
    return addNotification({ type: 'success', title, description, ...options })
  }

  const error = (title: string, description?: string, options?: Partial<Notification>) => {
    return addNotification({ type: 'error', title, description, duration: 0, ...options })
  }

  const warning = (title: string, description?: string, options?: Partial<Notification>) => {
    return addNotification({ type: 'warning', title, description, ...options })
  }

  const info = (title: string, description?: string, options?: Partial<Notification>) => {
    return addNotification({ type: 'info', title, description, ...options })
  }

  return {
    notifications,
    addNotification,
    dismissNotification,
    clearAllNotifications,
    success,
    error,
    warning,
    info,
  }
}

// 全局通知提供者
import { createContext, useContext } from 'react'

const NotificationContext = createContext<ReturnType<typeof useNotifications> | null>(null)

export function NotificationProvider({ children }: { children: React.ReactNode }) {
  const notificationMethods = useNotifications()

  return (
    <NotificationContext.Provider value={notificationMethods}>
      {children}
      <NotificationContainer
        notifications={notificationMethods.notifications}
        onDismiss={notificationMethods.dismissNotification}
      />
    </NotificationContext.Provider>
  )
}

export function useNotificationContext() {
  const context = useContext(NotificationContext)
  if (!context) {
    throw new Error('useNotificationContext must be used within a NotificationProvider')
  }
  return context
}
