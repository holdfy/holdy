import { useQuery } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Chip, Tooltip, IconButton, Link, Stack,
} from "@mui/material";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import { DataGrid } from "@mui/x-data-grid";
import { adminApi } from "../api";

// ── helpers ───────────────────────────────────────────────────────────────────

function fmtDoc(doc) {
  if (!doc) return "—";
  const d = doc.replace(/\D/g, "");
  if (d.length === 11) return d.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, "$1.$2.$3-$4");
  if (d.length === 14) return d.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, "$1.$2.$3/$4-$5");
  return doc || "—";
}

function fmtBRL(v) {
  if (!v) return "—";
  const n = parseFloat(v);
  return isNaN(n) ? v : n.toLocaleString("pt-BR", { style: "currency", currency: "BRL" });
}

function short(hash, len = 10) {
  if (!hash) return "—";
  return hash.length > len ? hash.slice(0, 8) + "…" : hash;
}

function NetworkChip({ network }) {
  if (network === "mainnet")
    return <Chip label="Mainnet" size="small" sx={{ bgcolor: "#059669", color: "#fff", fontWeight: 700, fontSize: 11 }} />;
  if (network === "testnet")
    return <Chip label="Testnet" size="small" sx={{ bgcolor: "#7c3aed", color: "#fff", fontWeight: 700, fontSize: 11 }} />;
  return <Chip label="Simulado" size="small" sx={{ bgcolor: "#6b7280", color: "#fff", fontWeight: 700, fontSize: 11 }} />;
}

function ModeChip({ mode }) {
  if (mode === "real")
    return <Chip label="On-chain real" size="small" color="success" sx={{ fontWeight: 700, fontSize: 11 }} />;
  if (mode === "mock")
    return <Chip label="Mock" size="small" color="warning" sx={{ fontWeight: 700, fontSize: 11 }} />;
  return <Chip label="Simulado" size="small" sx={{ bgcolor: "#e5e7eb", fontSize: 11 }} />;
}

function StatusChip({ status }) {
  const map = {
    completed: "success", in_custody: "warning", pending_funding: "info",
    failed: "error", cancelled: "default", locked: "warning", released: "success",
    disputed: "error",
  };
  const normalized = (status || "").toLowerCase().replace(/_/g, "_");
  return <Chip label={status || "—"} size="small" color={map[normalized] ?? "default"} sx={{ fontSize: 11 }} />;
}

function HashCell({ hash, url }) {
  if (!hash) return <span style={{ color: "#9ca3af" }}>—</span>;
  return (
    <Stack direction="row" alignItems="center" spacing={0.5}>
      <Tooltip title={hash} placement="top">
        <span style={{ fontFamily: "monospace", fontSize: 11 }}>{short(hash)}</span>
      </Tooltip>
      {url && (
        <IconButton size="small" component="a" href={url} target="_blank" sx={{ p: 0.2 }}>
          <OpenInNewIcon sx={{ fontSize: 13, color: "#7c3aed" }} />
        </IconButton>
      )}
    </Stack>
  );
}

// ── colunas ───────────────────────────────────────────────────────────────────

const columns = [
  {
    field: "order_id", headerName: "Pedido", width: 110,
    renderCell: (p) => (
      <Tooltip title={p.value} placement="top">
        <span style={{ fontFamily: "monospace", fontSize: 11 }}>{p.value?.slice(0, 8)}…</span>
      </Tooltip>
    ),
  },
  {
    field: "buyer_name", headerName: "Comprador", width: 160,
    renderCell: (p) => (
      <Tooltip title={p.value || ""} placement="top">
        <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", display: "block" }}>
          {p.value || "—"}
        </span>
      </Tooltip>
    ),
  },
  {
    field: "buyer_document", headerName: "CPF / CNPJ", width: 145,
    renderCell: (p) => <span style={{ fontFamily: "monospace", fontSize: 11 }}>{fmtDoc(p.value)}</span>,
  },
  {
    field: "amount_brl", headerName: "Valor", width: 120,
    renderCell: (p) => <strong>{fmtBRL(p.value)}</strong>,
  },
  {
    field: "order_status", headerName: "Status Pedido", width: 130,
    renderCell: (p) => <StatusChip status={p.value} />,
  },
  {
    field: "custody_status", headerName: "Custódia", width: 110,
    renderCell: (p) => p.value ? <StatusChip status={p.value} /> : <span style={{ color: "#9ca3af" }}>—</span>,
  },
  {
    field: "network", headerName: "Rede", width: 100,
    renderCell: (p) => <NetworkChip network={p.value} />,
  },
  {
    field: "soroban_mode", headerName: "Modo Soroban", width: 130,
    renderCell: (p) => <ModeChip mode={p.value} />,
  },
  {
    field: "soroban_lock_tx_hash", headerName: "TX Lock", width: 140,
    renderCell: (p) => {
      const row = p.row;
      return <HashCell hash={p.value} url={row.explorer_lock_url} />;
    },
  },
  {
    field: "soroban_release_tx_hash", headerName: "TX Release", width: 140,
    renderCell: (p) => <HashCell hash={p.value} url={null} />,
  },
  {
    field: "soroban_escrow_contract_id", headerName: "Contrato Escrow", width: 145,
    renderCell: (p) => {
      const row = p.row;
      return <HashCell hash={p.value} url={row.explorer_contract_url} />;
    },
  },
  {
    field: "created_at", headerName: "Data / Hora", width: 165,
    valueFormatter: (v) => v ? new Date(v).toLocaleString("pt-BR") : "—",
  },
];

// ── component ─────────────────────────────────────────────────────────────────

export default function StellarTransactions() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["stellar-transactions"],
    queryFn: adminApi.listStellarTransactions,
    refetchInterval: 60_000,
  });

  const network = data?.network ?? "";
  const rows = (data?.transactions ?? []).map((t, i) => ({ id: i, ...t }));

  return (
    <Box>
      {/* Cabeçalho */}
      <Stack direction="row" alignItems="flex-start" justifyContent="space-between" mb={3} flexWrap="wrap" rowGap={1}>
        <Box>
          <Typography variant="h5" fontWeight={700}>
            Transações Stellar
          </Typography>
          <Stack direction="row" alignItems="center" spacing={1} mt={0.5}>
            <Typography variant="body2" color="text.secondary">
              Pedidos com escrow Soroban — lock, release e hashes on-chain
            </Typography>
            {network && <NetworkChip network={network} />}
          </Stack>
        </Box>
        {network === "testnet" && (
          <Link
            href="https://stellar.expert/explorer/testnet"
            target="_blank"
            underline="hover"
            sx={{ fontSize: 13, color: "#7c3aed", display: "flex", alignItems: "center", gap: 0.5 }}
          >
            <OpenInNewIcon sx={{ fontSize: 14 }} /> Stellar Expert (testnet)
          </Link>
        )}
        {network === "mainnet" && (
          <Link
            href="https://stellar.expert/explorer/public"
            target="_blank"
            underline="hover"
            sx={{ fontSize: 13, color: "#059669", display: "flex", alignItems: "center", gap: 0.5 }}
          >
            <OpenInNewIcon sx={{ fontSize: 14 }} /> Stellar Expert (mainnet)
          </Link>
        )}
      </Stack>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          Erro ao carregar: {error.message}
        </Alert>
      )}

      {!error && rows.length === 0 && !isLoading && (
        <Alert severity="info">
          Nenhuma transação Stellar encontrada. As transações aparecem após o primeiro pedido em modo{" "}
          <strong>testnet</strong> ou <strong>mainnet</strong> com Soroban activo.
        </Alert>
      )}

      <Box sx={{ height: "calc(100vh - 220px)", minHeight: 400 }}>
        <DataGrid
          rows={rows}
          columns={columns}
          loading={isLoading}
          pageSizeOptions={[25, 50, 100]}
          initialState={{ pagination: { paginationModel: { pageSize: 25 } } }}
          disableRowSelectionOnClick
          density="compact"
          sx={{
            "& .MuiDataGrid-cell": { alignItems: "center", display: "flex" },
            "& .MuiDataGrid-columnHeaderTitle": { fontWeight: 700, fontSize: 12 },
          }}
        />
      </Box>

      {/* Legenda */}
      <Stack direction="row" spacing={2} mt={2} flexWrap="wrap" rowGap={1}>
        <Typography variant="caption" color="text.secondary">Legenda:</Typography>
        <Chip label="On-chain real" size="small" color="success" sx={{ fontSize: 11 }} />
        <Chip label="Mock (hash fake)" size="small" color="warning" sx={{ fontSize: 11 }} />
        <Chip label="Simulado (sem Soroban)" size="small" sx={{ bgcolor: "#e5e7eb", fontSize: 11 }} />
        <Chip label="Testnet" size="small" sx={{ bgcolor: "#7c3aed", color: "#fff", fontSize: 11 }} />
        <Chip label="Mainnet" size="small" sx={{ bgcolor: "#059669", color: "#fff", fontSize: 11 }} />
      </Stack>
    </Box>
  );
}
