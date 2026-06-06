const BASE = import.meta.env.VITE_API_URL ?? 'http://localhost:3000'

function token(): string | null {
  return localStorage.getItem('hf_token')
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  opts: { auth?: boolean; multipart?: boolean } = {},
): Promise<T> {
  const headers: Record<string, string> = {}
  if (opts.auth !== false) {
    const t = token()
    if (t) headers['Authorization'] = `Bearer ${t}`
  }
  if (!opts.multipart) headers['Content-Type'] = 'application/json'

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: opts.multipart ? (body as FormData) : body ? JSON.stringify(body) : undefined,
  })

  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText)
    throw new ApiError(res.status, text)
  }

  const text = await res.text()
  return text ? JSON.parse(text) : ({} as T)
}

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

export const api = {
  get: <T>(path: string) => request<T>('GET', path),
  post: <T>(path: string, body?: unknown) => request<T>('POST', path, body),
  postMultipart: <T>(path: string, form: FormData) =>
    request<T>('POST', path, form, { multipart: true }),
}

// ── Auth ────────────────────────────────────────────────────────────────────
export interface LoginReq { email: string; password: string }
export interface RegisterReq { email: string; password: string; phone?: string; role?: string }
export interface AuthResp {
  access_token: string
  user_id: string
  role: string
}
export const authApi = {
  login: (b: LoginReq) => api.post<AuthResp>('/auth/login', b),
  register: (b: RegisterReq) => api.post<AuthResp>('/auth/register', b),
}

// ── Proposals ────────────────────────────────────────────────────────────────
export interface ProposalResp {
  id: string
  seller_id: string
  buyer_id: string | null
  amount: string
  description: string | null
  status: 'pending' | 'accepted' | 'rejected' | 'expired'
  created_at: string
  expires_at: string
  order_id: string | null
  listing_url?: string | null
  listing_photos?: string[]
  listing_title?: string | null
  listing_price_suggested?: string | null
}
export interface CreateProposalReq {
  buyer_id?: string
  amount: string
  description?: string
  listing_id?: string
}
export interface AcceptProposalReq {
  cpf?: string
  social_links?: string[]
}
export interface AcceptProposalResp {
  proposal_id: string
  order_id: string
  pix_br_code: string
  amount: string
  status: string
  funding_instruction?: string
}

export const proposalApi = {
  create: (b: CreateProposalReq) => api.post<ProposalResp>('/proposals', b),
  get: (id: string) => api.get<ProposalResp>(`/proposals/${id}`),
  accept: (id: string, b: AcceptProposalReq) =>
    api.post<AcceptProposalResp>(`/proposals/${id}/accept`, b),
  reject: (id: string) => api.post<ProposalResp>(`/proposals/${id}/reject`),
}

// ── Orders ────────────────────────────────────────────────────────────────────
export interface OrderResp {
  id: string
  buyer_id: string
  seller_id: string
  amount: string
  status: 'pending_funding' | 'in_custody' | 'completed' | 'cancelled' | 'failed'
  description: string | null
  pix_br_code: string | null
  tracking_code: string | null
  created_at: string
  risk_score?: number
  risk_decision?: string
}
export interface OrdersResp { orders: OrderResp[]; total: number }

export const ordersApi = {
  list: (role: 'buyer' | 'seller') => api.get<OrdersResp>(`/orders?role=${role}`),
  get: (id: string) => api.get<OrderResp>(`/orders/${id}`),
  offRamp: (id: string, pix_key?: string) =>
    api.post(`/orders/${id}/off-ramp`, { destination_pix_key: pix_key }),
  setTracking: (id: string, code: string) =>
    api.post(`/orders/${id}/tracking`, { tracking_code: code }),
}

// ── Custody ───────────────────────────────────────────────────────────────────
export interface ReleaseReq { order_id: string; released_by: string; idempotency_key: string }
export const custodyApi = {
  release: (b: ReleaseReq) => api.post('/custody/release', b),
}

// ── Disputes ─────────────────────────────────────────────────────────────────
export interface DisputeResp {
  dispute_id: string
  order_id: string
  status: 'open' | 'closed' | 'resolved'
  opened_by: string
  reason: string
  deadline_at: string
  resolved_at: string | null
  resolution_type: string | null
  ai_verdict: string | null
  high_risk_buyer: boolean
  evidence: EvidenceItem[]
}
export interface EvidenceItem {
  id: string
  party: string
  kind: string
  minio_url: string | null
  content: string | null
  created_at: string
}
export const disputeApi = {
  open: (orderId: string, reason: string) =>
    api.post<DisputeResp>(`/orders/${orderId}/dispute`, { reason }),
  get: (orderId: string) => api.get<DisputeResp>(`/orders/${orderId}/dispute`),
  addTextEvidence: (orderId: string, kind: string, content: string) =>
    api.post(`/orders/${orderId}/dispute/evidence`, { kind, content }),
  addMediaEvidence: (orderId: string, form: FormData) =>
    api.postMultipart(`/orders/${orderId}/dispute/evidence`, form),
  analyze: (orderId: string) => api.post(`/orders/${orderId}/dispute/analyze`),
}

// ── Listings ──────────────────────────────────────────────────────────────────
export interface ListingResp {
  listing_id?: string
  title: string
  source_url: string
  price_suggested?: string
  photos: string[]
  video_url?: string
}
export const listingApi = {
  import: (url: string, user_id: string) =>
    api.post<ListingResp>('/internal/listings/import', { url, user_id }),
}

// ── Seller dashboard ──────────────────────────────────────────────────────────
export interface SellerDashResp {
  seller_id: string
  total_orders: number
  completed_orders: number
  in_custody_orders: number
  total_volume_brl: string
  completed_volume_brl: string
}
export const sellerApi = {
  dashboard: () => api.get<SellerDashResp>('/seller/dashboard'),
}
