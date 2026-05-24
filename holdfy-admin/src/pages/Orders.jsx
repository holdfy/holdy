import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Box, Typography, Alert, TextField, MenuItem, Stack, Chip,
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

const columns = [
  { field: "order_id", headerName: "ID", width: 280, renderCell: (p) => p.value?.slice(0, 8) + "…" },
  { field: "amount_minor", headerName: "Valor (R$)", width: 120, valueFormatter: (v) => (parseFloat(v) / 100).toLocaleString("pt-BR", { minimumFractionDigits: 2 }) },
  {
    field: "status",
    headerName: "Status",
    width: 130,
    renderCell: (p) => <Chip label={p.value} color={STATUS_COLORS[p.value] ?? "default"} size="small" />,
  },
  { field: "risk_score", headerName: "Score", width: 90, type: "number" },
  { field: "risk_decision", headerName: "Decisão", width: 100 },
  {
    field: "created_at",
    headerName: "Criado em",
    width: 180,
    valueFormatter: (v) => v ? new Date(v).toLocaleString("pt-BR") : "—",
  },
];

export default function Orders() {
  const [status, setStatus] = useState("");

  const { data, isLoading, error } = useQuery({
    queryKey: ["orders", status],
    queryFn: () => adminApi.listOrders(status ? { status } : {}),
    refetchInterval: 60_000,
  });

  const rows = (data?.orders ?? []).map((o, i) => ({ id: i, ...o }));

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Pedidos
      </Typography>
      <Stack direction="row" spacing={2} mb={2}>
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
          {data.total} pedido(s) encontrado(s)
        </Typography>
      )}
    </Box>
  );
}
