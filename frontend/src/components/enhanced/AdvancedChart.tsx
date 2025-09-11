/**
 * 高级图表组件 - 支持多种图表类型和交互功能
 * 基于 Chart.js 和 D3.js 的高性能数据可视化
 */

import React, { useRef, useEffect, useState, useMemo } from 'react';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  Filler,
  ArcElement,
} from 'chart.js';
import { Line, Bar, Pie, Doughnut } from 'react-chartjs-2';
import { debounce } from 'lodash';
import './AdvancedChart.css';

// 注册 Chart.js 组件
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  Filler,
  ArcElement
);

export type ChartType = 'line' | 'bar' | 'pie' | 'doughnut' | 'area' | 'scatter' | 'candlestick';

export interface ChartDataPoint {
  x: string | number | Date;
  y: number;
  label?: string;
  color?: string;
}

export interface ChartSeries {
  name: string;
  data: ChartDataPoint[];
  color?: string;
  type?: ChartType;
  yAxisID?: string;
}

export interface AdvancedChartProps {
  type: ChartType;
  series: ChartSeries[];
  title?: string;
  subtitle?: string;
  width?: number;
  height?: number;
  responsive?: boolean;
  interactive?: boolean;
  showLegend?: boolean;
  showTooltip?: boolean;
  showGrid?: boolean;
  enableZoom?: boolean;
  enablePan?: boolean;
  enableBrush?: boolean;
  theme?: 'light' | 'dark';
  animation?: boolean;
  onDataPointClick?: (point: ChartDataPoint, seriesIndex: number) => void;
  onZoom?: (range: { start: number; end: number }) => void;
  onBrush?: (selection: { start: number; end: number }) => void;
}

const AdvancedChart: React.FC<AdvancedChartProps> = ({
  type,
  series,
  title,
  subtitle,
  width,
  height = 400,
  responsive = true,
  interactive = true,
  showLegend = true,
  showTooltip = true,
  showGrid = true,
  enableZoom = false,
  enablePan = false,
  enableBrush = false,
  theme = 'light',
  animation = true,
  onDataPointClick,
  onZoom,
  onBrush,
}) => {
  const chartRef = useRef<any>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 主题配置
  const themeConfig = useMemo(() => {
    const isDark = theme === 'dark';
    return {
      backgroundColor: isDark ? '#1a1a1a' : '#ffffff',
      textColor: isDark ? '#ffffff' : '#333333',
      gridColor: isDark ? '#404040' : '#e0e0e0',
      borderColor: isDark ? '#606060' : '#d0d0d0',
    };
  }, [theme]);

  // 颜色调色板
  const colorPalette = [
    '#667eea', '#764ba2', '#f093fb', '#f5576c',
    '#4facfe', '#00f2fe', '#43e97b', '#38f9d7',
    '#ffecd2', '#fcb69f', '#a8edea', '#fed6e3',
  ];

  // 处理数据格式转换
  const chartData = useMemo(() => {
    if (!series || series.length === 0) {
      return { labels: [], datasets: [] };
    }

    // 获取所有唯一的 x 值作为标签
    const allLabels = Array.from(
      new Set(
        series.flatMap(s => s.data.map(d => d.x))
      )
    ).sort();

    const datasets = series.map((s, index) => {
      const color = s.color || colorPalette[index % colorPalette.length];
      
      // 根据图表类型配置数据集
      const baseConfig = {
        label: s.name,
        data: allLabels.map(label => {
          const point = s.data.find(d => d.x === label);
          return point ? point.y : null;
        }),
        borderColor: color,
        backgroundColor: type === 'area' ? `${color}20` : color,
        borderWidth: 2,
        pointRadius: type === 'line' || type === 'area' ? 4 : 0,
        pointHoverRadius: 6,
        tension: type === 'line' || type === 'area' ? 0.4 : 0,
        fill: type === 'area',
      };

      // 特定图表类型的配置
      if (type === 'bar') {
        return {
          ...baseConfig,
          backgroundColor: `${color}80`,
          borderWidth: 1,
        };
      }

      if (type === 'pie' || type === 'doughnut') {
        return {
          ...baseConfig,
          data: s.data.map(d => d.y),
          backgroundColor: s.data.map((_, i) => 
            colorPalette[i % colorPalette.length]
          ),
          borderWidth: 2,
          borderColor: themeConfig.backgroundColor,
        };
      }

      return baseConfig;
    });

    return {
      labels: type === 'pie' || type === 'doughnut' 
        ? series[0]?.data.map(d => d.label || d.x) || []
        : allLabels,
      datasets,
    };
  }, [series, type, colorPalette, themeConfig]);

  // 图表配置选项
  const chartOptions = useMemo(() => {
    const baseOptions = {
      responsive,
      maintainAspectRatio: !height,
      interaction: {
        intersect: false,
        mode: 'index' as const,
      },
      plugins: {
        legend: {
          display: showLegend,
          position: 'top' as const,
          labels: {
            color: themeConfig.textColor,
            usePointStyle: true,
            padding: 20,
          },
        },
        tooltip: {
          enabled: showTooltip,
          backgroundColor: themeConfig.backgroundColor,
          titleColor: themeConfig.textColor,
          bodyColor: themeConfig.textColor,
          borderColor: themeConfig.borderColor,
          borderWidth: 1,
          cornerRadius: 8,
          displayColors: true,
          callbacks: {
            label: (context: any) => {
              const label = context.dataset.label || '';
              const value = context.parsed.y || context.parsed;
              return `${label}: ${typeof value === 'number' ? value.toLocaleString() : value}`;
            },
          },
        },
        title: {
          display: !!title,
          text: title,
          color: themeConfig.textColor,
          font: {
            size: 18,
            weight: 'bold' as const,
          },
          padding: 20,
        },
      },
      animation: animation ? {
        duration: 1000,
        easing: 'easeInOutQuart' as const,
      } : false,
      onClick: (event: any, elements: any[]) => {
        if (interactive && onDataPointClick && elements.length > 0) {
          const element = elements[0];
          const datasetIndex = element.datasetIndex;
          const dataIndex = element.index;
          const point = series[datasetIndex]?.data[dataIndex];
          if (point) {
            onDataPointClick(point, datasetIndex);
          }
        }
      },
    };

    // 添加坐标轴配置（非饼图）
    if (type !== 'pie' && type !== 'doughnut') {
      return {
        ...baseOptions,
        scales: {
          x: {
            display: true,
            grid: {
              display: showGrid,
              color: themeConfig.gridColor,
            },
            ticks: {
              color: themeConfig.textColor,
            },
          },
          y: {
            display: true,
            grid: {
              display: showGrid,
              color: themeConfig.gridColor,
            },
            ticks: {
              color: themeConfig.textColor,
              callback: (value: any) => {
                return typeof value === 'number' ? value.toLocaleString() : value;
              },
            },
          },
        },
      };
    }

    return baseOptions;
  }, [
    responsive, height, showLegend, showTooltip, showGrid, theme, title,
    animation, interactive, onDataPointClick, series, type, themeConfig
  ]);

  // 渲染对应的图表组件
  const renderChart = () => {
    const commonProps = {
      ref: chartRef,
      data: chartData,
      options: chartOptions,
      width,
      height,
    };

    switch (type) {
      case 'line':
      case 'area':
        return <Line {...commonProps} />;
      case 'bar':
        return <Bar {...commonProps} />;
      case 'pie':
        return <Pie {...commonProps} />;
      case 'doughnut':
        return <Doughnut {...commonProps} />;
      default:
        return <Line {...commonProps} />;
    }
  };

  // 错误状态
  if (error) {
    return (
      <div className="chart-error" style={{ height }}>
        <div className="error-content">
          <span className="error-icon">⚠️</span>
          <p>图表加载失败</p>
          <small>{error}</small>
        </div>
      </div>
    );
  }

  // 加载状态
  if (isLoading) {
    return (
      <div className="chart-loading" style={{ height }}>
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>加载中...</p>
        </div>
      </div>
    );
  }

  // 无数据状态
  if (!series || series.length === 0 || series.every(s => s.data.length === 0)) {
    return (
      <div className="chart-no-data" style={{ height }}>
        <div className="no-data-content">
          <span className="no-data-icon">📊</span>
          <p>暂无数据</p>
          <small>请检查数据源或筛选条件</small>
        </div>
      </div>
    );
  }

  return (
    <div className={`advanced-chart-container ${theme}`}>
      {subtitle && (
        <div className="chart-subtitle">
          {subtitle}
        </div>
      )}
      
      <div className="chart-wrapper" style={{ height }}>
        {renderChart()}
      </div>

      {/* 图表工具栏 */}
      {interactive && (
        <div className="chart-toolbar">
          <div className="chart-controls">
            {enableZoom && (
              <button className="chart-btn" title="缩放">
                🔍
              </button>
            )}
            {enablePan && (
              <button className="chart-btn" title="平移">
                ✋
              </button>
            )}
            {enableBrush && (
              <button className="chart-btn" title="选择">
                📐
              </button>
            )}
            <button className="chart-btn" title="重置">
              🔄
            </button>
            <button className="chart-btn" title="下载">
              💾
            </button>
          </div>
          
          <div className="chart-info">
            <span>数据点: {series.reduce((sum, s) => sum + s.data.length, 0)}</span>
            <span>系列: {series.length}</span>
          </div>
        </div>
      )}
    </div>
  );
};

export default AdvancedChart;
