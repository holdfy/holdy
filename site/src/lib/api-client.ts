// In development, Vite proxies /auth, /orders, /proposals, /custody → localhost:3000.
// Set VITE_API_URL to override (e.g. staging / production).
const BASE_URL = (import.meta.env.VITE_API_URL as string | undefined) ?? "";

const STORAGE_ACCESS = "apicash_access_token";
const STORAGE_REFRESH = "apicash_refresh_token";

// ─── Token storage ────────────────────────────────────────────────────────────

export const tokenStore = {
  getAccess: () => localStorage.getItem(STORAGE_ACCESS),
  getRefresh: () => localStorage.getItem(STORAGE_REFRESH),
  set: (access: string, refresh: string) => {
    localStorage.setItem(STORAGE_ACCESS, access);
    localStorage.setItem(STORAGE_REFRESH, refresh);
  },
  clear: () => {
    localStorage.removeItem(STORAGE_ACCESS);
    localStorage.removeItem(STORAGE_REFRESH);
  },
};

// ─── Types ────────────────────────────────────────────────────────────────────

export interface LoginResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token: string;
  refresh_expires_in: number;
}

export interface OrderResponse {
  id: string;
  buyer_id: string;
  buyer_name?: string | null;
  seller_id: string;
  amount: string;
  status: "pending_funding" | "in_custody" | "completed" | "cancelled" | "failed";
  pix_br_code: string | null;
  fiat_rail: string | null;
  risk_score: number | null;
  risk_decision: "approve" | "review" | "block" | null;
  description?: string | null;
  tracking_code?: string | null;
}

export interface ProposalResponse {
  id: string;
  seller_id: string;
  buyer_id: string;
  amount: string;
  description: string | null;
  status: "pending" | "accepted" | "rejected" | "expired";
  created_at: string;
  expires_at: string;
  order_id: string | null;
  seller_document?: string | null;
  listing_photo?: string | null;
  seller_phone?: string | null;
}

export interface AcceptProposalResponse {
  proposal_id: string;
  order_id: string;
  pix_br_code: string;
  amount: string;
  status: string;
  funding_instruction: string;
}

export interface CreateOrderRequest {
  buyer_id: string;
  seller_id: string;
  amount: string;
  cpf: string;
  description?: string;
  social_links?: string[];
}

export interface CreateProposalRequest {
  buyer_id?: string;
  amount: string;
  description?: string;
  seller_pix_key?: string;
  listing_id?: string;
  /** WhatsApp do vendedor — salvo em wa_contacts para notificações de rastreio. */
  seller_phone?: string;
}

export interface ReleaseCustodyRequest {
  order_id: string;
  released_by: string;
  idempotency_key: string;
}

export interface ApiError {
  error: string;
  status: number;
  /** Código estável opcional (ex.: "antifraud_block") — usar em vez de parsear `error`. */
  code?: string;
}

export interface ProfileResponse {
  phone: string | null;
  pix_key: string | null;
}

export interface WalletResponse {
  user_id: string;
  available_balance: string;
  pending_balance: string;
  currency: string;
}

export interface SellerDashboard {
  seller_id: string;
  total_orders: number;
  completed_orders: number;
  in_custody_orders: number;
  total_volume_brl: string;
  completed_volume_brl: string;
}

export interface OrdersListResponse {
  orders: OrderResponse[];
  total: number;
}

export interface ImportedProductDraft {
  listing_id: string | null;
  title: string;
  description: string | null;
  price_suggested: string | null;
  photos: string[];
  source_url: string;
  source_platform: string;
  extractor_used: string;
  guarantee: string | null;
  condition: string | null;
  location: string | null;
  seller_name: string | null;
  seller_rating: string | null;
  raw_attributes: Record<string, string>;
}

export interface ShippingQuoteRequest {
  from_postal_code: string;
  to_postal_code: string;
  weight_kg: string;
  width_cm: number;
  height_cm: number;
  length_cm: number;
}

export interface ShippingQuote {
  carrier: string;
  carrier_label: string;
  service_name: string;
  price_brl: string;
  estimated_days: number;
  currency: string;
}

export interface TrackingInfo {
  tracking_code: string;
  carrier: string;
  current_status: string;
  events: Array<{
    status: string;
    description: string;
    location: string | null;
    occurred_at: string;
  }>;
  estimated_delivery: string | null;
  provider_used: string;
}

export interface DisputeEvidenceItem {
  id: string;
  party: "buyer" | "seller";
  kind: string;
  minio_url: string | null;
  content: string | null;
  ai_flagged: boolean;
  created_at: string;
}

export interface DisputeResponse {
  dispute_id: string;
  order_id: string;
  status: "open" | "under_review" | "resolved" | "closed";
  opened_by: "buyer" | "seller";
  reason: string;
  deadline_at: string | null;
  resolved_at: string | null;
  resolution_type: string | null;
  resolution_notes: string | null;
  ai_verdict: "favor_buyer" | "favor_seller" | "inconclusive" | null;
  ai_confidence: number | null;
  high_risk_buyer: boolean;
  evidence: DisputeEvidenceItem[];
}

export interface ReputationSeal {
  name: "verified" | "premium" | "authenticated";
  label: string;
  badge_color: "blue" | "gold" | "green";
}

export interface ReputationResponse {
  user_id: string;
  score: number;
  completed_transactions: number;
  dispute_rate: string;
  seal: ReputationSeal | null;
  kyc_approved: boolean;
  computed_at: string;
}

// ─── HTTP core ────────────────────────────────────────────────────────────────

let refreshPromise: Promise<LoginResponse> | null = null;

async function refreshTokens(): Promise<LoginResponse> {
  if (refreshPromise) return refreshPromise;
  const token = tokenStore.getRefresh();
  if (!token) throw new Error("no_refresh_token");

  refreshPromise = fetch(`${BASE_URL}/auth/refresh`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ refresh_token: token }),
  })
    .then(async (res) => {
      if (!res.ok) throw new Error("refresh_failed");
      const data: LoginResponse = await res.json();
      tokenStore.set(data.access_token, data.refresh_token);
      return data;
    })
    .finally(() => {
      refreshPromise = null;
    });

  return refreshPromise;
}

async function request<T>(
  path: string,
  options: RequestInit = {},
  retry = true,
): Promise<T> {
  const access = tokenStore.getAccess();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };
  if (access) headers["Authorization"] = `Bearer ${access}`;

  const res = await fetch(`${BASE_URL}${path}`, { ...options, headers });

  if (res.status === 401 && retry) {
    try {
      await refreshTokens();
      return request<T>(path, options, false);
    } catch {
      tokenStore.clear();
      window.location.href = "/login";
      throw new Error("session_expired");
    }
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    const err: ApiError = { error: body.error ?? res.statusText, status: res.status, code: body.code };
    throw err;
  }

  // 204 No Content
  if (res.status === 204) return undefined as unknown as T;
  return res.json() as Promise<T>;
}

// ─── API methods ──────────────────────────────────────────────────────────────

export const api = {
  // Auth
  login: (username: string, password: string) =>
    request<LoginResponse>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    }),

  register: (document: string, password: string, name?: string) =>
    request<LoginResponse>("/auth/register", {
      method: "POST",
      body: JSON.stringify({ document, password, name, role: "buyer" }),
    }),

  getProfile: () => request<ProfileResponse>("/profile"),

  // Orders
  createOrder: (data: CreateOrderRequest) =>
    request<OrderResponse>("/orders", {
      method: "POST",
      body: JSON.stringify({ ...data, platform: "site" }),
    }),

  listOrders: (role: "buyer" | "seller" = "buyer") =>
    request<OrdersListResponse>(`/orders?role=${role}`),

  getOrder: (id: string) => request<OrderResponse>(`/orders/${id}`),

  getWallet: () => request<WalletResponse>("/wallet"),

  getSellerDashboard: () => request<SellerDashboard>("/seller/dashboard"),

  openDispute: (orderId: string, reason: string) =>
    request<{ dispute_id: string; order_id: string; status: string; message: string }>(
      `/orders/${orderId}/dispute`,
      { method: "POST", body: JSON.stringify({ reason }) },
    ),

  getDispute: (orderId: string) =>
    request<DisputeResponse>(`/orders/${orderId}/dispute`),

  offRamp: (orderId: string, destinationPixKey: string) =>
    request(`/orders/${orderId}/off-ramp`, {
      method: "POST",
      body: JSON.stringify({ destination_pix_key: destinationPixKey }),
    }),

  // Proposals
  createProposal: (data: CreateProposalRequest) =>
    request<ProposalResponse>("/proposals", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  getProposal: (id: string) => request<ProposalResponse>(`/proposals/${id}`),

  acceptProposal: (id: string, cpf?: string, buyerPhone?: string) =>
    request<AcceptProposalResponse>(`/proposals/${id}/accept`, {
      method: "POST",
      body: JSON.stringify({ cpf, buyer_phone: buyerPhone || undefined, platform: "site" }),
    }),

  rejectProposal: (id: string) =>
    request<ProposalResponse>(`/proposals/${id}/reject`, { method: "POST" }),

  // Reputation
  getReputation: (userId: string, params?: { completed?: number; dispute_count?: number; kyc_approved?: boolean }) => {
    const q = new URLSearchParams();
    if (params?.completed != null) q.set("completed", String(params.completed));
    if (params?.dispute_count != null) q.set("dispute_count", String(params.dispute_count));
    if (params?.kyc_approved != null) q.set("kyc_approved", String(params.kyc_approved));
    return request<ReputationResponse>(`/reputation/${userId}?${q.toString()}`);
  },

  // Custody
  releaseCustody: (data: ReleaseCustodyRequest) =>
    request(`/custody/release`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // Importer
  importListing: (url: string) =>
    request<ImportedProductDraft>("/v1/listings/import", {
      method: "POST",
      body: JSON.stringify({ url }),
    }),

  // Logistics
  quoteShipping: (data: ShippingQuoteRequest) =>
    request<{ quotes: ShippingQuote[] }>("/logistics/shipping/quote", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  trackShipment: (code: string) =>
    request<TrackingInfo>(`/logistics/tracking/${encodeURIComponent(code)}`),

  // KYC — Receita Federal lookup (cached 24h server-side)
  lookupDocument: (document: string) =>
    request<{ document: string; document_type: string; name: string | null; situation: string | null; source: string }>(
      `/kyc/document/${encodeURIComponent(document)}`,
    ),

  // Dispute evidence upload (base64 encoded content)
  addDisputeEvidence: (orderId: string, kind: string, content: string, ext?: string) =>
    request<{ evidence_id: string; dispute_id: string; kind: string; message: string }>(
      `/orders/${orderId}/dispute/evidence`,
      { method: "POST", body: JSON.stringify({ kind, content, ext }) },
    ),

  // Dispara a análise da IA sobre a evidência já enviada (fire-and-forget no backend)
  analyzeDispute: (orderId: string) =>
    request<{ dispute_id: string; order_id: string; status: string; message: string }>(
      `/orders/${orderId}/dispute/analyze`,
      { method: "POST", body: JSON.stringify({}) },
    ),

  // Logistics — tracking
  setTracking: (orderId: string, trackingCode: string) =>
    request<{ order_id: string; tracking_code: string }>(
      `/orders/${orderId}/tracking`,
      { method: "POST", body: JSON.stringify({ tracking_code: trackingCode }) },
    ),

  // Profile — chave PIX e WhatsApp
  updatePixKey: (pixKey: string) =>
    request<{ user_id: string; pix_key: string; status: string }>(
      "/profile/pix-key",
      { method: "PUT", body: JSON.stringify({ pix_key: pixKey }) },
    ),

  updatePhone: (phone: string) =>
    request<{ user_id: string; phone: string; status: string }>(
      "/profile/phone",
      { method: "PUT", body: JSON.stringify({ phone }) },
    ),

  // Profile — vincular CPF/CNPJ pós-login social
  linkDocument: (document: string) =>
    request<{ ok: boolean }>(
      "/auth/profile/link-document",
      { method: "POST", body: JSON.stringify({ document }) },
    ),
};
