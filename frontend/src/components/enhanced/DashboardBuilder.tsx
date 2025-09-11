/**
 * 拖拽式仪表板构建器
 * 支持组件拖拽、布局调整和实时预览
 */

import React, { useState, useCallback, useMemo } from 'react';
import { DndProvider, useDrag, useDrop } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import { Responsive, WidthProvider } from 'react-grid-layout';
import VirtualizedTable from './VirtualizedTable';
import AdvancedChart from './AdvancedChart';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';
import './DashboardBuilder.css';

const ResponsiveGridLayout = WidthProvider(Responsive);

// 组件类型定义
export type WidgetType = 'chart' | 'table' | 'metric' | 'text' | 'filter' | 'map';

export interface Widget {
  id: string;
  type: WidgetType;
  title: string;
  config: any;
  layout: {
    x: number;
    y: number;
    w: number;
    h: number;
  };
}

export interface DashboardConfig {
  id: string;
  name: string;
  description?: string;
  widgets: Widget[];
  layout: any[];
  theme: 'light' | 'dark';
  autoRefresh?: number;
}

interface DashboardBuilderProps {
  initialConfig?: DashboardConfig;
  onSave?: (config: DashboardConfig) => void;
  onPreview?: (config: DashboardConfig) => void;
  readOnly?: boolean;
}

// 可拖拽的组件面板项
const DraggableWidget: React.FC<{
  type: WidgetType;
  title: string;
  icon: string;
  description: string;
}> = ({ type, title, icon, description }) => {
  const [{ isDragging }, drag] = useDrag({
    type: 'widget',
    item: { type, title },
    collect: (monitor) => ({
      isDragging: monitor.isDragging(),
    }),
  });

  return (
    <div
      ref={drag}
      className={`widget-item ${isDragging ? 'dragging' : ''}`}
      title={description}
    >
      <span className="widget-icon">{icon}</span>
      <span className="widget-title">{title}</span>
    </div>
  );
};

// 可放置的画布区域
const DropCanvas: React.FC<{
  widgets: Widget[];
  onAddWidget: (type: WidgetType, position: { x: number; y: number }) => void;
  onLayoutChange: (layout: any[]) => void;
  onWidgetUpdate: (id: string, config: any) => void;
  onWidgetDelete: (id: string) => void;
  readOnly?: boolean;
}> = ({ widgets, onAddWidget, onLayoutChange, onWidgetUpdate, onWidgetDelete, readOnly }) => {
  const [{ isOver }, drop] = useDrop({
    accept: 'widget',
    drop: (item: { type: WidgetType }, monitor) => {
      const offset = monitor.getDropResult();
      if (offset) {
        onAddWidget(item.type, { x: 0, y: 0 });
      }
    },
    collect: (monitor) => ({
      isOver: monitor.isOver(),
    }),
  });

  const layouts = useMemo(() => {
    return {
      lg: widgets.map(widget => ({
        i: widget.id,
        x: widget.layout.x,
        y: widget.layout.y,
        w: widget.layout.w,
        h: widget.layout.h,
        minW: 2,
        minH: 2,
      })),
    };
  }, [widgets]);

  const renderWidget = (widget: Widget) => {
    switch (widget.type) {
      case 'chart':
        return (
          <AdvancedChart
            type={widget.config.chartType || 'line'}
            series={widget.config.series || []}
            title={widget.title}
            height={300}
            responsive={true}
          />
        );
      case 'table':
        return (
          <VirtualizedTable
            data={widget.config.data || []}
            columns={widget.config.columns || []}
            height={300}
          />
        );
      case 'metric':
        return (
          <div className="metric-widget">
            <div className="metric-value">{widget.config.value || '0'}</div>
            <div className="metric-label">{widget.config.label || 'Metric'}</div>
            <div className="metric-change">
              <span className={`change ${widget.config.changeType || 'neutral'}`}>
                {widget.config.change || '0%'}
              </span>
            </div>
          </div>
        );
      case 'text':
        return (
          <div className="text-widget">
            <div dangerouslySetInnerHTML={{ __html: widget.config.content || '' }} />
          </div>
        );
      default:
        return (
          <div className="placeholder-widget">
            <span>未知组件类型: {widget.type}</span>
          </div>
        );
    }
  };

  return (
    <div
      ref={drop}
      className={`drop-canvas ${isOver ? 'drop-over' : ''}`}
    >
      <ResponsiveGridLayout
        className="layout"
        layouts={layouts}
        breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
        cols={{ lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 }}
        rowHeight={60}
        onLayoutChange={(layout) => onLayoutChange(layout)}
        isDraggable={!readOnly}
        isResizable={!readOnly}
        margin={[16, 16]}
        containerPadding={[16, 16]}
      >
        {widgets.map((widget) => (
          <div key={widget.id} className="widget-container">
            <div className="widget-header">
              <span className="widget-title">{widget.title}</span>
              {!readOnly && (
                <div className="widget-actions">
                  <button
                    className="widget-action edit"
                    onClick={() => onWidgetUpdate(widget.id, widget.config)}
                    title="编辑"
                  >
                    ✏️
                  </button>
                  <button
                    className="widget-action delete"
                    onClick={() => onWidgetDelete(widget.id)}
                    title="删除"
                  >
                    🗑️
                  </button>
                </div>
              )}
            </div>
            <div className="widget-content">
              {renderWidget(widget)}
            </div>
          </div>
        ))}
      </ResponsiveGridLayout>
    </div>
  );
};

const DashboardBuilder: React.FC<DashboardBuilderProps> = ({
  initialConfig,
  onSave,
  onPreview,
  readOnly = false,
}) => {
  const [config, setConfig] = useState<DashboardConfig>(
    initialConfig || {
      id: `dashboard_${Date.now()}`,
      name: '新建仪表板',
      widgets: [],
      layout: [],
      theme: 'light',
    }
  );

  const [selectedWidget, setSelectedWidget] = useState<string | null>(null);
  const [showWidgetPanel, setShowWidgetPanel] = useState(!readOnly);

  // 可用组件列表
  const availableWidgets = [
    { type: 'chart' as WidgetType, title: '图表', icon: '📊', description: '各种类型的数据图表' },
    { type: 'table' as WidgetType, title: '表格', icon: '📋', description: '数据表格显示' },
    { type: 'metric' as WidgetType, title: '指标', icon: '📈', description: '关键指标卡片' },
    { type: 'text' as WidgetType, title: '文本', icon: '📝', description: '文本内容块' },
    { type: 'filter' as WidgetType, title: '筛选器', icon: '🔍', description: '数据筛选控件' },
    { type: 'map' as WidgetType, title: '地图', icon: '🗺️', description: '地理数据可视化' },
  ];

  // 添加组件
  const handleAddWidget = useCallback((type: WidgetType, position: { x: number; y: number }) => {
    const newWidget: Widget = {
      id: `widget_${Date.now()}`,
      type,
      title: `新${availableWidgets.find(w => w.type === type)?.title || '组件'}`,
      config: {},
      layout: {
        x: position.x,
        y: position.y,
        w: type === 'metric' ? 3 : 6,
        h: type === 'metric' ? 2 : 4,
      },
    };

    setConfig(prev => ({
      ...prev,
      widgets: [...prev.widgets, newWidget],
    }));
  }, [availableWidgets]);

  // 更新布局
  const handleLayoutChange = useCallback((layout: any[]) => {
    setConfig(prev => ({
      ...prev,
      widgets: prev.widgets.map(widget => {
        const layoutItem = layout.find(item => item.i === widget.id);
        if (layoutItem) {
          return {
            ...widget,
            layout: {
              x: layoutItem.x,
              y: layoutItem.y,
              w: layoutItem.w,
              h: layoutItem.h,
            },
          };
        }
        return widget;
      }),
      layout,
    }));
  }, []);

  // 更新组件配置
  const handleWidgetUpdate = useCallback((id: string, newConfig: any) => {
    setConfig(prev => ({
      ...prev,
      widgets: prev.widgets.map(widget =>
        widget.id === id ? { ...widget, config: newConfig } : widget
      ),
    }));
  }, []);

  // 删除组件
  const handleWidgetDelete = useCallback((id: string) => {
    setConfig(prev => ({
      ...prev,
      widgets: prev.widgets.filter(widget => widget.id !== id),
    }));
  }, []);

  // 保存配置
  const handleSave = useCallback(() => {
    onSave?.(config);
  }, [config, onSave]);

  // 预览
  const handlePreview = useCallback(() => {
    onPreview?.(config);
  }, [config, onPreview]);

  return (
    <DndProvider backend={HTML5Backend}>
      <div className={`dashboard-builder ${config.theme}`}>
        {/* 工具栏 */}
        <div className="builder-toolbar">
          <div className="toolbar-left">
            <input
              type="text"
              value={config.name}
              onChange={(e) => setConfig(prev => ({ ...prev, name: e.target.value }))}
              className="dashboard-name-input"
              placeholder="仪表板名称"
              readOnly={readOnly}
            />
            <select
              value={config.theme}
              onChange={(e) => setConfig(prev => ({ ...prev, theme: e.target.value as 'light' | 'dark' }))}
              className="theme-selector"
              disabled={readOnly}
            >
              <option value="light">浅色主题</option>
              <option value="dark">深色主题</option>
            </select>
          </div>
          
          <div className="toolbar-right">
            {!readOnly && (
              <>
                <button
                  className="toolbar-btn"
                  onClick={() => setShowWidgetPanel(!showWidgetPanel)}
                >
                  {showWidgetPanel ? '隐藏组件' : '显示组件'}
                </button>
                <button className="toolbar-btn primary" onClick={handleSave}>
                  保存
                </button>
              </>
            )}
            <button className="toolbar-btn" onClick={handlePreview}>
              预览
            </button>
          </div>
        </div>

        <div className="builder-content">
          {/* 组件面板 */}
          {showWidgetPanel && !readOnly && (
            <div className="widget-panel">
              <h3>组件库</h3>
              <div className="widget-list">
                {availableWidgets.map((widget) => (
                  <DraggableWidget
                    key={widget.type}
                    type={widget.type}
                    title={widget.title}
                    icon={widget.icon}
                    description={widget.description}
                  />
                ))}
              </div>
            </div>
          )}

          {/* 画布区域 */}
          <div className="canvas-area">
            <DropCanvas
              widgets={config.widgets}
              onAddWidget={handleAddWidget}
              onLayoutChange={handleLayoutChange}
              onWidgetUpdate={handleWidgetUpdate}
              onWidgetDelete={handleWidgetDelete}
              readOnly={readOnly}
            />
          </div>
        </div>

        {/* 状态栏 */}
        <div className="builder-statusbar">
          <span>组件数量: {config.widgets.length}</span>
          <span>最后修改: {new Date().toLocaleString()}</span>
          {config.autoRefresh && (
            <span>自动刷新: {config.autoRefresh}秒</span>
          )}
        </div>
      </div>
    </DndProvider>
  );
};

export default DashboardBuilder;
