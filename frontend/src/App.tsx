import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Dashboard } from './pages/Dashboard'
import { SiteDetail } from './pages/SiteDetail'

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/sites/:id" element={<SiteDetail />} />
      </Routes>
    </BrowserRouter>
  )
}

export default App