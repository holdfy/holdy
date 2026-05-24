/**
 * API client para Gatebox
 */

const API_BASE = process.env.REACT_APP_API_BASE_URL || "http://localhost:8080/api/v1";

async function request(url, options = {}) {
  const headers = {
    "Content-Type": "application/json",
    ...options.headers,
  };
  const token = localStorage.getItem("customerToken") ||
    localStorage.getItem("adminToken") ||
    localStorage.getItem("backofficeToken");
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  const res = await fetch(url, { ...options, headers });
  if (res.status === 401) {
    localStorage.removeItem("customerToken");
    localStorage.removeItem("adminToken");
    localStorage.removeItem("backofficeToken");
    window.location.href = "/#/customer/login";
    throw new Error("Sessão expirada");
  }
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || err.error || res.statusText);
  }
  return res.json().catch(() => ({}));
}

function setAuthToken(profile, token) {
  localStorage.removeItem("customerToken");
  localStorage.removeItem("adminToken");
  localStorage.removeItem("backofficeToken");
  const key = profile === "admin" ? "adminToken" : profile === "backoffice" ? "backofficeToken" : "customerToken";
  if (token) localStorage.setItem(key, token);
}

// --- Customers API ---
export const customersApi = {
  auth: {
    login: (username, password) =>
      request(`${API_BASE}/customers/auth/login`, {
        method: "POST",
        body: JSON.stringify({ username, password }),
      }),
    register: (data) =>
      request(`${API_BASE}/customers/auth/register`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    profile: () => request(`${API_BASE}/customers/auth/profile`),
    updateProfile: (data) =>
      request(`${API_BASE}/customers/auth/profile`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    changePassword: (currentPassword, newPassword) =>
      request(`${API_BASE}/customers/auth/change-password`, {
        method: "POST",
        body: JSON.stringify({ currentPassword, newPassword }),
      }),
  },
  account: {
    balance: (full) => request(`${API_BASE}/customers/account/balance${full ? "?full=true" : ""}`),
    extract: (params) =>
      request(`${API_BASE}/customers/account/extract?${new URLSearchParams(params || {}).toString()}`),
    limits: () => request(`${API_BASE}/customers/account/limits`),
    keys: {
      list: () => request(`${API_BASE}/customers/account/keys`),
      create: (data) =>
        request(`${API_BASE}/customers/account/keys`, {
          method: "POST",
          body: JSON.stringify(data),
        }),
      delete: (id) =>
        request(`${API_BASE}/customers/account/keys/${id}`, { method: "DELETE" }),
    },
    webhooks: {
      list: () => request(`${API_BASE}/customers/account/webhooks`),
      create: (data) =>
        request(`${API_BASE}/customers/account/webhooks`, {
          method: "POST",
          body: JSON.stringify(data),
        }),
    },
  },
  pix: {
    send: (data) =>
      request(`${API_BASE}/customers/pix/send`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    decodeBrcode: (brcode) =>
      request(`${API_BASE}/customers/pix/decode-brcode`, {
        method: "POST",
        body: JSON.stringify({ brcode }),
      }),
    qrcode: (data) =>
      request(`${API_BASE}/customers/pix/qrcode`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    status: (params) =>
      request(`${API_BASE}/customers/pix/status?${new URLSearchParams(params || {}).toString()}`),
    transactions: (params) =>
      request(`${API_BASE}/customers/pix/transactions?${new URLSearchParams(params || {}).toString()}`),
    reversal: (data) =>
      request(`${API_BASE}/customers/pix/reversal`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
  },
  p2p: {
    send: (data) =>
      request(`${API_BASE}/customers/p2p/send`, {
        method: "POST",
        body: JSON.stringify({
          receiverId: data.receiverId ?? data.receiver_id,
          amount: data.amount,
          description: data.description,
        }),
      }),
    history: (params) =>
      request(`${API_BASE}/customers/p2p/history?${new URLSearchParams(params || {}).toString()}`),
    status: (transferId) => request(`${API_BASE}/customers/p2p/status/${transferId}`),
    search: (params) =>
      request(`${API_BASE}/customers/p2p/search?${new URLSearchParams(params || {}).toString()}`),
  },
};

// --- Admin API ---
export const adminApi = {
  auth: {
    login: (username, password) =>
      request(`${API_BASE}/admin/auth/login`, {
        method: "POST",
        body: JSON.stringify({ username, password }),
      }),
    profile: () => request(`${API_BASE}/admin/auth/profile`),
    changePassword: (currentPassword, newPassword) =>
      request(`${API_BASE}/admin/auth/change-password`, {
        method: "POST",
        body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
      }),
  },
  customers: {
    list: (params) =>
      request(`${API_BASE}/admin/customers?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/admin/customers/${id}`),
    balance: (id) => request(`${API_BASE}/admin/customers/${id}/balance`),
    extract: (id, params) =>
      request(`${API_BASE}/admin/customers/${id}/extract?${new URLSearchParams(params || {}).toString()}`),
    update: (id, data) =>
      request(`${API_BASE}/admin/customers/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (id) =>
      request(`${API_BASE}/admin/customers/${id}`, { method: "DELETE" }),
    approveKyc: (id) =>
      request(`${API_BASE}/admin/customers/${id}/kyc`, { method: "POST" }),
    createAccount: (id) =>
      request(`${API_BASE}/admin/customers/${id}/account`, { method: "POST" }),
  },
  reports: {
    profit: () => request(`${API_BASE}/admin/reports/profit`),
    customerActivities: (params) =>
      request(`${API_BASE}/admin/reports/customer-activities?${new URLSearchParams(params || {}).toString()}`),
    balanceDifferences: () => request(`${API_BASE}/admin/reports/balance-differences`),
  },
  pix: {
    transactions: (params) =>
      request(`${API_BASE}/admin/pix/transactions?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/admin/pix/transactions/${id}`),
    send: (data) =>
      request(`${API_BASE}/admin/pix/send`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    status: (params) =>
      request(`${API_BASE}/admin/pix/status?${new URLSearchParams(params || {}).toString()}`),
    qrcode: (data) =>
      request(`${API_BASE}/admin/pix/qrcode`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    cancel: (id) =>
      request(`${API_BASE}/admin/pix/transactions/${id}/cancel`, {
        method: "POST",
      }),
  },
  settings: {
    get: () => request(`${API_BASE}/admin/settings`),
    update: (data) =>
      request(`${API_BASE}/admin/settings`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    partners: {
      list: (params) =>
        request(`${API_BASE}/admin/settings/partners?${new URLSearchParams(params || {}).toString()}`),
      create: (data) =>
        request(`${API_BASE}/admin/settings/partners`, {
          method: "POST",
          body: JSON.stringify(data),
        }),
      update: (id, data) =>
        request(`${API_BASE}/admin/settings/partners/${id}`, {
          method: "PUT",
          body: JSON.stringify(data),
        }),
      delete: (id) =>
        request(`${API_BASE}/admin/settings/partners/${id}`, {
          method: "DELETE",
        }),
    },
  },
  webhooks: {
    list: (params) =>
      request(`${API_BASE}/admin/webhooks?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/admin/webhooks/${id}`),
    create: (data) =>
      request(`${API_BASE}/admin/webhooks`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (id, data) =>
      request(`${API_BASE}/admin/webhooks/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (id) =>
      request(`${API_BASE}/admin/webhooks/${id}`, { method: "DELETE" }),
    test: (id) =>
      request(`${API_BASE}/admin/webhooks/${id}/test`, { method: "POST" }),
  },
  disputes: {
    list: (params) =>
      request(`${API_BASE}/admin/disputes?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/admin/disputes/${id}`),
    create: (data) =>
      request(`${API_BASE}/admin/disputes`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    resolve: (id, resolution, notes) =>
      request(`${API_BASE}/admin/disputes/${id}/resolve`, {
        method: "POST",
        body: JSON.stringify({ resolution, notes }),
      }),
  },
};

// --- Backoffice API ---
export const backofficeApi = {
  auth: {
    login: (username, password) =>
      request(`${API_BASE}/backoffice/auth/login`, {
        method: "POST",
        body: JSON.stringify({ username, password }),
      }),
    profile: () => request(`${API_BASE}/backoffice/auth/profile`),
  },
  logs: {
    list: (params) =>
      request(`${API_BASE}/backoffice/logs?${new URLSearchParams(params || {}).toString()}`),
    metrics: () => request(`${API_BASE}/backoffice/logs/metrics`),
    transactions: (params) =>
      request(`${API_BASE}/backoffice/logs/transactions?${new URLSearchParams(params || {}).toString()}`),
    errors: (params) =>
      request(`${API_BASE}/backoffice/logs/errors?${new URLSearchParams(params || {}).toString()}`),
  },
  accounts: {
    list: (params) =>
      request(`${API_BASE}/backoffice/accounts?${new URLSearchParams(params || {}).toString()}`),
    statistics: () => request(`${API_BASE}/backoffice/accounts/statistics`),
    get: (id) => request(`${API_BASE}/backoffice/accounts/${id}`),
    transactions: (id, params) =>
      request(`${API_BASE}/backoffice/accounts/${id}/transactions?${new URLSearchParams(params || {}).toString()}`),
    updateStatus: (id, accountStatusId) =>
      request(`${API_BASE}/backoffice/accounts/${id}/status`, {
        method: "PUT",
        body: JSON.stringify({ accountStatusId }),
      }),
  },
};

// --- Anchor (auditoria blockchain) ---
export const anchorApi = {
  audit: (params) =>
    request(`${API_BASE}/anchor/audit?${new URLSearchParams(params || {}).toString()}`),
  proof: async (entityType, entityId) => {
    const res = await request(
      `${API_BASE}/anchor/audit?entity_type=${encodeURIComponent(entityType)}&entity_id=${encodeURIComponent(entityId)}&limit=1`
    );
    return res.items?.[0] || null;
  },
};

// --- Entidades ---
export const entityApi = {
  transaction: {
    list: (params) =>
      request(`${API_BASE}/transaction?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/transaction/${id}`),
  },
  secMed: {
    list: (params) =>
      request(`${API_BASE}/sec_med?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/sec_med/${id}`),
  },
  invoice: {
    list: (params) =>
      request(`${API_BASE}/invoice?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/invoice/${id}`),
  },
  keyPix: {
    list: (params) =>
      request(`${API_BASE}/key_pix?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/key_pix/${id}`),
    create: (data) =>
      request(`${API_BASE}/key_pix`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (id, data) =>
      request(`${API_BASE}/key_pix/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (id) =>
      request(`${API_BASE}/key_pix/${id}`, { method: "DELETE" }),
  },
  accounts: {
    list: (params) =>
      request(`${API_BASE}/accounts?${new URLSearchParams(params || {}).toString()}`),
    get: (id) => request(`${API_BASE}/accounts/${id}`),
  },
};

export { API_BASE, setAuthToken };
