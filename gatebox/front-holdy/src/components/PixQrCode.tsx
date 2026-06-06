import { useState } from 'react'

interface Props {
  brCode: string
  amount: string
}

export function PixQrCode({ brCode, amount }: Props) {
  const [copied, setCopied] = useState(false)

  async function copy() {
    await navigator.clipboard.writeText(brCode)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  // QR image via public API (no server needed)
  const qrUrl = `https://api.qrserver.com/v1/create-qr-code/?size=240x240&data=${encodeURIComponent(brCode)}`

  return (
    <div className="flex flex-col items-center gap-4">
      <p className="text-2xl font-bold text-brand">R$ {amount}</p>
      <img src={qrUrl} alt="QR PIX" className="rounded-xl border border-gray-200 shadow" width={240} height={240} />
      <p className="text-sm text-gray-500">Escaneie ou copie o código abaixo</p>
      <div className="w-full bg-gray-50 rounded-lg border border-gray-200 p-3">
        <p className="text-xs font-mono break-all text-gray-700 select-all">{brCode}</p>
      </div>
      <button
        onClick={copy}
        className="w-full py-3 rounded-xl font-semibold text-white bg-brand hover:bg-brand-dark transition-colors"
      >
        {copied ? '✓ Copiado!' : 'Copiar código PIX'}
      </button>
    </div>
  )
}
