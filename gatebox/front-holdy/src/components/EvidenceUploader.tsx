import { useRef, useState } from 'react'
import { disputeApi } from '../lib/api'

interface Props {
  orderId: string
  onUploaded?: (count: number) => void
}

export function EvidenceUploader({ orderId, onUploaded }: Props) {
  const [count, setCount] = useState(0)
  const [text, setText] = useState('')
  const [loading, setLoading] = useState(false)
  const fileRef = useRef<HTMLInputElement>(null)

  async function sendText() {
    if (!text.trim()) return
    setLoading(true)
    try {
      await disputeApi.addTextEvidence(orderId, 'message', text.trim())
      const next = count + 1
      setCount(next)
      setText('')
      onUploaded?.(next)
    } finally {
      setLoading(false)
    }
  }

  async function sendFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    setLoading(true)
    try {
      const form = new FormData()
      form.append('file', file)
      form.append('kind', file.type.startsWith('video') ? 'video' : 'photo')
      await disputeApi.addMediaEvidence(orderId, form)
      const next = count + 1
      setCount(next)
      onUploaded?.(next)
    } finally {
      setLoading(false)
      if (fileRef.current) fileRef.current.value = ''
    }
  }

  return (
    <div className="space-y-3">
      {count > 0 && (
        <p className="text-sm text-green-700 font-medium">✓ {count} evidência{count > 1 ? 's' : ''} enviada{count > 1 ? 's' : ''}</p>
      )}

      <div className="flex gap-2">
        <input
          value={text}
          onChange={e => setText(e.target.value)}
          placeholder="Descreva o problema ou código de rastreio…"
          className="flex-1 border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-brand"
          onKeyDown={e => e.key === 'Enter' && sendText()}
        />
        <button
          onClick={sendText}
          disabled={loading || !text.trim()}
          className="px-4 py-2 bg-brand text-white rounded-lg text-sm font-medium disabled:opacity-50"
        >
          Enviar
        </button>
      </div>

      <div>
        <input ref={fileRef} type="file" accept="image/*,video/*" className="hidden" onChange={sendFile} />
        <button
          onClick={() => fileRef.current?.click()}
          disabled={loading}
          className="w-full border-2 border-dashed border-gray-300 rounded-xl py-4 text-sm text-gray-500 hover:border-brand hover:text-brand transition-colors disabled:opacity-50"
        >
          📎 Anexar foto ou vídeo
        </button>
      </div>

      {loading && <p className="text-xs text-gray-400 animate-pulse">Enviando…</p>}
    </div>
  )
}
