import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useOrderPolling } from '../../hooks/useOrderPolling'
import { ordersApi, disputeApi } from '../../lib/api'
import { Layout, Card, Btn, Input, ErrorMsg } from '../../components/Layout'
import { OrderStatusBadge } from '../../components/OrderStatusBadge'
import { EvidenceUploader } from '../../components/EvidenceUploader'

export function PedidoVendedor() {
  const { id } = useParams<{ id: string }>()
  const { data: order, isLoading } = useOrderPolling(id ?? null)
  const [tracking, setTracking] = useState('')
  const [trackingDone, setTrackingDone] = useState(false)
  const [trackingLoading, setTrackingLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showDispute, setShowDispute] = useState(false)
  const [evidenceCount, setEvidenceCount] = useState(0)
  const [analyzeLoading, setAnalyzeLoading] = useState(false)

  async function registerTracking() {
    if (!tracking.trim() || !id) return
    setTrackingLoading(true); setError(null)
    try {
      await ordersApi.setTracking(id, tracking.trim().toUpperCase())
      setTrackingDone(true)
    } catch (err: any) {
      setError(err.message ?? 'Erro ao registrar rastreio')
    } finally {
      setTrackingLoading(false)
    }
  }

  async function sendAnalyze() {
    if (!id) return
    setAnalyzeLoading(true)
    try {
      await disputeApi.analyze(id)
    } finally {
      setAnalyzeLoading(false)
    }
  }

  if (isLoading) return <Layout><p className="text-center text-gray-400 py-12">Carregando…</p></Layout>
  if (!order) return <Layout><p className="text-center text-gray-400 py-12">Pedido não encontrado</p></Layout>

  return (
    <Layout title="Pedido">
      <Card className="space-y-4 mb-4">
        <div className="flex justify-between items-start">
          <div>
            <p className="text-2xl font-bold text-gray-900">R$ {order.amount}</p>
            <p className="text-sm text-gray-500 mt-0.5">{order.description ?? 'Sem descrição'}</p>
          </div>
          <OrderStatusBadge status={order.status} />
        </div>
        <p className="text-xs text-gray-400 font-mono">{order.id}</p>
      </Card>

      {/* Status: aguardando pagamento */}
      {order.status === 'pending_funding' && (
        <Card className="text-center py-6 space-y-2">
          <div className="text-4xl animate-pulse">⏳</div>
          <p className="font-medium text-gray-700">Aguardando pagamento do comprador</p>
          <p className="text-sm text-gray-400">Você será notificado assim que o PIX for confirmado</p>
        </Card>
      )}

      {/* Status: pago — informar rastreio */}
      {order.status === 'in_custody' && (
        <div className="space-y-4">
          <Card className="bg-green-50 border-green-200 space-y-2">
            <p className="font-semibold text-green-800">✅ Pagamento confirmado!</p>
            <p className="text-sm text-green-700">O dinheiro está em custódia segura. Envie o produto e registre o código de rastreio.</p>
          </Card>

          {order.tracking_code ? (
            <Card>
              <p className="text-sm text-gray-500">Código de rastreio registrado:</p>
              <p className="font-mono font-bold text-gray-900 mt-1">{order.tracking_code}</p>
            </Card>
          ) : trackingDone ? (
            <Card><p className="text-green-700 font-medium">✓ Rastreio registrado com sucesso!</p></Card>
          ) : (
            <Card className="space-y-3">
              <p className="text-sm font-medium text-gray-700">Informe o código de rastreio após postar o produto:</p>
              <Input
                value={tracking}
                onChange={e => setTracking(e.target.value)}
                placeholder="AA123456789BR"
              />
              <ErrorMsg msg={error} />
              <Btn onClick={registerTracking} disabled={trackingLoading || !tracking.trim()}>
                {trackingLoading ? 'Registrando…' : 'Registrar rastreio'}
              </Btn>
            </Card>
          )}
        </div>
      )}

      {/* Status: concluído */}
      {order.status === 'completed' && (
        <Card className="text-center py-6 space-y-2 bg-green-50 border-green-200">
          <div className="text-4xl">💚</div>
          <p className="font-bold text-green-800">Pedido concluído!</p>
          <p className="text-sm text-green-700">O pagamento foi liberado para sua chave PIX.</p>
        </Card>
      )}

      {/* Disputa ativa */}
      {order.status === 'in_custody' && (
        <div className="mt-4">
          {!showDispute ? (
            <button
              onClick={() => setShowDispute(true)}
              className="text-sm text-red-500 underline"
            >
              Há uma disputa aberta — enviar defesa
            </button>
          ) : (
            <Card className="space-y-4">
              <p className="font-semibold text-red-700">📋 Contestar disputa</p>
              <p className="text-sm text-gray-500">Envie fotos, vídeos ou documentos que comprovem a entrega.</p>
              <EvidenceUploader orderId={id!} onUploaded={setEvidenceCount} />
              {evidenceCount > 0 && (
                <Btn onClick={sendAnalyze} disabled={analyzeLoading} variant="secondary">
                  {analyzeLoading ? 'Enviando…' : 'Concluir defesa e acionar análise'}
                </Btn>
              )}
            </Card>
          )}
        </div>
      )}

      <div className="mt-4">
        <Link to="/vendedor" className="text-sm text-gray-400 hover:text-brand">← Voltar ao painel</Link>
      </div>
    </Layout>
  )
}
