import { useEffect } from 'react'
import { Routes, Route } from 'react-router-dom'
import { Toaster } from 'sonner'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import QueryAnalytics from './pages/QueryAnalytics'
import DataExplorer from './pages/DataExplorer'
import AIAgent from './pages/AIAgent'
import Settings from './pages/Settings'
import DuckLakeManager from './pages/DuckLakeManager'
import { initializeAuth } from './utils/auth'

function App() {
  useEffect(() => {
    // 初始化认证状态
    initializeAuth().catch(console.error)
  }, [])

  return (
    <div className="min-h-screen bg-background">
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/query-analytics" element={<QueryAnalytics />} />
          <Route path="/data-explorer" element={<DataExplorer />} />
          <Route path="/ai-agent" element={<AIAgent />} />
          <Route path="/ducklake-manager" element={<DuckLakeManager />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Layout>
      <Toaster />
    </div>
  )
}

export default App
