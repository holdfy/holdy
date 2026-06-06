const MAP: Record<string, { label: string; cls: string }> = {
  pending_funding: { label: 'Aguardando pagamento', cls: 'bg-yellow-100 text-yellow-800' },
  in_custody:      { label: 'Em custódia',           cls: 'bg-blue-100 text-blue-800' },
  completed:       { label: 'Concluído',              cls: 'bg-green-100 text-green-800' },
  cancelled:       { label: 'Cancelado',              cls: 'bg-gray-100 text-gray-600' },
  failed:          { label: 'Falhou',                 cls: 'bg-red-100 text-red-700' },
}

export function OrderStatusBadge({ status }: { status: string }) {
  const { label, cls } = MAP[status] ?? { label: status, cls: 'bg-gray-100 text-gray-600' }
  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${cls}`}>
      {label}
    </span>
  )
}
