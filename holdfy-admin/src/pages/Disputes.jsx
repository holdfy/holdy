import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Chip, Button, Dialog, DialogTitle,
  DialogContent, DialogActions, TextField, MenuItem, Stack,
} from "@mui/material";
import { DataGrid } from "@mui/x-data-grid";
import { adminApi } from "../api";

const STATUS_COLORS = { Open: "error", UnderReview: "warning", Resolved: "success", Closed: "default" };
const RESOLUTION_TYPES = ["FavorBuyer", "FavorSeller", "Split", "Rejected"];

export default function Disputes() {
  const [resolveDialog, setResolveDialog] = useState(null);
  const [resolution, setResolution] = useState("FavorBuyer");
  const [notes, setNotes] = useState("");
  const qc = useQueryClient();

  const { data: disputes = [], isLoading, error } = useQuery({
    queryKey: ["disputes"],
    queryFn: adminApi.listDisputes,
    refetchInterval: 30_000,
  });

  const resolveMutation = useMutation({
    mutationFn: ({ id }) => adminApi.resolveDispute(id, { resolution, notes: notes || null }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["disputes"] });
      qc.invalidateQueries({ queryKey: ["dashboard"] });
      setResolveDialog(null);
      setNotes("");
    },
  });

  const columns = [
    { field: "id", headerName: "ID", width: 280, renderCell: (p) => String(p.value).slice(0, 8) + "…" },
    { field: "order_id", headerName: "Pedido", width: 280, renderCell: (p) => p.value ? String(p.value).slice(0, 8) + "…" : "—" },
    {
      field: "status",
      headerName: "Status",
      width: 130,
      renderCell: (p) => <Chip label={p.value} color={STATUS_COLORS[p.value] ?? "default"} size="small" />,
    },
    { field: "reason", headerName: "Motivo", flex: 1 },
    {
      field: "created_at",
      headerName: "Aberta em",
      width: 180,
      valueFormatter: (v) => v ? new Date(v).toLocaleString("pt-BR") : "—",
    },
    {
      field: "actions",
      headerName: "Ação",
      width: 120,
      renderCell: (p) =>
        p.row.status === "Open" || p.row.status === "UnderReview" ? (
          <Button size="small" variant="outlined" onClick={() => setResolveDialog(p.row)}>
            Resolver
          </Button>
        ) : null,
    },
  ];

  const rows = disputes.map((d, i) => ({ id: d.id ?? i, ...d }));

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Disputas
      </Typography>
      {error && <Alert severity="error" sx={{ mb: 2 }}>Erro: {error.message}</Alert>}
      <DataGrid
        rows={rows}
        columns={columns}
        loading={isLoading}
        autoHeight
        pageSizeOptions={[25, 50]}
        initialState={{ pagination: { paginationModel: { pageSize: 25 } } }}
        disableRowSelectionOnClick
      />

      <Dialog open={!!resolveDialog} onClose={() => setResolveDialog(null)} maxWidth="sm" fullWidth>
        <DialogTitle>Resolver Disputa</DialogTitle>
        <DialogContent>
          <Stack spacing={2} mt={1}>
            <TextField
              select
              label="Resolução"
              value={resolution}
              onChange={(e) => setResolution(e.target.value)}
              fullWidth
            >
              {RESOLUTION_TYPES.map((r) => <MenuItem key={r} value={r}>{r}</MenuItem>)}
            </TextField>
            <TextField
              label="Notas (opcional)"
              multiline
              rows={3}
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              fullWidth
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setResolveDialog(null)}>Cancelar</Button>
          <Button
            variant="contained"
            color="primary"
            disabled={resolveMutation.isPending}
            onClick={() => resolveMutation.mutate({ id: resolveDialog?.id })}
          >
            {resolveMutation.isPending ? "Salvando..." : "Confirmar"}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
