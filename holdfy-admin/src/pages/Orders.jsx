import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Box, Typography, Alert, TextField, MenuItem, Stack, Chip, ToggleButton, ToggleButtonGroup,
} from "@mui/material";
import { DataGrid } from "@mui/x-data-grid";
import { adminApi } from "../api";

const STATUS_COLORS = {
  Created: "default",
  Paid: "info",
  InCustody: "warning",
  Disputed: "error",
  Settled: "success",
  Cancelled: "default",
};

const PLATFORM_LABELS = {
  whatsapp: "WhatsApp",
  site: "Site",
  app_ios: "App iOS",
  app_android: "App Android",
};

const PLATFORM_COLORS = {
  whatsapp: "success",
  site: "info",
  app_ios: "default",
  app_android: "default",
};

const columns = [
  { field: "order_id", headerName: "ID", width: 280, renderCell: (p) => p.value?.slice(0, 8) + "…" },
  {
    field: "person_type",
    headerName: "Tipo",
    width: 80,
    renderCell: (p) => {
      const v = p.value;
      const isPJ = v === "legal" || v === "pj";
      return <Chip label={isPJ ? "PJ" : "PF"} color={isPJ ? "primary" : "default"} size="small" />;
    },
  },
  {
    field: "amount_minor",
    headerName: "Valor (R$)",
    width: 120,
    valueFormatter: (v) => (parseFloat(v) / 100).toLocaleString("pt-BR", { minimumFractionDigits: 2 }),
  },
  {
    field: "status",
    headerName: "Status",
    width: 130,
    renderCell: (p) => <Chip label={p.value} color={STATUS_COLORS[p.value] ?? "default"} size="small" />,
  },
  { field: "risk_score", headerName: "Score", width: 90, type: "number" },
  { field: "risk_decision", headerName: "Decisão", width: 100 },
  {
    field: "platform_origin",
    headerName: "Plataforma",
    width: 130,
    renderCell: (p) => (
      <Chip
        label={PLATFORM_LABELS[p.value] ?? p.value ?? "—"}
        color={PLATFORM_COLORS[p.value] ?? "default"}
        size="small"
      />
    ),
  },
  {
    field: "created_at",
    headerName: "Criado em",
    width: 180,
    valueFormatter: (v) => v ? new Date(v).toLocaleString("pt-BR") : "—",
  },
];

function matchesPersonFilter(order, filter) {
  if (filter === "all") return true;
  const isPJ = order.person_type === "legal" || order.person_type === "pj";
  return filter === "pj" ? isPJ : !isPJ;
}

function matchesPlatformFilter(order, filter) {
  if (filter === "all") return true;
  return order.platform_origin === filter;
}

export default function Orders() {
  const [status, setStatus] = useState("");
  const [personFilter, setPersonFilter] = useState("all");
  const [platformFilter, setPlatformFilter] = useState("all");

  const { data, isLoading, error } = useQuery({
    queryKey: ["orders", status],
    queryFn: () => adminApi.listOrders(status ? { status } : {}),
    refetchInterval: 60_000,
  });

  const rows = (data?.orders ?? [])
    .filter((o) => matchesPersonFilter(o, personFilter))
    .filter((o) => matchesPlatformFilter(o, platformFilter))
    .map((o, i) => ({ id: i, ...o }));

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Pedidos
      </Typography>
      <Stack direction="row" spacing={2} mb={2} alignItems="center" flexWrap="wrap" rowGap={2}>
        <TextField
          select
          label="Status"
          value={status}
          onChange={(e) => setStatus(e.target.value)}
          size="small"
          sx={{ minWidth: 160 }}
        >
          <MenuItem value="">Todos</MenuItem>
          {Object.keys(STATUS_COLORS).map((s) => (
            <MenuItem key={s} value={s}>{s}</MenuItem>
          ))}
        </TextField>
        <ToggleButtonGroup
          value={personFilter}
          exclusive
          onChange={(_, v) => v && setPersonFilter(v)}
          size="small"
        >
          <ToggleButton value="all">Todos</ToggleButton>
          <ToggleButton value="pf">PF</ToggleButton>
          <ToggleButton value="pj">PJ</ToggleButton>
        </ToggleButtonGroup>
        <ToggleButtonGroup
          value={platformFilter}
          exclusive
          onChange={(_, v) => v && setPlatformFilter(v)}
          size="small"
        >
          <ToggleButton value="all">Todas plataformas</ToggleButton>
          <ToggleButton value="whatsapp">WhatsApp</ToggleButton>
          <ToggleButton value="site">Site</ToggleButton>
          <ToggleButton value="app_ios">App iOS</ToggleButton>
          <ToggleButton value="app_android">App Android</ToggleButton>
        </ToggleButtonGroup>
      </Stack>
      {error && <Alert severity="error" sx={{ mb: 2 }}>Erro: {error.message}</Alert>}
      <DataGrid
        rows={rows}
        columns={columns}
        loading={isLoading}
        autoHeight
        pageSizeOptions={[25, 50, 100]}
        initialState={{ pagination: { paginationModel: { pageSize: 25 } } }}
        disableRowSelectionOnClick
      />
      {data && (
        <Typography variant="body2" color="text.secondary" mt={1}>
          {rows.length} pedido(s) exibido(s)
          {personFilter !== "all" ? ` (${personFilter.toUpperCase()})` : ""}
          {data.total ? ` de ${data.total} total` : ""}
        </Typography>
      )}
    </Box>
  );
}
