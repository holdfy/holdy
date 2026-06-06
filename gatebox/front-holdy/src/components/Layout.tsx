import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import type { ReactNode } from 'react'

export function Layout({ children, title }: { children: ReactNode; title?: string }) {
  const { isAuthenticated, logout, role } = useAuth()
  const nav = useNavigate()

  function handleLogout() {
    logout()
    nav('/login')
  }

  return (
    <div className="min-h-screen flex flex-col">
      <header className="bg-white border-b border-gray-200 sticky top-0 z-10">
        <div className="max-w-2xl mx-auto px-4 h-14 flex items-center justify-between">
          <Link to="/" className="font-bold text-brand text-lg">HoldFy</Link>
          {isAuthenticated && (
            <div className="flex items-center gap-4">
              {role === 'seller' || role === 'platform' ? (
                <Link to="/vendedor" className="text-sm text-gray-600 hover:text-brand">Vendedor</Link>
              ) : null}
              {role === 'buyer' || role === 'platform' ? (
                <Link to="/comprador" className="text-sm text-gray-600 hover:text-brand">Comprador</Link>
              ) : null}
              <button onClick={handleLogout} className="text-sm text-gray-400 hover:text-red-500">Sair</button>
            </div>
          )}
        </div>
      </header>

      <main className="flex-1 max-w-2xl mx-auto w-full px-4 py-6">
        {title && <h1 className="text-xl font-bold text-gray-900 mb-6">{title}</h1>}
        {children}
      </main>
    </div>
  )
}

export function Card({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <div className={`bg-white rounded-2xl shadow-sm border border-gray-100 p-5 ${className}`}>
      {children}
    </div>
  )
}

export function Btn({
  children,
  onClick,
  disabled,
  variant = 'primary',
  type = 'button',
  className = '',
}: {
  children: ReactNode
  onClick?: () => void
  disabled?: boolean
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  type?: 'button' | 'submit'
  className?: string
}) {
  const base = 'w-full py-3 rounded-xl font-semibold transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
  const variants = {
    primary:   'bg-brand text-white hover:bg-brand-dark',
    secondary: 'bg-brand-light text-brand hover:bg-violet-100',
    danger:    'bg-red-600 text-white hover:bg-red-700',
    ghost:     'bg-gray-100 text-gray-700 hover:bg-gray-200',
  }
  return (
    <button type={type} onClick={onClick} disabled={disabled} className={`${base} ${variants[variant]} ${className}`}>
      {children}
    </button>
  )
}

export function Input({
  label,
  ...props
}: React.InputHTMLAttributes<HTMLInputElement> & { label?: string }) {
  return (
    <div className="space-y-1">
      {label && <label className="block text-sm font-medium text-gray-700">{label}</label>}
      <input
        {...props}
        className="w-full border border-gray-300 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-brand focus:border-transparent"
      />
    </div>
  )
}

export function ErrorMsg({ msg }: { msg: string | null }) {
  if (!msg) return null
  return <p className="text-sm text-red-600 bg-red-50 rounded-lg p-3">{msg}</p>
}
