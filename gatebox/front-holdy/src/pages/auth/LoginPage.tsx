import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../../context/AuthContext'
import { Btn, Card, ErrorMsg, Input } from '../../components/Layout'

export function LoginPage() {
  const { login } = useAuth()
  const nav = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      await login({ email, password })
      nav('/')
    } catch (err: any) {
      setError(err.message ?? 'Email ou senha inválidos')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <div className="w-full max-w-sm">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-brand">HoldFy</h1>
          <p className="text-gray-500 mt-1">Compra e venda com segurança</p>
        </div>
        <Card>
          <form onSubmit={submit} className="space-y-4">
            <Input label="E-mail" type="email" value={email} onChange={e => setEmail(e.target.value)} required autoFocus />
            <Input label="Senha" type="password" value={password} onChange={e => setPassword(e.target.value)} required />
            <ErrorMsg msg={error} />
            <Btn type="submit" disabled={loading}>{loading ? 'Entrando…' : 'Entrar'}</Btn>
          </form>
        </Card>
        <p className="text-center text-sm text-gray-500 mt-4">
          Não tem conta?{' '}
          <Link to="/cadastro" className="text-brand font-medium hover:underline">Cadastre-se</Link>
        </p>
      </div>
    </div>
  )
}
