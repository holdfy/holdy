import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { disputeApi } from '../../lib/api'
import { Layout, Card, Btn, ErrorMsg } from '../../components/Layout'
import { EvidenceUploader } from '../../components/EvidenceUploader'

export function DisputaPage() {
  const { id } = useParams<{ id: string }>()

  const { data: dispute, isLoading, refetch } = useQuery({
    queryKey: ['dispute', id],
    queryFn: () => disputeApi.get(id!),
    enabled: !!id,
    refetchInterval: (q) => {
      const s = q.state.data?.status
      return s === 'open' ? 10_000 : false
    },
  })

  const [evidenceCount, setEvidenceCount] = useState(0)
  const [analyzeLoading, setAnalyzeLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function finishEvidence() {
    if (!id) return
    setAnalyzeLoading(true); setError(null)
    try {
      await disputeApi.analyze(id)
      await refetch()
    } catch (err: any) {
      setError(err.message ?? 'Erro ao acionar análise')
    } finally {
      setAnalyzeLoading(false)
    }
  }

  if (isLoading) return <Layout><p className="text-center text-gray-400 py-12">Carregando disputa…</p></Layout>
  if (!dispute) return <Layout><p className="text-center text-gray-400 py-12">Disputa não encontrada</p></Layout>

  const resolved = dispute.status === 'resolved' || dispute.status === 'closed'

  return (
    <Layout title="Disputa">
      {/* Cabeçalho */}
      <Card className="mb-4 space-y-2">
        <div className="flex justify-between items-center">
          <p className="font-semibold text-gray-800">Disputa #{dispute.dispute_id.slice(0, 8)}</p>
          <span className={`text-xs px-2.5 py-0.5 rounded-full font-medium ${
            resolved ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
          }`}>
            {resolved ? 'Resolvida' : 'Em análise'}
          </span>
        </div>
        <p className="text-sm text-gray-500">Motivo: <strong>{dispute.reason}</strong></p>
        {!resolved && (
          <p className="text-xs text-gray-400">Prazo: {new Date(dispute.deadline_at).toLocaleDateString('pt-BR')}</p>
        )}
      </Card>

      {/* Resultado */}
      {resolved && dispute.resolution_type && (
        <Card className={`mb-4 text-center py-6 space-y-2 ${
          dispute.resolution_type.includes('buyer')
            ? 'bg-green-50 border-green-200'
            : 'bg-red-50 border-red-200'
        }`}>
          <div className="text-4xl">
            {dispute.resolution_type.includes('buyer') ? '✅' : '❌'}
          </div>
          <p className="font-bold text-lg">
            {dispute.resolution_type.includes('buyer')
              ? 'Decidido em seu favor!'
              : 'Decidido em favor do vendedor'}
          </p>
          {dispute.ai_verdict && (
            <p className="text-sm text-gray-500">
              {dispute.resolution_type.includes('buyer')
                ? 'O valor será estornado para você.'
                : 'O valor foi liberado ao vendedor.'}
            </p>
          )}
        </Card>
      )}

      {/* Evidências enviadas */}
      {dispute.evidence.length > 0 && (
        <Card className="mb-4 space-y-3">
          <p className="text-sm font-semibold text-gray-700">Evidências ({dispute.evidence.length})</p>
          {dispute.evidence.map(e => (
            <div key={e.id} className="text-sm border-l-2 border-brand pl-3 space-y-1">
              <p className="text-xs text-gray-400 uppercase font-medium">{e.kind} · {e.party}</p>
              {e.content && <p className="text-gray-700">{e.content}</p>}
              {e.minio_url && (
                <a href={e.minio_url} target="_blank" rel="noreferrer">
                  <img src={e.minio_url} alt="evidência" className="max-h-40 rounded-lg" />
                </a>
              )}
            </div>
          ))}
        </Card>
      )}

      {/* Coletar evidências (disputa aberta) */}
      {dispute.status === 'open' && (
        <Card className="space-y-4">
          <p className="font-medium text-gray-800">Adicionar evidências</p>
          <p className="text-sm text-gray-500">
            Envie fotos, vídeos, código de rastreio ou descrição do problema. Máximo 5 evidências.
          </p>
          <EvidenceUploader orderId={id!} onUploaded={setEvidenceCount} />
          <ErrorMsg msg={error} />
          {(evidenceCount > 0 || dispute.evidence.length > 0) && (
            <Btn onClick={finishEvidence} disabled={analyzeLoading} variant="secondary">
              {analyzeLoading ? 'Enviando para análise…' : '✓ Concluir e acionar análise IA'}
            </Btn>
          )}
          <p className="text-xs text-gray-400 text-center">
            A análise é feita automaticamente. Para valores abaixo de R$ 2.000 com evidências claras, a decisão é imediata.
          </p>
        </Card>
      )}

      {/* Aguardando decisão */}
      {dispute.status === 'open' && evidenceCount === 0 && dispute.evidence.length === 0 && (
        <Card className="text-center py-6 space-y-2 bg-yellow-50 border-yellow-200">
          <div className="text-3xl animate-pulse">🔍</div>
          <p className="font-medium text-yellow-800">Análise em andamento</p>
          <p className="text-sm text-yellow-700">Você será notificado com o resultado.</p>
        </Card>
      )}

      <div className="mt-4">
        <Link to={`/comprador/pedido/${id}`} className="text-sm text-gray-400 hover:text-brand">
          ← Voltar ao pedido
        </Link>
      </div>
    </Layout>
  )
}
