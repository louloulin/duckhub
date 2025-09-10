import React from 'react'
import { AlertTriangle, RefreshCw, Home } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'

interface ErrorBoundaryState {
  hasError: boolean
  error?: Error
  errorInfo?: React.ErrorInfo
}

interface ErrorBoundaryProps {
  children: React.ReactNode
  fallback?: React.ComponentType<ErrorFallbackProps>
}

interface ErrorFallbackProps {
  error?: Error
  resetError: () => void
}

// 默认错误回退组件
function DefaultErrorFallback({ error, resetError }: ErrorFallbackProps) {
  return (
    <div className="min-h-[400px] flex items-center justify-center p-6">
      <Card className="max-w-md w-full p-6 text-center">
        <div className="flex justify-center mb-4">
          <div className="w-12 h-12 rounded-full bg-destructive/10 flex items-center justify-center">
            <AlertTriangle className="h-6 w-6 text-destructive" />
          </div>
        </div>
        
        <h2 className="text-lg font-semibold text-foreground mb-2">
          出现了一些问题
        </h2>
        
        <p className="text-sm text-muted-foreground mb-4">
          抱歉，页面遇到了意外错误。请尝试刷新页面或返回首页。
        </p>
        
        {process.env.NODE_ENV === 'development' && error && (
          <details className="text-left mb-4">
            <summary className="text-xs text-muted-foreground cursor-pointer mb-2">
              错误详情 (开发模式)
            </summary>
            <pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-32">
              {error.message}
              {error.stack}
            </pre>
          </details>
        )}
        
        <div className="flex gap-2 justify-center">
          <Button
            variant="outline"
            size="sm"
            onClick={resetError}
            className="flex items-center gap-2"
          >
            <RefreshCw className="h-4 w-4" />
            重试
          </Button>
          
          <Button
            variant="default"
            size="sm"
            onClick={() => window.location.href = '/'}
            className="flex items-center gap-2"
          >
            <Home className="h-4 w-4" />
            返回首页
          </Button>
        </div>
      </Card>
    </div>
  )
}

// 错误边界类组件
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error
    }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo)
    
    this.setState({
      error,
      errorInfo
    })

    // 这里可以添加错误报告逻辑
    // reportError(error, errorInfo)
  }

  resetError = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined })
  }

  render() {
    if (this.state.hasError) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback
      return (
        <FallbackComponent 
          error={this.state.error} 
          resetError={this.resetError}
        />
      )
    }

    return this.props.children
  }
}

// 网络错误组件
interface NetworkErrorProps {
  onRetry?: () => void
  title?: string
  description?: string
}

export function NetworkError({ 
  onRetry, 
  title = "网络连接失败",
  description = "请检查您的网络连接，然后重试。"
}: NetworkErrorProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12">
      <div className="w-16 h-16 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
        <AlertTriangle className="h-8 w-8 text-destructive" />
      </div>
      
      <h3 className="text-lg font-medium text-foreground mb-2">{title}</h3>
      <p className="text-sm text-muted-foreground mb-4 text-center max-w-md">
        {description}
      </p>
      
      {onRetry && (
        <Button onClick={onRetry} className="flex items-center gap-2">
          <RefreshCw className="h-4 w-4" />
          重试
        </Button>
      )}
    </div>
  )
}

// 404错误组件
export function NotFoundError() {
  return (
    <div className="flex flex-col items-center justify-center py-12">
      <div className="text-6xl font-bold text-muted-foreground mb-4">404</div>
      <h3 className="text-lg font-medium text-foreground mb-2">页面未找到</h3>
      <p className="text-sm text-muted-foreground mb-4 text-center max-w-md">
        抱歉，您访问的页面不存在或已被移动。
      </p>
      
      <Button 
        onClick={() => window.location.href = '/'}
        className="flex items-center gap-2"
      >
        <Home className="h-4 w-4" />
        返回首页
      </Button>
    </div>
  )
}

// 权限错误组件
export function PermissionError() {
  return (
    <div className="flex flex-col items-center justify-center py-12">
      <div className="w-16 h-16 rounded-full bg-warning/10 flex items-center justify-center mb-4">
        <AlertTriangle className="h-8 w-8 text-warning" />
      </div>
      
      <h3 className="text-lg font-medium text-foreground mb-2">访问受限</h3>
      <p className="text-sm text-muted-foreground mb-4 text-center max-w-md">
        您没有权限访问此页面。请联系管理员获取相应权限。
      </p>
      
      <Button 
        variant="outline"
        onClick={() => window.history.back()}
        className="flex items-center gap-2"
      >
        返回上一页
      </Button>
    </div>
  )
}
