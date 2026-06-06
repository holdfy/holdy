import { useQuery } from '@tanstack/react-query'
import { ordersApi, type OrderResp } from '../lib/api'

const TERMINAL = new Set(['completed', 'cancelled', 'failed'])

export function useOrderPolling(orderId: string | null, targetStatus?: string) {
  return useQuery<OrderResp>({
    queryKey: ['order', orderId],
    queryFn: () => ordersApi.get(orderId!),
    enabled: !!orderId,
    refetchInterval: (query) => {
      const status = query.state.data?.status
      if (!status) return 5_000
      if (TERMINAL.has(status)) return false
      if (targetStatus && status === targetStatus) return false
      return 5_000
    },
  })
}
