import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { ordersApi } from '../../lib/api'
import { Layout, Card } from '../../components/Layout'
import { OrderStatusBadge } from '../../components/OrderStatusBadge'

export function DashboardComprador() {
  const { data } = useQuery({
    queryKey: ['orders', 'buyer'],
    queryFn: () => ordersApi.list('buyer'),
  })

  return (
    <Layout title="Minhas compras">
      <div className="space-y-3">
        {data?.orders.length === 0 && (
          <Card><p className="text-sm text-gray-400 text-center py-4">Nenhuma compra ainda</p></Card>
        )}
        {data?.orders.map(o => (
          <Link key={o.id} to={`/comprador/pedido/${o.id}`}>
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
