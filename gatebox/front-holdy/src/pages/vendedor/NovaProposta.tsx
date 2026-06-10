import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../../context/AuthContext'
import { listingApi, proposalApi, type ListingResp } from '../../lib/api'
import { Layout, Card, Btn, Input, ErrorMsg } from '../../components/Layout'
import { StepWizard } from '../../components/StepWizard'
import { maskCpfCnpj, stripDoc, validateCpf, validateCnpj } from '../../lib/doc'

const STEPS = ['Anúncio', 'Valor', 'Seus dados', 'Compartilhar']

export function NovaProposta() {
  const { userId } = useAuth()
  const nav = useNavigate()
  const [step, setStep] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  // Step 0 — listing URL
  const [listingUrl, setListingUrl] = useState('')
  const [listing, setListing] = useState<ListingResp | null>(null)

  // Step 1 — amount + description
  const [amount, setAmount] = useState('')
  const [description, setDescription] = useState('')

  // Step 2 — seller doc + pix key
  const [sellerDoc, setSellerDoc] = useState('')
  const [pixKey, setPixKey] = useState('')

  // Step 3 — resultado
  const [proposalId, setProposalId] = useState<string | null>(null)

  async function importListing() {
    if (!listingUrl.trim()) { setStep(1); return }
    setLoading(true); setError(null)
    try {
      const r = await listingApi.import(listingUrl.trim(), userId!)
      setListing(r)
      if (r.price_suggested) setAmount(r.price_suggested)
      if (r.title) setDescription(r.title)
    } catch {
      // import falhou — continua sem anúncio
    } finally {
      setLoading(false)
      setStep(1)
    }
  }

  async function createProposal() {
    if (!amount.trim()) { setError('Informe o valor'); return }
    if (sellerDoc) {
      const doc = stripDoc(sellerDoc)
      if (doc.length === 11 && !validateCpf(doc)) { setError('CPF inválido. Verifique os dígitos.'); return }
      if (doc.length === 14 && !validateCnpj(doc)) { setError('CNPJ inválido. Verifique os dígitos.'); return }
      if (doc.length !== 11 && doc.length !== 14) { setError('Informe um CPF (11 dígitos) ou CNPJ (14 dígitos) válido.'); return }
    }
    setLoading(true); setError(null)
    try {
      const p = await proposalApi.create({
        amount: amount.trim().replace(',', '.'),
        description: description.trim() || undefined,
        listing_id: listing?.listing_id,
      })
      setProposalId(p.id)
      setStep(3)
    } catch (err: any) {
      setError(err.message ?? 'Erro ao criar proposta')
    } finally {
      setLoading(false)
    }
  }

  const proposalLink = proposalId
    ? `${window.location.origin}/proposta/${proposalId}`
    : ''

  const [copied, setCopied] = useState(false)
  function copyLink() {
    navigator.clipboard.writeText(proposalLink)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Layout title="Nova proposta">
      <StepWizard steps={STEPS} current={step} />

      {/* Step 0 — anúncio */}
      {step === 0 && (
        <Card className="space-y-4">
          <div>
            <p className="text-sm font-medium text-gray-700 mb-1">Link do anúncio <span className="text-gray-400">(opcional)</span></p>
            <input
              value={listingUrl}
              onChange={e => setListingUrl(e.target.value)}
              placeholder="https://www.instagram.com/p/..."
              className="w-full border border-gray-300 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-brand"
            />
            <p className="text-xs text-gray-400 mt-1">Instagram, Mercado Livre, OLX, Shopee, TikTok Shop…</p>
          </div>
          <Btn onClick={importListing} disabled={loading}>
            {loading ? 'Importando…' : listingUrl ? 'Importar e continuar' : 'Continuar sem anúncio'}
          </Btn>
        </Card>
      )}

      {/* Step 1 — valor */}
      {step === 1 && (
        <Card className="space-y-4">
          {listing && (
            <div className="flex gap-3 p-3 bg-brand-light rounded-xl">
              {listing.photos?.[0] && (
                <img src={listing.photos[0]} alt="" className="w-16 h-16 rounded-lg object-cover flex-shrink-0" />
              )}
              <div>
                <p className="text-sm font-medium text-gray-800 line-clamp-2">{listing.title}</p>
                {listing.price_suggested && (
                  <p className="text-xs text-brand mt-1">Preço sugerido: R$ {listing.price_suggested}</p>
                )}
              </div>
            </div>
          )}
          <Input
            label="Valor (R$)"
            type="number"
            min="1"
            step="0.01"
            value={amount}
            onChange={e => setAmount(e.target.value)}
            placeholder="250,00"
            required
            autoFocus
          />
          <div className="space-y-1">
            <label className="block text-sm font-medium text-gray-700">Descrição</label>
            <textarea
              value={description}
              onChange={e => setDescription(e.target.value)}
              placeholder="Ex: Tênis Nike Air Max preto tam 42, seminovo"
              className="w-full border border-gray-300 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-brand resize-none"
              rows={3}
            />
          </div>
          <ErrorMsg msg={error} />
          <div className="flex gap-2">
            <Btn variant="ghost" onClick={() => setStep(0)}>← Voltar</Btn>
            <Btn onClick={() => { if (!amount) { setError('Informe o valor'); return }; setError(null); setStep(2) }}>
              Continuar
            </Btn>
          </div>
        </Card>
      )}

      {/* Step 2 — dados do vendedor */}
      {step === 2 && (
        <Card className="space-y-4">
          <p className="text-sm text-gray-500">Seus dados são usados para antifraude e confirmação de identidade.</p>
          <Input
            label="Seu CPF ou CNPJ"
            value={sellerDoc}
            onChange={e => setSellerDoc(maskCpfCnpj(e.target.value))}
            placeholder="000.000.000-00"
            maxLength={18}
          />
          <Input
            label="Sua chave PIX (para receber)"
            value={pixKey}
            onChange={e => setPixKey(e.target.value)}
            placeholder="CPF, e-mail, celular ou chave aleatória"
          />
          <ErrorMsg msg={error} />
          <div className="flex gap-2">
            <Btn variant="ghost" onClick={() => setStep(1)}>← Voltar</Btn>
            <Btn onClick={createProposal} disabled={loading}>
              {loading ? 'Criando…' : 'Criar proposta'}
            </Btn>
          </div>
        </Card>
      )}

      {/* Step 3 — compartilhar */}
      {step === 3 && proposalId && (
        <Card className="space-y-4 text-center">
          <div className="text-5xl">🎉</div>
          <h2 className="text-lg font-bold text-gray-900">Proposta criada!</h2>
          <p className="text-sm text-gray-500">Compartilhe o link abaixo com o comprador. Quando ele aceitar, você recebe o aviso.</p>
          <div className="bg-gray-50 rounded-xl border border-gray-200 p-3">
            <p className="text-xs font-mono break-all text-gray-700 select-all">{proposalLink}</p>
          </div>
          <Btn onClick={copyLink} variant={copied ? 'ghost' : 'primary'}>
            {copied ? '✓ Link copiado!' : 'Copiar link'}
          </Btn>
          <div className="flex gap-2">
            <Btn variant="ghost" onClick={() => nav('/vendedor')}>Ver meus pedidos</Btn>
            <Btn variant="secondary" onClick={() => nav(`/vendedor/pedido/${proposalId}`)}>
              Acompanhar proposta
            </Btn>
          </div>
        </Card>
      )}
    </Layout>
  )
}
