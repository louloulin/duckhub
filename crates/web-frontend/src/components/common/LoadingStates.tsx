import { cn } from '@/lib/utils'

interface SkeletonProps {
  className?: string
  style?: React.CSSProperties
}

export function Skeleton({ className, style }: SkeletonProps) {
  return (
    <div
      className={cn(
        "animate-pulse rounded-md bg-muted skeleton-loading",
        className
      )}
      style={style}
    />
  )
}

// 卡片骨架屏
export function CardSkeleton() {
  return (
    <div className="rounded-lg border bg-card p-6 shadow-sm">
      <div className="space-y-4">
        <div className="flex items-center space-x-4">
          <Skeleton className="h-12 w-12 rounded-full" />
          <div className="space-y-2">
            <Skeleton className="h-4 w-[200px]" />
            <Skeleton className="h-4 w-[160px]" />
          </div>
        </div>
        <div className="space-y-2">
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-[80%]" />
          <Skeleton className="h-4 w-[60%]" />
        </div>
      </div>
    </div>
  )
}

// 表格骨架屏
export function TableSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div className="rounded-lg border bg-card">
      <div className="border-b p-4">
        <div className="flex items-center justify-between">
          <Skeleton className="h-6 w-[200px]" />
          <Skeleton className="h-8 w-[100px]" />
        </div>
      </div>
      <div className="p-4">
        <div className="space-y-3">
          {Array.from({ length: rows }).map((_, i) => (
            <div key={i} className="flex items-center space-x-4">
              <Skeleton className="h-4 w-[100px]" />
              <Skeleton className="h-4 w-[150px]" />
              <Skeleton className="h-4 w-[120px]" />
              <Skeleton className="h-4 w-[80px]" />
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

// 图表骨架屏
export function ChartSkeleton() {
  return (
    <div className="rounded-lg border bg-card p-6 shadow-sm">
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <Skeleton className="h-6 w-[150px]" />
          <Skeleton className="h-8 w-[80px]" />
        </div>
        <div className="space-y-2">
          <div className="flex items-end space-x-2 h-[200px]">
            {Array.from({ length: 7 }).map((_, i) => (
              <Skeleton 
                key={i} 
                className="flex-1" 
                style={{ height: `${Math.random() * 80 + 20}%` }}
              />
            ))}
          </div>
        </div>
        <div className="flex justify-center space-x-4">
          <div className="flex items-center space-x-2">
            <Skeleton className="h-3 w-3 rounded-full" />
            <Skeleton className="h-3 w-[60px]" />
          </div>
          <div className="flex items-center space-x-2">
            <Skeleton className="h-3 w-3 rounded-full" />
            <Skeleton className="h-3 w-[60px]" />
          </div>
        </div>
      </div>
    </div>
  )
}

// 页面加载骨架屏
export function PageSkeleton() {
  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="space-y-2">
        <Skeleton className="h-8 w-[300px]" />
        <Skeleton className="h-4 w-[500px]" />
      </div>
      
      {/* 统计卡片 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="rounded-lg border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <Skeleton className="h-4 w-[100px]" />
              <Skeleton className="h-8 w-[80px]" />
              <Skeleton className="h-3 w-[60px]" />
            </div>
          </div>
        ))}
      </div>
      
      {/* 主要内容区域 */}
      <div className="grid gap-6 lg:grid-cols-2">
        <ChartSkeleton />
        <TableSkeleton />
      </div>
    </div>
  )
}

// 加载指示器
interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg'
  className?: string
}

export function LoadingSpinner({ size = 'md', className }: LoadingSpinnerProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-6 w-6',
    lg: 'h-8 w-8'
  }

  return (
    <div
      className={cn(
        "animate-spin rounded-full border-2 border-muted border-t-primary",
        sizeClasses[size],
        className
      )}
    />
  )
}

// 加载状态组件
interface LoadingStateProps {
  title?: string
  description?: string
  className?: string
}

export function LoadingState({ 
  title = "加载中...", 
  description = "请稍候，正在获取数据",
  className 
}: LoadingStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center py-12", className)}>
      <LoadingSpinner size="lg" className="mb-4" />
      <h3 className="text-lg font-medium text-foreground">{title}</h3>
      <p className="text-sm text-muted-foreground mt-1">{description}</p>
    </div>
  )
}

// 空状态组件
interface EmptyStateProps {
  title: string
  description?: string
  action?: React.ReactNode
  icon?: React.ReactNode
  className?: string
}

export function EmptyState({ 
  title, 
  description, 
  action, 
  icon,
  className 
}: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center py-12", className)}>
      {icon && <div className="mb-4 text-muted-foreground">{icon}</div>}
      <h3 className="text-lg font-medium text-foreground">{title}</h3>
      {description && (
        <p className="text-sm text-muted-foreground mt-1 text-center max-w-md">
          {description}
        </p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  )
}
