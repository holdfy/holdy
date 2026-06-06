import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { ordersApi, sellerApi } from '../../lib/api'
import { Layout, Card, Btn } from '../../components/Layout'
import { OrderStatusBadge } from '../../components/OrderStatusBadge'

export function DashboardVendedor() {
  const { data: dash } = useQuery({
    queryKey: ['seller-dashboard'],
    queryFn: () => sellerApi.dashboard(),
  })
  const { data: ordersData } = useQuery({
    queryKey: ['orders', 'seller'],
    queryFn: () => ordersApi.list('seller'),
  })

  return (
    <Layout title="Painel do Vendedor">
      {/* Stats */}
      <div className="grid grid-cols-2 gap-3 mb-6">
        <Card>
          <p className="text-xs text-gray-500">Pedidos ativos</p>
          <p className="text-2xl font-bold text-brand">{dash?.in_custody_orders ?? '—'}</p>
        </Card>
        <Card>
          <p className="text-xs text-gray-500">Pedidos concluídos</p>
          <p className="text-2xl font-bold text-gray-800">{dash?.completed_orders ?? '—'}</p>
        </Card>
        <Card>
          <p className="text-xs text-gray-500">Volume total</p>
          <p className="text-lg font-bold text-gray-800">R$ {dash?.total_volume_brl ?? '—'}</p>
        </Card>
        <Card>
          <p className="text-xs text-gray-500">Liberado</p>
          <p className="text-lg font-bold text-green-700">R$ {dash?.completed_volume_brl ?? '—'}</p>
        </Card>
      </div>

      <Btn onClick={() => {}} className="mb-6">
        <Link to="/vendedor/nova-proposta" className="block w-full">+ Nova proposta</Link>
      </Btn>

      {/* Pedidos */}
      <h2 className="text-base font-semibold text-gray-800 mb-3">Meus pedidos</h2>
      <div className="space-y-3">
        {ordersData?.orders.length === 0 && (
          <Card><p className="text-sm text-gray-400 text-center py-4">Nenhum pedido ainda</p></Card>
        )}
        {ordersData?.orders.map(o => (
          <Link key={o.id} to={`/vendedor/pedido/${o.id}`}>
            <Card className="hover:border-brand transition-colors cursor-pointer">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium text-gray-900">R$ {o.amount}</p>
                  <p className="text-xs text-gray-400 mt-0.5">{o.description ?? 'Sem descrição'}</p>
                </div>
                <OrderStatusBadge status={o.status} />
              </div>
            </Card>
          </Link>
        ))}
      </div>
    </Layout>
  )
}
