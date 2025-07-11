# DuckHub 前后端数据对接详细计划 (plan5.md)

## 📊 项目概述

### 🎯 目标
将前端所有mock数据替换为真实的后端API调用，实现完整的前后端数据对接，消除所有模拟数据，建立生产级的数据流。

### 📈 当前状态分析
- **后端API**: ✅ 已完成28个API端点实现
- **前端组件**: ⚠️ 大量使用mock数据，需要全面对接
- **数据流**: ❌ 前后端数据格式不完全匹配
- **状态管理**: ⚠️ Redux store需要适配真实API响应

## 🔍 Mock数据全面分析

### 📋 Mock数据使用统计
| 页面/组件 | Mock数据类型 | 数量 | 优先级 |
|-----------|-------------|------|--------|
| Dashboard | 系统指标、DuckLake指标、图表数据 | 15+ | 🔴 高 |
| DataExplorer | 表信息、Schema、数据预览 | 10+ | 🔴 高 |
| QueryAnalytics | 查询结果、时间旅行、比较数据 | 8+ | 🟡 中 |
| AIAgent | 对话消息、推荐建议、洞察数据 | 12+ | 🟡 中 |
| DuckLakeManager | 数据库、快照、版本、指标 | 20+ | 🔴 高 |
| Settings | 配置数据 | 5+ | 🟢 低 |

### 🎯 关键Mock数据识别

#### 1. Dashboard页面Mock数据
```typescript
// 位置: src/pages/Dashboard.tsx
const [duckLakeMetrics] = useState<DuckLakeMetrics>({
  activeDatabases: 3,
  totalSnapshots: 127,
  timeTravelQueries: 1250,
  schemaEvolutions: 15,
  queryPerformance: [...], // 6个数据点
  snapshotActivity: [...], // 时间序列数据
  storageUsage: [...],     // 存储使用数据
  transactionStats: {...}  // 事务统计
})
```

#### 2. DatabasePanel组件Mock数据
```typescript
// 位置: src/components/ducklake/DatabasePanel.tsx
const mockDatabases: DuckLakeDatabase[] = [
  {
    id: '1', name: 'financial_data', path: '/data/ducklake/financial_data.db',
    status: 'connected', size: '2.3 GB', tables: 15,
    lastAccessed: '2分钟前', connections: 3,
    description: '金融交易数据主库'
  },
  // ... 3个数据库记录
]
```

#### 3. SnapshotBrowser组件Mock数据
```typescript
// 位置: src/components/ducklake/SnapshotBrowser.tsx
const mockSnapshots: Snapshot[] = [
  {
    id: '1', version: 127, timestamp: '2024-01-11 14:30:25',
    database: 'financial_data', size: '2.3 GB', tables: 15,
    description: '日终数据快照', tags: ['daily', 'production'],
    changes: 1250, author: 'system', type: 'automatic',
    parentVersion: 126, checksum: 'sha256:a1b2c3d4...',
    metadata: { rowCount: 1250000, schemaVersion: 5, compressionRatio: 0.65 }
  },
  // ... 5个快照记录
]
```

#### 4. DataExplorer页面Mock数据
```typescript
// 位置: src/pages/DataExplorer.tsx
const tables: TableInfo[] = [
  {
    name: 'transactions', rows: 1250000, size: '2.3 GB',
    schema_version: 5, last_modified: '2024-01-11 14:30:25',
    description: '交易记录表'
  },
  // ... 4个表记录
]

const tableSchema: SchemaColumn[] = [
  { column: 'id', type: 'BIGINT', nullable: false, key: 'PRIMARY', comment: '主键ID' },
  // ... 6个字段定义
]

const schemaVersions: SchemaVersion[] = [
  {
    version: 5, timestamp: '2024-01-11 14:30:25', author: 'admin',
    description: '添加交易状态字段', compatibility: 'backward',
    changes: [{ type: 'add_column', table: 'transactions', ... }]
  },
  // ... 版本历史记录
]
```

## 🔄 API路径映射分析

### 📡 前端API调用 vs 后端端点对比

| 前端API调用 | 后端实际端点 | 状态 | 数据格式匹配 |
|-------------|-------------|------|-------------|
| `dashboardAPI.getMetrics()` | `/api/v1/dashboard/metrics` | ✅ | ⚠️ 部分匹配 |
| `dashboardAPI.getQueryTrends()` | `/api/v1/dashboard/query-trends` | ✅ | ⚠️ 需调整 |
| `dashboardAPI.getSystemHealth()` | `/api/v1/dashboard/system-health` | ✅ | ✅ 匹配 |
| `dataExplorerAPI.getTables()` | `/api/v1/data/tables` | ✅ | ⚠️ 需调整 |
| `dataExplorerAPI.getTableSchema()` | `/api/v1/data/tables/{name}/schema` | ✅ | ⚠️ 需调整 |
| `dataExplorerAPI.getTableData()` | `/api/v1/data/tables/{name}/data` | ✅ | ✅ 匹配 |
| `aiAgentAPI.sendMessage()` | `/api/v1/ai/chat` | ✅ | ⚠️ 需调整 |
| `queryAPI.execute()` | `/api/v1/query/execute` | ✅ | ✅ 匹配 |

### ❌ 缺失的API端点

| 前端需求 | 缺失的后端端点 | 优先级 |
|----------|---------------|--------|
| DuckLake数据库管理 | `/api/v1/ducklake/databases` | 🔴 高 |
| 快照管理 | `/api/v1/ducklake/snapshots` | 🔴 高 |
| 版本控制 | `/api/v1/ducklake/versions` | 🔴 高 |
| DuckLake指标 | `/api/v1/ducklake/metrics` | 🔴 高 |
| 时间旅行查询 | `/api/v1/query/time-travel` | 🟡 中 |
| Schema演进 | `/api/v1/ducklake/schema/evolution` | 🟡 中 |
| 配置管理 | `/api/v1/system/config` | ✅ 已有 |

## 📋 详细对接TODO清单

### 🎯 Phase 1: 核心数据流对接 (优先级: 🔴 高)

#### 1.1 Dashboard数据对接
**目标**: 替换Dashboard页面所有mock数据

**任务清单**:
- [ ] **1.1.1** 修改`dashboardSlice.ts`适配后端数据格式
  - 调整`DashboardMetrics`接口匹配后端响应
  - 修改`ChartData`接口支持后端时间序列格式
  - 更新`SystemHealth`接口字段名称

- [ ] **1.1.2** 创建DuckLake专项指标API
  - 后端新增`/api/v1/ducklake/metrics`端点
  - 返回`activeDatabases`, `totalSnapshots`, `timeTravelQueries`等
  - 支持时间范围查询参数

- [ ] **1.1.3** 更新Dashboard组件
  - 移除硬编码的`duckLakeMetrics`状态
  - 集成Redux store获取真实数据
  - 添加加载状态和错误处理

**数据格式对接**:
```typescript
// 前端期望格式
interface DuckLakeMetrics {
  activeDatabases: number
  totalSnapshots: number
  timeTravelQueries: number
  schemaEvolutions: number
  queryPerformance: Array<{
    time: string
    version: number
    avgResponseTime: number
    throughput: number
  }>
}

// 后端需要提供的格式
interface DuckLakeMetricsResponse {
  active_databases: number
  total_snapshots: number
  time_travel_queries: number
  schema_evolutions: number
  query_performance: Array<{
    timestamp: string
    version: number
    avg_response_time: number
    throughput: number
  }>
}
```

#### 1.2 数据探索对接
**目标**: 替换DataExplorer页面所有mock数据

**任务清单**:
- [ ] **1.2.1** 扩展数据探索API响应格式
  - 后端`/api/v1/data/tables`返回完整表信息
  - 包含`rows`, `size`, `schema_version`, `last_modified`, `description`
  - 支持分页和搜索参数

- [ ] **1.2.2** 增强表Schema API
  - 后端`/api/v1/data/tables/{name}/schema`返回详细字段信息
  - 包含`column`, `type`, `nullable`, `key`, `default_value`, `comment`
  - 支持Schema版本历史查询

- [ ] **1.2.3** 创建Schema演进API
  - 新增`/api/v1/ducklake/schema/evolution/{table}`端点
  - 返回Schema版本历史和变更详情
  - 支持版本比较功能

- [ ] **1.2.4** 更新DataExplorer组件
  - 移除所有硬编码的表数据和Schema数据
  - 集成真实API调用
  - 添加数据加载和错误状态处理

#### 1.3 DuckLake管理器对接
**目标**: 替换DuckLakeManager及其子组件的所有mock数据

**任务清单**:
- [ ] **1.3.1** 创建数据库管理API
  - 新增`/api/v1/ducklake/databases`端点
  - 支持GET(列表)、POST(创建)、PUT(更新)、DELETE(删除)
  - 返回数据库连接状态、大小、表数量等信息

- [ ] **1.3.2** 创建快照管理API
  - 新增`/api/v1/ducklake/snapshots`端点
  - 支持快照列表、创建、删除、恢复操作
  - 返回快照元数据、大小、变更统计等

- [ ] **1.3.3** 创建版本控制API
  - 新增`/api/v1/ducklake/versions`端点
  - 支持版本历史查询、回滚操作
  - 返回版本变更详情和影响分析

- [ ] **1.3.4** 更新DuckLake组件
  - `DatabasePanel.tsx`: 移除mock数据，集成数据库API
  - `SnapshotBrowser.tsx`: 移除mock数据，集成快照API
  - `VersionControl.tsx`: 移除mock数据，集成版本API
  - `DuckLakeMetrics.tsx`: 移除mock数据，集成指标API

### 🎯 Phase 2: 高级功能对接 (优先级: 🟡 中)

#### 2.1 查询分析对接
**目标**: 替换QueryAnalytics页面的时间旅行和比较功能

**任务清单**:
- [ ] **2.1.1** 创建时间旅行查询API
  - 新增`/api/v1/query/time-travel`端点
  - 支持版本号、时间戳、时间范围查询
  - 返回历史数据和查询元数据

- [ ] **2.1.2** 创建查询比较API
  - 新增`/api/v1/query/compare`端点
  - 支持两个查询结果的差异分析
  - 返回数据变更统计和详细差异

- [ ] **2.1.3** 更新QueryAnalytics组件
  - 移除模拟的时间旅行查询逻辑
  - 集成真实的时间旅行API
  - 实现真实的查询结果比较功能

#### 2.2 AI助手对接
**目标**: 替换AIAgent页面的对话和推荐数据

**任务清单**:
- [ ] **2.2.1** 完善AI对话API
  - 优化`/api/v1/ai/chat`端点响应格式
  - 支持DuckLake专项对话上下文
  - 返回结构化的AI响应和元数据

- [ ] **2.2.2** 创建AI推荐API
  - 新增`/api/v1/ai/recommendations`端点
  - 基于用户行为和数据状态生成推荐
  - 支持不同类型的推荐(性能、安全、优化等)

- [ ] **2.2.3** 创建数据洞察API
  - 新增`/api/v1/ai/insights`端点
  - 提供数据趋势分析和异常检测
  - 返回可视化友好的洞察数据

- [ ] **2.2.4** 更新AIAgent组件
  - 移除硬编码的消息和推荐数据
  - 集成真实的AI API调用
  - 实现会话状态管理

### 🎯 Phase 3: 配置和设置对接 (优先级: 🟢 低)

#### 3.1 系统设置对接
**目标**: 替换Settings页面的配置数据

**任务清单**:
- [ ] **3.1.1** 扩展系统配置API
  - 优化现有`/api/v1/system/config`端点
  - 支持DuckLake专项配置管理
  - 包含快照保留、性能、安全、自动化配置

- [ ] **3.1.2** 更新Settings组件
  - 移除硬编码的配置数据
  - 集成真实的配置API
  - 实现配置验证和保存功能

## 🔧 技术实施细节

### 📡 API响应格式标准化

#### 统一响应格式
```typescript
interface APIResponse<T> {
  success: boolean
  data: T
  message: string
  timestamp?: string
  pagination?: {
    page: number
    limit: number
    total: number
    has_more: boolean
  }
}
```

#### 错误处理格式
```typescript
interface APIError {
  success: false
  error: {
    code: string
    message: string
    details?: any
  }
  timestamp: string
}
```

### 🔄 状态管理更新

#### Redux Store适配
- 更新所有slice的数据结构匹配后端响应
- 添加统一的错误处理逻辑
- 实现数据缓存和刷新机制
- 添加加载状态管理

#### API服务层重构
```typescript
// src/services/api.ts 扩展
export const duckLakeAPI = {
  // 数据库管理
  getDatabases: () => api.get('/api/v1/ducklake/databases'),
  createDatabase: (config: DatabaseConfig) => api.post('/api/v1/ducklake/databases', config),
  updateDatabase: (id: string, config: DatabaseConfig) => api.put(`/api/v1/ducklake/databases/${id}`, config),
  deleteDatabase: (id: string) => api.delete(`/api/v1/ducklake/databases/${id}`),
  
  // 快照管理
  getSnapshots: (params?: SnapshotQuery) => api.get('/api/v1/ducklake/snapshots', { params }),
  createSnapshot: (request: CreateSnapshotRequest) => api.post('/api/v1/ducklake/snapshots', request),
  deleteSnapshot: (id: string) => api.delete(`/api/v1/ducklake/snapshots/${id}`),
  restoreSnapshot: (id: string) => api.post(`/api/v1/ducklake/snapshots/${id}/restore`),
  
  // 版本控制
  getVersions: (params?: VersionQuery) => api.get('/api/v1/ducklake/versions', { params }),
  rollbackVersion: (version: number) => api.post(`/api/v1/ducklake/versions/${version}/rollback`),
  
  // DuckLake指标
  getMetrics: (timeRange?: string) => api.get('/api/v1/ducklake/metrics', { params: { range: timeRange } }),
}
```

### 🧪 测试策略

#### 单元测试
- 为所有新的API调用编写单元测试
- 测试数据格式转换逻辑
- 测试错误处理场景

#### 集成测试
- 端到端的前后端数据流测试
- API响应格式验证
- 用户交互流程测试

#### 性能测试
- API响应时间测试
- 大数据量加载测试
- 并发请求处理测试

## 📊 实施时间表

### 🗓️ 开发计划 (总计: 8-10个工作日)

| 阶段 | 任务 | 预计时间 | 依赖关系 |
|------|------|----------|----------|
| Phase 1.1 | Dashboard数据对接 | 2天 | 后端API完成 |
| Phase 1.2 | 数据探索对接 | 2天 | Phase 1.1 |
| Phase 1.3 | DuckLake管理器对接 | 3天 | 新增后端API |
| Phase 2.1 | 查询分析对接 | 1.5天 | Phase 1完成 |
| Phase 2.2 | AI助手对接 | 1.5天 | Phase 2.1 |
| Phase 3.1 | 系统设置对接 | 1天 | 所有核心功能完成 |

### 🎯 里程碑检查点

#### Milestone 1: 核心数据流 (Day 4)
- ✅ Dashboard完全使用真实数据
- ✅ 数据探索功能正常工作
- ✅ 基础错误处理完成

#### Milestone 2: DuckLake功能 (Day 7)
- ✅ DuckLake管理器完全功能化
- ✅ 所有CRUD操作正常工作
- ✅ 实时数据更新机制

#### Milestone 3: 高级功能 (Day 9)
- ✅ 时间旅行查询功能
- ✅ AI助手智能对话
- ✅ 完整的用户体验

#### Final Milestone: 生产就绪 (Day 10)
- ✅ 所有mock数据移除
- ✅ 完整的测试覆盖
- ✅ 性能优化完成
- ✅ 错误处理健壮

## 🚀 预期成果

### ✅ 技术成果
- **100%真实数据**: 完全消除mock数据依赖
- **企业级API**: 生产就绪的前后端集成
- **类型安全**: 完整的TypeScript类型定义
- **错误处理**: 健壮的错误处理和用户反馈
- **性能优化**: 高效的数据加载和缓存机制

### 📈 业务价值
- **真实体验**: 用户获得完整的产品体验
- **数据一致性**: 前后端数据完全同步
- **功能完整性**: 所有DuckLake功能完全可用
- **生产部署**: 可直接用于生产环境

### 🎯 用户体验提升
- **即时反馈**: 真实的数据加载和状态反馈
- **数据准确性**: 显示真实的系统状态和指标
- **功能可靠性**: 所有操作都有真实的后端支持
- **性能表现**: 优化的数据加载和响应速度

---

## 📋 总结

这个详细的对接计划将确保DuckHub从一个功能演示转变为完全可用的生产级数据湖管理平台。通过系统性地替换所有mock数据，我们将实现：

1. **完整的数据流**: 前后端无缝集成
2. **企业级质量**: 生产就绪的代码质量
3. **用户体验**: 真实、可靠的用户交互
4. **可维护性**: 清晰的架构和代码组织

预计在8-10个工作日内完成所有对接工作，最终交付一个完全功能化的DuckHub数据湖管理平台。

## 🔧 详细技术实施指南

### 📋 缺失后端API实施清单

#### 🗄️ DuckLake数据库管理API
**新增端点**: `/api/v1/ducklake/databases`

```rust
// 后端实现: src/handlers/ducklake_db.rs
#[derive(Debug, Serialize)]
pub struct DuckLakeDatabase {
    pub id: String,
    pub name: String,
    pub path: String,
    pub status: String, // "connected", "disconnected", "error"
    pub size: String,
    pub tables: u32,
    pub last_accessed: String,
    pub connections: u32,
    pub description: Option<String>,
    pub created_at: String,
    pub config: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub config: DatabaseConfig,
}

// API端点实现
pub async fn get_databases() -> ActixResult<HttpResponse> { ... }
pub async fn create_database(request: web::Json<CreateDatabaseRequest>) -> ActixResult<HttpResponse> { ... }
pub async fn update_database(path: web::Path<String>, request: web::Json<UpdateDatabaseRequest>) -> ActixResult<HttpResponse> { ... }
pub async fn delete_database(path: web::Path<String>) -> ActixResult<HttpResponse> { ... }
pub async fn get_database_status(path: web::Path<String>) -> ActixResult<HttpResponse> { ... }
```

#### 📸 快照管理API
**新增端点**: `/api/v1/ducklake/snapshots`

```rust
// 后端实现: src/handlers/ducklake_snapshots.rs
#[derive(Debug, Serialize)]
pub struct DuckLakeSnapshot {
    pub id: String,
    pub version: u32,
    pub timestamp: String,
    pub database: String,
    pub size: String,
    pub tables: u32,
    pub description: String,
    pub tags: Vec<String>,
    pub changes: u32,
    pub author: String,
    pub snapshot_type: String, // "automatic", "manual"
    pub parent_version: Option<u32>,
    pub checksum: String,
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Serialize)]
pub struct SnapshotMetadata {
    pub row_count: u64,
    pub schema_version: u32,
    pub compression_ratio: f64,
    pub file_count: u32,
    pub index_size: u64,
}

// API端点实现
pub async fn get_snapshots(query: web::Query<SnapshotQuery>) -> ActixResult<HttpResponse> { ... }
pub async fn create_snapshot(request: web::Json<CreateSnapshotRequest>) -> ActixResult<HttpResponse> { ... }
pub async fn delete_snapshot(path: web::Path<String>) -> ActixResult<HttpResponse> { ... }
pub async fn restore_snapshot(path: web::Path<String>) -> ActixResult<HttpResponse> { ... }
pub async fn compare_snapshots(request: web::Json<CompareSnapshotsRequest>) -> ActixResult<HttpResponse> { ... }
```

#### 🔄 版本控制API
**新增端点**: `/api/v1/ducklake/versions`

```rust
// 后端实现: src/handlers/ducklake_versions.rs
#[derive(Debug, Serialize)]
pub struct DuckLakeVersion {
    pub id: String,
    pub version: u32,
    pub timestamp: String,
    pub author: String,
    pub operation: String, // "create", "update", "delete", "schema_change"
    pub table: String,
    pub description: String,
    pub changes: VersionChanges,
    pub compatibility: String, // "backward", "forward", "breaking"
    pub rollback_available: bool,
}

#[derive(Debug, Serialize)]
pub struct VersionChanges {
    pub added: u32,
    pub modified: u32,
    pub deleted: u32,
    pub schema_changes: Vec<SchemaChange>,
}

// API端点实现
pub async fn get_versions(query: web::Query<VersionQuery>) -> ActixResult<HttpResponse> { ... }
pub async fn rollback_version(path: web::Path<u32>) -> ActixResult<HttpResponse> { ... }
pub async fn get_version_diff(path: web::Path<(u32, u32)>) -> ActixResult<HttpResponse> { ... }
```

#### 📊 DuckLake指标API
**新增端点**: `/api/v1/ducklake/metrics`

```rust
// 后端实现: src/handlers/ducklake_metrics.rs
#[derive(Debug, Serialize)]
pub struct DuckLakeMetrics {
    pub timestamp: String,
    pub active_databases: u32,
    pub total_snapshots: u32,
    pub time_travel_queries: u64,
    pub schema_evolutions: u32,
    pub query_performance: Vec<QueryPerformancePoint>,
    pub snapshot_activity: Vec<SnapshotActivityPoint>,
    pub storage_usage: Vec<StorageUsagePoint>,
    pub transaction_stats: TransactionStats,
}

// API端点实现
pub async fn get_ducklake_metrics(query: web::Query<MetricsQuery>) -> ActixResult<HttpResponse> { ... }
pub async fn get_performance_history(query: web::Query<TimeRangeQuery>) -> ActixResult<HttpResponse> { ... }
```

### 🔄 前端组件重构指南

#### 📊 Dashboard组件重构
**文件**: `src/pages/Dashboard.tsx`

```typescript
// 移除mock数据，添加真实数据获取
export default function Dashboard() {
  const dispatch = useDispatch<AppDispatch>()
  const { metrics, queryTrends, performanceData, systemHealth } = useSelector(
    (state: RootState) => state.dashboard
  )

  // 新增DuckLake指标状态
  const [duckLakeMetrics, setDuckLakeMetrics] = useState<DuckLakeMetrics | null>(null)
  const [duckLakeLoading, setDuckLakeLoading] = useState(false)

  useEffect(() => {
    // 获取基础仪表板数据
    dispatch(fetchDashboardMetrics())
    dispatch(fetchQueryTrends('24h'))
    dispatch(fetchSystemHealth())

    // 获取DuckLake专项指标
    fetchDuckLakeMetrics()
  }, [dispatch])

  const fetchDuckLakeMetrics = async () => {
    setDuckLakeLoading(true)
    try {
      const response = await duckLakeAPI.getMetrics('24h')
      setDuckLakeMetrics(response.data.data)
    } catch (error) {
      console.error('获取DuckLake指标失败:', error)
    } finally {
      setDuckLakeLoading(false)
    }
  }

  // 移除所有硬编码的mock数据
  // 使用真实的duckLakeMetrics状态
}
```

#### 🗄️ DatabasePanel组件重构
**文件**: `src/components/ducklake/DatabasePanel.tsx`

```typescript
export default function DatabasePanel() {
  const [databases, setDatabases] = useState<DuckLakeDatabase[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // 移除mock数据加载，使用真实API
  useEffect(() => {
    loadDatabases()
  }, [])

  const loadDatabases = async () => {
    setLoading(true)
    setError(null)
    try {
      const response = await duckLakeAPI.getDatabases()
      setDatabases(response.data.data)
    } catch (err) {
      setError('加载数据库列表失败')
      console.error('数据库加载错误:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleCreateDatabase = async (config: CreateDatabaseRequest) => {
    try {
      await duckLakeAPI.createDatabase(config)
      await loadDatabases() // 重新加载列表
      toast.success('数据库创建成功')
    } catch (error) {
      toast.error('数据库创建失败')
    }
  }

  // 移除所有mock数据，使用真实的databases状态
}
```

#### 📸 SnapshotBrowser组件重构
**文件**: `src/components/ducklake/SnapshotBrowser.tsx`

```typescript
export default function SnapshotBrowser() {
  const [snapshots, setSnapshots] = useState<Snapshot[]>([])
  const [loading, setLoading] = useState(true)
  const [viewMode, setViewMode] = useState<'list' | 'timeline'>('list')

  useEffect(() => {
    loadSnapshots()
  }, [])

  const loadSnapshots = async () => {
    setLoading(true)
    try {
      const response = await duckLakeAPI.getSnapshots({
        limit: 50,
        sort: 'timestamp',
        order: 'desc'
      })
      setSnapshots(response.data.data)
    } catch (error) {
      console.error('快照加载失败:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleCreateSnapshot = async (request: CreateSnapshotRequest) => {
    try {
      await duckLakeAPI.createSnapshot(request)
      await loadSnapshots()
      toast.success('快照创建成功')
    } catch (error) {
      toast.error('快照创建失败')
    }
  }

  const handleRestoreSnapshot = async (snapshotId: string) => {
    try {
      await duckLakeAPI.restoreSnapshot(snapshotId)
      toast.success('快照恢复成功')
    } catch (error) {
      toast.error('快照恢复失败')
    }
  }
}
```

### 🔧 Redux Store更新

#### 新增DuckLake Slice
**文件**: `src/store/slices/duckLakeSlice.ts`

```typescript
import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { duckLakeAPI } from '../../services/api'

export interface DuckLakeState {
  databases: DuckLakeDatabase[]
  snapshots: DuckLakeSnapshot[]
  versions: DuckLakeVersion[]
  metrics: DuckLakeMetrics | null
  loading: {
    databases: boolean
    snapshots: boolean
    versions: boolean
    metrics: boolean
  }
  error: {
    databases: string | null
    snapshots: string | null
    versions: string | null
    metrics: string | null
  }
}

// 异步操作
export const fetchDatabases = createAsyncThunk(
  'duckLake/fetchDatabases',
  async () => {
    const response = await duckLakeAPI.getDatabases()
    return response.data.data
  }
)

export const fetchSnapshots = createAsyncThunk(
  'duckLake/fetchSnapshots',
  async (params?: SnapshotQuery) => {
    const response = await duckLakeAPI.getSnapshots(params)
    return response.data.data
  }
)

export const fetchVersions = createAsyncThunk(
  'duckLake/fetchVersions',
  async (params?: VersionQuery) => {
    const response = await duckLakeAPI.getVersions(params)
    return response.data.data
  }
)

export const fetchDuckLakeMetrics = createAsyncThunk(
  'duckLake/fetchMetrics',
  async (timeRange?: string) => {
    const response = await duckLakeAPI.getMetrics(timeRange)
    return response.data.data
  }
)

const duckLakeSlice = createSlice({
  name: 'duckLake',
  initialState,
  reducers: {
    clearError: (state, action) => {
      const { section } = action.payload
      state.error[section] = null
    },
  },
  extraReducers: (builder) => {
    // 数据库相关
    builder
      .addCase(fetchDatabases.pending, (state) => {
        state.loading.databases = true
        state.error.databases = null
      })
      .addCase(fetchDatabases.fulfilled, (state, action) => {
        state.loading.databases = false
        state.databases = action.payload
      })
      .addCase(fetchDatabases.rejected, (state, action) => {
        state.loading.databases = false
        state.error.databases = action.error.message || '获取数据库列表失败'
      })
    // ... 其他异步操作的处理
  },
})

export default duckLakeSlice.reducer
```

#### 更新Store配置
**文件**: `src/store/index.ts`

```typescript
import { configureStore } from '@reduxjs/toolkit'
import querySlice from './slices/querySlice'
import dashboardSlice from './slices/dashboardSlice'
import aiAgentSlice from './slices/aiAgentSlice'
import duckLakeSlice from './slices/duckLakeSlice' // 新增

export const store = configureStore({
  reducer: {
    query: querySlice,
    dashboard: dashboardSlice,
    aiAgent: aiAgentSlice,
    duckLake: duckLakeSlice, // 新增
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST'],
      },
    }),
})
```

### 🧪 测试实施计划

#### 单元测试
**文件**: `src/services/__tests__/duckLakeAPI.test.ts`

```typescript
import { duckLakeAPI } from '../api'
import { server } from '../../mocks/server'
import { rest } from 'msw'

describe('DuckLake API', () => {
  test('获取数据库列表', async () => {
    server.use(
      rest.get('/api/v1/ducklake/databases', (req, res, ctx) => {
        return res(ctx.json({
          success: true,
          data: [
            {
              id: '1',
              name: 'test_db',
              status: 'connected',
              // ... 其他字段
            }
          ]
        }))
      })
    )

    const response = await duckLakeAPI.getDatabases()
    expect(response.data.success).toBe(true)
    expect(response.data.data).toHaveLength(1)
  })

  // ... 其他API测试
})
```

#### 组件集成测试
**文件**: `src/components/__tests__/DatabasePanel.test.tsx`

```typescript
import { render, screen, waitFor } from '@testing-library/react'
import { Provider } from 'react-redux'
import { store } from '../../store'
import DatabasePanel from '../ducklake/DatabasePanel'
import { server } from '../../mocks/server'

describe('DatabasePanel', () => {
  test('加载并显示数据库列表', async () => {
    render(
      <Provider store={store}>
        <DatabasePanel />
      </Provider>
    )

    // 检查加载状态
    expect(screen.getByText('加载中...')).toBeInTheDocument()

    // 等待数据加载完成
    await waitFor(() => {
      expect(screen.getByText('financial_data')).toBeInTheDocument()
    })

    // 检查数据显示
    expect(screen.getByText('2.3 GB')).toBeInTheDocument()
    expect(screen.getByText('15个表')).toBeInTheDocument()
  })
})
```

### 📈 性能优化策略

#### 数据缓存机制
```typescript
// src/utils/cache.ts
class DataCache {
  private cache = new Map<string, { data: any; timestamp: number; ttl: number }>()

  set(key: string, data: any, ttl: number = 300000) { // 5分钟默认TTL
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl
    })
  }

  get(key: string) {
    const item = this.cache.get(key)
    if (!item) return null

    if (Date.now() - item.timestamp > item.ttl) {
      this.cache.delete(key)
      return null
    }

    return item.data
  }

  clear() {
    this.cache.clear()
  }
}

export const dataCache = new DataCache()
```

#### API请求优化
```typescript
// src/services/api.ts 增强
const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器 - 添加缓存检查
api.interceptors.request.use(
  (config) => {
    // 对GET请求检查缓存
    if (config.method === 'get') {
      const cacheKey = `${config.url}?${JSON.stringify(config.params)}`
      const cachedData = dataCache.get(cacheKey)
      if (cachedData) {
        // 返回缓存数据
        return Promise.resolve({ data: cachedData, fromCache: true })
      }
    }

    const token = localStorage.getItem('auth_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => Promise.reject(error)
)

// 响应拦截器 - 添加缓存存储
api.interceptors.response.use(
  (response) => {
    // 对成功的GET请求缓存数据
    if (response.config.method === 'get' && response.data.success) {
      const cacheKey = `${response.config.url}?${JSON.stringify(response.config.params)}`
      dataCache.set(cacheKey, response.data)
    }
    return response
  },
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('auth_token')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)
```

### 🔍 错误处理增强

#### 统一错误处理Hook
```typescript
// src/hooks/useErrorHandler.ts
import { useCallback } from 'react'
import { toast } from 'sonner'

export const useErrorHandler = () => {
  const handleError = useCallback((error: any, context?: string) => {
    console.error(`错误 ${context ? `[${context}]` : ''}:`, error)

    let message = '操作失败，请稍后重试'

    if (error.response?.data?.error?.message) {
      message = error.response.data.error.message
    } else if (error.message) {
      message = error.message
    }

    toast.error(message)
  }, [])

  return { handleError }
}
```

#### 组件错误边界
```typescript
// src/components/ErrorBoundary.tsx
import React from 'react'
import { AlertCircle } from 'lucide-react'
import { Button } from './ui/button'

interface Props {
  children: React.ReactNode
  fallback?: React.ReactNode
}

interface State {
  hasError: boolean
  error?: Error
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('组件错误:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="flex flex-col items-center justify-center p-8 text-center">
          <AlertCircle className="h-12 w-12 text-red-500 mb-4" />
          <h2 className="text-xl font-semibold mb-2">出现了一些问题</h2>
          <p className="text-gray-600 mb-4">
            {this.state.error?.message || '组件加载失败'}
          </p>
          <Button onClick={() => window.location.reload()}>
            刷新页面
          </Button>
        </div>
      )
    }

    return this.props.children
  }
}
```

## 🎯 质量保证检查清单

### ✅ 代码质量检查
- [ ] 所有TypeScript类型定义完整
- [ ] 所有组件都有适当的错误处理
- [ ] API调用都有加载状态管理
- [ ] 所有用户操作都有反馈提示
- [ ] 代码符合项目的ESLint规则
- [ ] 所有新功能都有单元测试覆盖

### ✅ 功能完整性检查
- [ ] 所有mock数据都已移除
- [ ] 所有API端点都正常工作
- [ ] 数据格式前后端完全匹配
- [ ] 所有CRUD操作都能正常执行
- [ ] 错误场景都有适当处理
- [ ] 用户权限控制正常工作

### ✅ 性能检查
- [ ] 页面加载时间 < 2秒
- [ ] API响应时间 < 500ms
- [ ] 大数据量加载有分页处理
- [ ] 图表渲染流畅无卡顿
- [ ] 内存使用合理无泄漏
- [ ] 网络请求有适当缓存

### ✅ 用户体验检查
- [ ] 所有操作都有加载指示
- [ ] 错误信息清晰易懂
- [ ] 成功操作有确认反馈
- [ ] 界面响应及时流畅
- [ ] 数据更新实时反映
- [ ] 操作流程符合直觉

---

通过这个详细的技术实施指南，开发团队可以系统性地完成前后端数据对接工作，确保最终交付一个完全功能化、生产就绪的DuckHub数据湖管理平台。
