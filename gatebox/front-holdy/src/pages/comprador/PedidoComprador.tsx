import { useState } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../../context/AuthContext'
import { custodyApi, disputeApi } from '../../lib/api'
import { useOrderPolling } from '../../hooks/useOrderPolling'
import { Layout, Card, Btn, ErrorMsg } from '../../components/Layout'
import { OrderStatusBadge } from '../../components/OrderStatusBadge'

const DISPUTE_REASONS = [
  { value: 'non_delivery',    label: '1 — Não recebi o produto' },
  { value: 'wrong_product',   label: '2 — Produto diferente do anúncio' },
  { value: 'broken_product',  label: '3 — Produto chegou quebrado' },
  { value: 'empty_box',       label: '4 — Caixa chegou vazia' },
  { value: 'other',           label: '5 — Outro motivo' },
]

export function PedidoComprador() {
  const { id } = useParams<{ id: string }>()
  const { userId } = useAuth()
  const nav = useNavigate()
  const { data: order, isLoading } = useOrderPolling(id ?? null)

  const [confirmStep, setConfirmStep] = useState<'idle' | 'confirm'>('idle')
  const [releaseLoading, setReleaseLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const [showDisputeMenu, setShowDisputeMenu] = useState(false)
  const [disputeLoading, setDisputeLoading] = useState(false)

  async function releaseEscrow() {
    if (!id || !userId) return
    setReleaseLoading(true); setError(null)
    try {
      await custodyApi.release({
        order_id: id,
        released_by: userId,
        idempotency_key: `confirm_receipt:${id}:${userId}`,
      })
      window.location.reload()
    } catch (err: any) {
      setError(err.message ?? 'Erro ao confirmar recebimento')
    } finally {
      setReleaseLoading(false)
    }
  }

  async function openDispute(reason: string) {
    if (!id) return
    setDisputeLoading(true); setError(null)
    try {
      await disputeApi.open(id, reason)
      nav(`/comprador/pedido/${id}/disputa`)
    } catch (err: any) {
      setError(err.message ?? 'Erro ao abrir disputa')
    } finally {
      setDisputeLoading(false)
    }
  }

  if (isLoading) return <Layout><p className="text-center text-gray-400 py-12">Carregando…</p></Layout>
  if (!order) return <Layout><p className="text-center text-gray-400 py-12">Pedido não encontrado</p></Layout>

  return (
    <Layout title="Meu pedido">
      <Card className="space-y-3 mb-4">
        <div className="flex justify-between items-start">
          <div>
            <p className="text-2xl font-bold">R$ {order.amount}</p>
            <p className="text-sm text-gray-500 mt-0.5">{order.description ?? 'Sem descrição'}</p>
          </div>
          <OrderStatusBadge status={order.status} />
        </div>
        <p className="text-xs text-gray-400 font-mono">{order.id}</p>
        {order.tracking_code && (
          <div className="bg-blue-50 rounded-lg p-3">
            <p className="text-xs text-blue-600 font-medium">Código de rastreio</p>
            <p className="font-mono font-bold text-blue-900 mt-0.5">{order.tracking_code}</p>
          </div>
        )}
      </Card>

      {/* Aguardando pagamento */}
      {order.status === 'pending_funding' && (
        <Card className="text-center space-y-3">
          <div className="text-4xl">💳</div>
          <p className="font-medium">Aguardando pagamento</p>
          <Btn onClick={() => nav(`/comprador/pedido/${id}/pix`)}>Ver QR PIX</Btn>
        </Card>
      )}

      {/* Em custódia — aguardando envio */}
      {order.status === 'in_custody' && !order.tracking_code && (
        <Card className="text-center py-5 space-y-2 bg-blue-50 border-blue-200">
          <div className="text-4xl">📦</div>
          <p className="font-medium text-blue-800">Pagamento confirmado!</p>
          <p className="text-sm text-blue-600">Aguardando o vendedor enviar o produto e registrar o rastreio.</p>
        </Card>
      )}

      {/* Em custódia com rastreio — pode confirmar */}
      {order.status === 'in_custody' && (
        <div className="space-y-3 mt-4">
          {confirmStep === 'idle' ? (
            <>
              <Btn onClick={() => setConfirmStep('confirm')}>
                ✓ Recebi o produto — liberar pagamento
              </Btn>
              <button
                onClick={() => setShowDisputeMenu(true)}
                className="w-full text-sm text-red-500 py-2 hover:underline"
              >
                Há um problema — abrir disputa
              </button>
            </>
          ) : (
            <Card className="space-y-4 border-yellow-300 bg-yellow-50">
              <p className="font-semibold text-yellow-800">⚠️ Confirmar recebimento?</p>
              <p className="text-sm text-yellow-700">
                Ao confirmar, o valor de <strong>R$ {order.amount}</strong> será liberado ao vendedor. Esta ação não pode ser desfeita.
              </p>
              <ErrorMsg msg={error} />
              <Btn onClick={releaseEscrow} disabled={releaseLoading}>
                {releaseLoading ? 'Liberando…' : 'Sim, confirmar recebimento'}
              </Btn>
              <Btn variant="ghost" onClick={() => setConfirmStep('idle')}>Cancelar</Btn>
            </Card>
          )}

          {showDisputeMenu && (
            <Card className="space-y-3">
              <p className="font-semibold text-red-700">Qual é o problema?</p>
              {DISPUTE_REASONS.map(r => (
                <button
                  key={r.value}
                  onClick={() => openDispute(r.value)}
                  disabled={disputeLoading}
                  className="w-full text-left px-4 py-3 border border-gray-200 rounded-xl text-sm hover:border-red-400 hover:bg-red-50 transition-colors disabled:opacity-50"
                >
                  {r.label}
                </button>
              ))}
              <ErrorMsg msg={error} />
              <Btn variant="ghost" onClick={() => setShowDisputeMenu(false)}>Cancelar</Btn>
            </Card>
          )}
        </div>
      )}

      {/* Concluído */}
      {order.status === 'completed' && (
        <Card className="text-center py-6 bg-green-50 border-green-200 space-y-2">
          <div className="text-4xl">🎉</div>
          <p className="font-bold text-green-800">Pedido concluído!</p>
          <p className="text-sm text-green-700">O pagamento foi liberado ao vendedor.</p>
        </Card>
      )}

      <div className="mt-6">
        <Link to="/comprador" className="text-sm text-gray-400 hover:text-brand">← Meus pedidos</Link>
      </div>
    </Layout>
  )
}
