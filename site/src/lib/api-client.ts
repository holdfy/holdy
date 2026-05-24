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
  seller_id: string;
  amount: string;
  status: "pending_funding" | "in_custody" | "completed" | "cancelled" | "failed";
  pix_br_code: string | null;
  fiat_rail: string | null;
  risk_score: number | null;
  risk_decision: "approve" | "review" | "block" | null;
  description?: string | null;
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
  buyer_id: string;
  amount: string;
  description?: string;
}

export interface ReleaseCustodyRequest {
  order_id: string;
  released_by: string;
  idempotency_key: string;
}

export interface ApiError {
  error: string;
  status: number;
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
    const err: ApiError = { error: body.error ?? res.statusText, status: res.status };
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

  // Orders
  createOrder: (data: CreateOrderRequest) =>
    request<OrderResponse>("/orders", {
      method: "POST",
      body: JSON.stringify(data),
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

  acceptProposal: (id: string, cpf?: string) =>
    request<AcceptProposalResponse>(`/proposals/${id}/accept`, {
      method: "POST",
      body: JSON.stringify({ cpf }),
    }),

  rejectProposal: (id: string) =>
    request<ProposalResponse>(`/proposals/${id}/reject`, { method: "POST" }),

  // Custody
  releaseCustody: (data: ReleaseCustodyRequest) =>
    request(`/custody/release`, {
      method: "POST",
      body: JSON.stringify(data),
    }),
};
