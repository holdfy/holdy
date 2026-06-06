import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { proposalApi } from '../../lib/api'
import { useAuth } from '../../context/AuthContext'
import { Layout, Card, Btn, Input, ErrorMsg } from '../../components/Layout'

export function PropostaPage() {
  const { id } = useParams<{ id: string }>()
  const { isAuthenticated } = useAuth()
  const nav = useNavigate()

  const { data: proposal, isLoading } = useQuery({
    queryKey: ['proposal', id],
    queryFn: () => proposalApi.get(id!),
    enabled: !!id,
  })

  const [cpf, setCpf] = useState('')
  const [acceptLoading, setAcceptLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [step, setStep] = useState<'view' | 'doc'>('view')

  async function accept() {
    if (!id) return
    const doc = cpf.replace(/\D/g, '')
    if (doc.length !== 11 && doc.length !== 14) {
      setError('CPF deve ter 11 dígitos ou CNPJ 14 dígitos')
      return
    }
    setAcceptLoading(true); setError(null)
    try {
      const resp = await proposalApi.accept(id, { cpf: doc })
      nav(`/comprador/pedido/${resp.order_id}/pix`, { state: { pix_br_code: resp.pix_br_code, amount: resp.amount } })
    } catch (err: any) {
      setError(err.message ?? 'Erro ao aceitar proposta')
    } finally {
      setAcceptLoading(false)
    }
  }

  function goAccept() {
    if (!isAuthenticated) {
      nav(`/login?next=/proposta/${id}`)
      return
    }
    setStep('doc')
  }

  if (isLoading) return <Layout><p className="text-center text-gray-400 py-12">Carregando proposta…</p></Layout>
  if (!proposal) return <Layout><p className="text-center text-gray-400 py-12">Proposta não encontrada</p></Layout>

  const expired = proposal.status === 'expired' || new Date(proposal.expires_at) < new Date()

  return (
    <Layout title="Proposta de compra">
      {proposal.listing_photos?.[0] && (
        <img
          src={proposal.listing_photos[0]}
          alt="Produto"
          className="w-full h-48 object-cover rounded-2xl mb-4"
        />
      )}

      <Card className="space-y-4 mb-4">
        {proposal.listing_title && (
          <p className="font-semibold text-gray-900 text-lg">{proposal.listing_title}</p>
        )}
        {proposal.description && !proposal.listing_title && (
          <p className="text-gray-700">{proposal.description}</p>
        )}

        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs text-gray-400">Valor protegido</p>
            <p className="text-3xl font-bold text-brand">R$ {proposal.amount}</p>
          </div>
          {expired ? (
            <span className="bg-red-100 text-red-700 text-xs px-3 py-1 rounded-full font-medium">Expirada</span>
          ) : (
            <span className="bg-green-100 text-green-700 text-xs px-3 py-1 rounded-full font-medium">Válida</span>
          )}
        </div>

        {proposal.listing_url && (
          <a href={proposal.listing_url} target="_blank" rel="noreferrer" className="text-xs text-brand underline">
            Ver anúncio original ��
          </a>
        )}

        <div className="bg-brand-light rounded-xl p-3 text-sm text-brand">
          🔒 O dinheiro fica em custódia até você confirmar que recebeu o produto.
        </div>
      </Card>

      {expired ? (
        <Card>
          <p className="text-center text-gray-500 text-sm">Esta proposta expirou. Peça ao vendedor uma nova.</p>
        </Card>
      ) : step === 'view' ? (
        <div className="space-y-3">
          <Btn onClick={goAccept}>Aceitar e pagar com PIX</Btn>
          <Btn variant="ghost" onClick={() => proposalApi.reject(id!).then(() => nav('/'))}>
            Recusar proposta
          </Btn>
        </div>
      ) : (
        <Card className="space-y-4">
          <p className="text-sm font-medium text-gray-700">Informe seu CPF ou CNPJ para gerar o PIX:</p>
          <Input
            label="CPF (11 dígitos) ou CNPJ (14 dígitos)"
            value={cpf}
            onChange={e => setCpf(e.target.value.replace(/\D/g, ''))}
            placeholder="000.000.000-00"
            maxLength={14}
            autoFocus
          />
          <ErrorMsg msg={error} />
          <Btn onClick={accept} disabled={acceptLoading}>
            {acceptLoading ? 'Gerando PIX…' : 'Gerar QR PIX →'}
          </Btn>
          <Btn variant="ghost" onClick={() => setStep('view')}>← Voltar</Btn>
        </Card>
      )}
    </Layout>
  )
}
