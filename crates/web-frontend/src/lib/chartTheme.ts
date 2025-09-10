// Supabase风格图表主题配置

// 主色彩配置
export const CHART_COLORS = {
  primary: '#3ECF8E',      // Supabase绿
  secondary: '#249361',    // 深绿
  accent: '#4FD1A7',       // 浅绿
  success: '#10B981',      // 成功绿
  warning: '#F59E0B',      // 警告黄
  error: '#EF4444',        // 错误红
  info: '#3B82F6',         // 信息蓝
  muted: '#6B7280',        // 静音灰
  background: '#FFFFFF',   // 背景白
  surface: '#F9FAFB',      // 表面灰
  border: '#E5E7EB',       // 边框灰
  text: '#111827',         // 文字黑
  textMuted: '#6B7280',    // 静音文字
}

// 深色模式色彩
export const CHART_COLORS_DARK = {
  primary: '#3ECF8E',
  secondary: '#249361',
  accent: '#4FD1A7',
  success: '#10B981',
  warning: '#F59E0B',
  error: '#EF4444',
  info: '#3B82F6',
  muted: '#9CA3AF',
  background: '#0F1419',   // 深色背景
  surface: '#1F2937',      // 深色表面
  border: '#374151',       // 深色边框
  text: '#F9FAFB',         // 深色文字
  textMuted: '#9CA3AF',    // 深色静音文字
}

// 图表色彩序列
export const CHART_COLOR_SEQUENCE = [
  '#3ECF8E', // 主绿色
  '#3B82F6', // 蓝色
  '#F59E0B', // 黄色
  '#EF4444', // 红色
  '#8B5CF6', // 紫色
  '#06B6D4', // 青色
  '#84CC16', // 柠檬绿
  '#F97316', // 橙色
  '#EC4899', // 粉色
  '#6366F1', // 靛蓝
]

// Recharts主题配置
export const getRechartsTheme = (isDark = false) => {
  const colors = isDark ? CHART_COLORS_DARK : CHART_COLORS
  
  return {
    background: colors.background,
    text: colors.text,
    grid: colors.border,
    tooltip: {
      backgroundColor: colors.surface,
      border: `1px solid ${colors.border}`,
      borderRadius: '8px',
      boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
      color: colors.text,
    },
    legend: {
      color: colors.textMuted,
    },
    axis: {
      stroke: colors.border,
      fontSize: 12,
      fontFamily: 'Inter, sans-serif',
    },
    cartesianGrid: {
      stroke: colors.border,
      strokeDasharray: '3 3',
      opacity: 0.5,
    }
  }
}

// Chart.js主题配置
export const getChartJsTheme = (isDark = false) => {
  const colors = isDark ? CHART_COLORS_DARK : CHART_COLORS
  
  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        labels: {
          color: colors.textMuted,
          font: {
            family: 'Inter, sans-serif',
            size: 12,
          },
          usePointStyle: true,
          pointStyle: 'circle',
        },
      },
      tooltip: {
        backgroundColor: colors.surface,
        titleColor: colors.text,
        bodyColor: colors.text,
        borderColor: colors.border,
        borderWidth: 1,
        cornerRadius: 8,
        displayColors: true,
        font: {
          family: 'Inter, sans-serif',
        },
      },
    },
    scales: {
      x: {
        grid: {
          color: colors.border,
          borderDash: [3, 3],
        },
        ticks: {
          color: colors.textMuted,
          font: {
            family: 'Inter, sans-serif',
            size: 11,
          },
        },
        border: {
          color: colors.border,
        },
      },
      y: {
        grid: {
          color: colors.border,
          borderDash: [3, 3],
        },
        ticks: {
          color: colors.textMuted,
          font: {
            family: 'Inter, sans-serif',
            size: 11,
          },
        },
        border: {
          color: colors.border,
        },
      },
    },
    elements: {
      point: {
        radius: 4,
        hoverRadius: 6,
        borderWidth: 2,
      },
      line: {
        borderWidth: 2,
        tension: 0.1,
      },
      bar: {
        borderRadius: 4,
        borderSkipped: false,
      },
    },
  }
}

// 获取图表颜色
export const getChartColor = (index: number, opacity = 1) => {
  const color = CHART_COLOR_SEQUENCE[index % CHART_COLOR_SEQUENCE.length]
  if (opacity === 1) return color
  
  // 转换为rgba格式
  const hex = color.replace('#', '')
  const r = parseInt(hex.substr(0, 2), 16)
  const g = parseInt(hex.substr(2, 2), 16)
  const b = parseInt(hex.substr(4, 2), 16)
  
  return `rgba(${r}, ${g}, ${b}, ${opacity})`
}

// 渐变色生成器
export const createGradient = (ctx: CanvasRenderingContext2D, color: string, direction = 'vertical') => {
  const gradient = direction === 'vertical' 
    ? ctx.createLinearGradient(0, 0, 0, 400)
    : ctx.createLinearGradient(0, 0, 400, 0)
  
  gradient.addColorStop(0, getChartColor(CHART_COLOR_SEQUENCE.indexOf(color), 0.8))
  gradient.addColorStop(1, getChartColor(CHART_COLOR_SEQUENCE.indexOf(color), 0.1))
  
  return gradient
}

// 图表动画配置
export const CHART_ANIMATIONS = {
  duration: 750,
  easing: 'easeInOutQuart',
  delay: (context: any) => context.dataIndex * 50,
}

// 响应式断点
export const CHART_BREAKPOINTS = {
  mobile: 480,
  tablet: 768,
  desktop: 1024,
  large: 1280,
}

// 获取响应式图表配置
export const getResponsiveChartConfig = (width: number) => {
  if (width < CHART_BREAKPOINTS.mobile) {
    return {
      legend: { display: false },
      scales: {
        x: { ticks: { maxTicksLimit: 4 } },
        y: { ticks: { maxTicksLimit: 5 } },
      },
    }
  } else if (width < CHART_BREAKPOINTS.tablet) {
    return {
      legend: { position: 'bottom' as const },
      scales: {
        x: { ticks: { maxTicksLimit: 6 } },
        y: { ticks: { maxTicksLimit: 6 } },
      },
    }
  } else {
    return {
      legend: { position: 'top' as const },
      scales: {
        x: { ticks: { maxTicksLimit: 10 } },
        y: { ticks: { maxTicksLimit: 8 } },
      },
    }
  }
}

// 图表导出配置
export const EXPORT_CONFIG = {
  png: {
    backgroundColor: '#ffffff',
    pixelRatio: 2,
  },
  svg: {
    backgroundColor: 'transparent',
  },
  pdf: {
    format: 'A4' as const,
    orientation: 'landscape' as const,
    margin: 20,
  },
}

// 图表类型配置
export const CHART_TYPES = {
  line: {
    icon: '📈',
    name: '折线图',
    description: '显示数据随时间的变化趋势',
  },
  bar: {
    icon: '📊',
    name: '柱状图',
    description: '比较不同类别的数据',
  },
  pie: {
    icon: '🥧',
    name: '饼图',
    description: '显示数据的组成比例',
  },
  area: {
    icon: '📈',
    name: '面积图',
    description: '强调数据量的累积变化',
  },
  scatter: {
    icon: '⚪',
    name: '散点图',
    description: '显示两个变量之间的关系',
  },
  radar: {
    icon: '🕸️',
    name: '雷达图',
    description: '多维数据的可视化比较',
  },
}
