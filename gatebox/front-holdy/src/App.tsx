import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AuthProvider, useAuth } from './context/AuthContext'

import { LoginPage }          from './pages/auth/LoginPage'
import { CadastroPage }       from './pages/auth/CadastroPage'
import { DashboardVendedor }  from './pages/vendedor/DashboardVendedor'
import { NovaProposta }       from './pages/vendedor/NovaProposta'
import { PedidoVendedor }     from './pages/vendedor/PedidoVendedor'
import { DashboardComprador } from './pages/comprador/DashboardComprador'
import { PropostaPage }       from './pages/comprador/PropostaPage'
import { PagamentoPix }       from './pages/comprador/PagamentoPix'
import { PedidoComprador }    from './pages/comprador/PedidoComprador'
import { DisputaPage }        from './pages/disputa/DisputaPage'

const qc = new QueryClient()

function RequireAuth({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth()
  return isAuthenticated ? <>{children}</> : <Navigate to="/login" replace />
}

function Home() {
  const { role } = useAuth()
  if (role === 'buyer') return <Navigate to="/comprador" replace />
  return <Navigate to="/vendedor" replace />
}

export default function App() {
  return (
    <QueryClientProvider client={qc}>
      <AuthProvider>
        <BrowserRouter>
          <Routes>
            {/* Public */}
            <Route path="/login"        element={<LoginPage />} />
            <Route path="/cadastro"     element={<CadastroPage />} />
            <Route path="/proposta/:id" element={<PropostaPage />} />

            {/* Home redirect */}
            <Route path="/" element={<RequireAuth><Home /></RequireAuth>} />

            {/* Vendedor */}
            <Route path="/vendedor"                  element={<RequireAuth><DashboardVendedor /></RequireAuth>} />
            <Route path="/vendedor/nova-proposta"    element={<RequireAuth><NovaProposta /></RequireAuth>} />
            <Route path="/vendedor/pedido/:id"       element={<RequireAuth><PedidoVendedor /></RequireAuth>} />

            {/* Comprador */}
            <Route path="/comprador"                        element={<RequireAuth><DashboardComprador /></RequireAuth>} />
            <Route path="/comprador/pedido/:id/pix"         element={<RequireAuth><PagamentoPix /></RequireAuth>} />
            <Route path="/comprador/pedido/:id/disputa"     element={<RequireAuth><DisputaPage /></RequireAuth>} />
            <Route path="/comprador/pedido/:id"             element={<RequireAuth><PedidoComprador /></RequireAuth>} />

            {/* Catch-all */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  )
}
