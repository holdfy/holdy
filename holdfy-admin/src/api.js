// apicash-frontend (Leptos, :3002) expõe /svc/admin/* como proxy reverso pro
// apicash-admin-backend (:3001), stripando o prefixo /svc/admin antes de repassar.
// Porta 3001 não é alcançável direto entre nós no load balancer — só via essa ponte.
const BASE = "/svc/admin";

const KEY_STORAGE = "holdfy_admin_key";

export const adminKeyStore = {
  get: () => localStorage.getItem(KEY_STORAGE) ?? "",
  set: (key) => localStorage.setItem(KEY_STORAGE, key),
  clear: () => localStorage.removeItem(KEY_STORAGE),
};

async function req(path, options = {}) {
  const key = adminKeyStore.get();
  const resp = await fetch(`${BASE}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      "X-API-Key": key,
      ...options.headers,
    },
  });
  if (!resp.ok) {
    const body = await resp.json().catch(() => null);
    const err = new Error(body?.error || `HTTP ${resp.status}`);
    err.status = resp.status;
    throw err;
  }
  return resp.json();
}

export const adminApi = {
  dashboard: () => req("/admin/dashboard"),
  listOrders: (params = {}) => {
    const qs = new URLSearchParams(
      Object.fromEntries(Object.entries(params).filter(([, v]) => v != null))
    ).toString();
    return req(`/admin/orders${qs ? `?${qs}` : ""}`);
  },
  listDisputes: () => req("/admin/disputes"),
  resolveDispute: (id, resolution) =>
    req(`/admin/disputes/${id}/resolve`, {
      method: "POST",
      body: JSON.stringify(resolution),
    }),
  listScores: () => req("/admin/users/score"),
  listStellarTransactions: () => req("/admin/stellar/transactions"),
  yieldReport: (params = {}) => {
    const qs = new URLSearchParams(
      Object.fromEntries(Object.entries(params).filter(([, v]) => v != null))
    ).toString();
    return req(`/admin/reports/yield${qs ? `?${qs}` : ""}`);
  },
  devStatus: () => req("/admin/dev/status"),
  devSettleOrder: (id) => req(`/admin/dev/orders/${id}/settle`, { method: "POST" }),
  devReleaseOrder: (id) => req(`/admin/dev/orders/${id}/release`, { method: "POST" }),
  devForceAcceptProposal: (id, buyerId) =>
    req(`/admin/dev/proposals/${id}/force-accept`, {
      method: "POST",
      body: JSON.stringify({ buyer_id: buyerId || undefined }),
    }),
  devWallet: () => req("/admin/dev/wallet"),
  devWalletMint: (amount) =>
    req("/admin/dev/wallet/mint", { method: "POST", body: JSON.stringify({ amount }) }),
};
