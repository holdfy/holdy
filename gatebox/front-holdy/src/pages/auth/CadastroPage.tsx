import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../../context/AuthContext'
import { Btn, Card, ErrorMsg, Input } from '../../components/Layout'

export function CadastroPage() {
  const { register } = useAuth()
  const nav = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [phone, setPhone] = useState('')
  const [role, setRole] = useState<'seller' | 'buyer'>('seller')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    if (password.length < 8) { setError('Senha deve ter pelo menos 8 caracteres'); return }
    setLoading(true)
    try {
      await register({ email, password, phone: phone || undefined, role })
      nav('/')
    } catch (err: any) {
      setError(err.message ?? 'Erro ao criar conta')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <div className="w-full max-w-sm">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-brand">HoldFy</h1>
          <p className="text-gray-500 mt-1">Crie sua conta</p>
        </div>
        <Card>
          <form onSubmit={submit} className="space-y-4">
            <div className="grid grid-cols-2 gap-2">
              {(['seller', 'buyer'] as const).map(r => (
                <button
                  key={r}
                  type="button"
                  onClick={() => setRole(r)}
                  className={`py-2 rounded-xl text-sm font-medium border transition-colors ${
                    role === r ? 'bg-brand text-white border-brand' : 'border-gray-200 text-gray-600'
                  }`}
                >
                  {r === 'seller' ? '🛍 Vendedor' : '🛒 Comprador'}
                </button>
              ))}
            </div>
            <Input label="E-mail" type="email" value={email} onChange={e => setEmail(e.target.value)} required autoFocus />
            <Input label="Senha (mín. 8 caracteres)" type="password" value={password} onChange={e => setPassword(e.target.value)} required />
            <Input label="Celular (opcional)" type="tel" value={phone} onChange={e => setPhone(e.target.value)} placeholder="(41) 99999-9999" />
            <ErrorMsg msg={error} />
            <Btn type="submit" disabled={loading}>{loading ? 'Criando conta…' : 'Criar conta'}</Btn>
          </form>
        </Card>
        <p className="text-center text-sm text-gray-500 mt-4">
          Já tem conta?{' '}
          <Link to="/login" className="text-brand font-medium hover:underline">Entrar</Link>
        </p>
      </div>
    </div>
  )
}
