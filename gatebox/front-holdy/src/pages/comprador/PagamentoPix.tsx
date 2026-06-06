import { useParams, useLocation, useNavigate } from 'react-router-dom'
import { useOrderPolling } from '../../hooks/useOrderPolling'
import { Layout, Card } from '../../components/Layout'
import { PixQrCode } from '../../components/PixQrCode'
import { useEffect } from 'react'

export function PagamentoPix() {
  const { id } = useParams<{ id: string }>()
  const location = useLocation()
  const nav = useNavigate()
  const state = location.state as { pix_br_code?: string; amount?: string } | null

  const { data: order } = useOrderPolling(id ?? null, 'in_custody')

  // Quando pagamento for confirmado, avança automaticamente
  useEffect(() => {
    if (order?.status === 'in_custody') {
      nav(`/comprador/pedido/${id}`, { replace: true })
    }
  }, [order?.status, id, nav])

  const brCode = state?.pix_br_code ?? order?.pix_br_code
  const amount = state?.amount ?? order?.amount ?? ''

  return (
    <Layout title="Pagar com PIX">
      <Card className="mb-4">
        {brCode ? (
          <PixQrCode brCode={brCode} amount={amount} />
        ) : (
          <p className="text-center text-gray-400 py-8">Código PIX não disponível</p>
        )}
      </Card>

      <Card className="bg-yellow-50 border-yellow-200">
        <div className="flex items-start gap-3">
          <span className="text-xl">⏳</span>
          <div>
            <p className="font-medium text-yellow-800">Aguardando confirmação</p>
            <p className="text-sm text-yellow-700 mt-0.5">
              Após o pagamento, esta página avança automaticamente. Pode levar alguns segundos.
            </p>
          </div>
        </div>
      </Card>

      <div className="mt-4 text-center">
        <p className="text-xs text-gray-400">Pedido: <span className="font-mono">{id}</span></p>
      </div>
    </Layout>
  )
}
