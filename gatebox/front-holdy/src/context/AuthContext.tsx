import { createContext, useContext, useState, type ReactNode } from 'react'
import { authApi, type AuthResp, type LoginReq, type RegisterReq } from '../lib/api'

interface AuthState {
  token: string | null
  userId: string | null
  role: string | null
}

interface AuthCtx extends AuthState {
  login: (data: LoginReq) => Promise<void>
  register: (data: RegisterReq) => Promise<void>
  logout: () => void
  isAuthenticated: boolean
}

const Ctx = createContext<AuthCtx | null>(null)

const STORAGE_KEY = 'hf_token'
const USER_KEY = 'hf_user'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>(() => {
    const token = localStorage.getItem(STORAGE_KEY)
    const raw = localStorage.getItem(USER_KEY)
    const user = raw ? JSON.parse(raw) : {}
    return { token, userId: user.userId ?? null, role: user.role ?? null }
  })

  function persist(resp: AuthResp) {
    localStorage.setItem(STORAGE_KEY, resp.access_token)
    localStorage.setItem(USER_KEY, JSON.stringify({ userId: resp.user_id, role: resp.role }))
    setState({ token: resp.access_token, userId: resp.user_id, role: resp.role })
  }

  async function login(data: LoginReq) {
    const resp = await authApi.login(data)
    persist(resp)
  }

  async function register(data: RegisterReq) {
    const resp = await authApi.register(data)
    persist(resp)
  }

  function logout() {
    localStorage.removeItem(STORAGE_KEY)
    localStorage.removeItem(USER_KEY)
    setState({ token: null, userId: null, role: null })
  }

  return (
    <Ctx.Provider value={{ ...state, login, register, logout, isAuthenticated: !!state.token }}>
      {children}
    </Ctx.Provider>
  )
}

export function useAuth(): AuthCtx {
  const ctx = useContext(Ctx)
  if (!ctx) throw new Error('useAuth must be inside AuthProvider')
  return ctx
}
