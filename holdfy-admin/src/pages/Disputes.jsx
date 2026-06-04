import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Chip, Button, Dialog, DialogTitle,
  DialogContent, DialogActions, TextField, MenuItem, Stack,
  Slider, Divider, Grid, Tooltip, Paper,
} from "@mui/material";
import { DataGrid } from "@mui/x-data-grid";
import WarningAmberIcon from "@mui/icons-material/WarningAmber";
import ImageIcon from "@mui/icons-material/Image";
import { adminApi } from "../api";

const STATUS_COLORS = {
  open:         "error",
  under_review: "warning",
  resolved:     "success",
  closed:       "default",
};

const VERDICT_COLORS = {
  favor_buyer:   "error",
  favor_seller:  "success",
  inconclusive:  "warning",
};

const VERDICT_LABELS = {
  favor_buyer:   "Favor Comprador",
  favor_seller:  "Favor Vendedor",
  inconclusive:  "Inconclusivo",
};

const RESOLUTION_OPTS = [
  { value: "refund_buyer",      label: "Reembolsar Comprador" },
  { value: "release_to_seller", label: "Liberar ao Vendedor" },
  { value: "split",             label: "Dividir" },
  { value: "manual",            label: "Manual (admin externo)" },
];

export default function Disputes() {
  const [resolveDialog, setResolveDialog] = useState(null);
  const [evidenceDialog, setEvidenceDialog] = useState(null);
  const [resolution, setResolution] = useState("refund_buyer");
  const [splitPct, setSplitPct] = useState(50);
  const [notes, setNotes] = useState("");
  const qc = useQueryClient();

  const { data: disputes = [], isLoading, error } = useQuery({
    queryKey: ["disputes"],
    queryFn: adminApi.listDisputes,
    refetchInterval: 15_000,
  });

  const resolveMutation = useMutation({
    mutationFn: ({ id }) =>
      adminApi.resolveDispute(id, {
        resolution,
        split_pct: resolution === "split" ? splitPct : undefined,
        notes: notes || null,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["disputes"] });
      qc.invalidateQueries({ queryKey: ["dashboard"] });
      setResolveDialog(null);
      setNotes("");
    },
  });

  const columns = [
    {
      field: "id",
      headerName: "ID",
      width: 100,
      renderCell: (p) => (
        <Tooltip title={p.value}>
          <span style={{ fontFamily: "monospace", fontSize: 12 }}>
            {String(p.value).slice(0, 8)}…
          </span>
        </Tooltip>
      ),
    },
    {
      field: "order_id",
      headerName: "Pedido",
      width: 100,
      renderCell: (p) => p.value ? (
        <Tooltip title={p.value}>
          <span style={{ fontFamily: "monospace", fontSize: 12 }}>
            {String(p.value).slice(0, 8)}…
          </span>
        </Tooltip>
      ) : "—",
    },
    {
      field: "status",
      headerName: "Status",
      width: 130,
      renderCell: (p) => (
        <Chip
          label={p.value}
          color={STATUS_COLORS[p.value?.toLowerCase()] ?? "default"}
          size="small"
        />
      ),
    },
    {
      field: "ai_verdict",
      headerName: "IA",
      width: 150,
      renderCell: (p) => p.value ? (
        <Chip
          label={VERDICT_LABELS[p.value] ?? p.value}
          color={VERDICT_COLORS[p.value] ?? "default"}
          size="small"
          variant="outlined"
        />
      ) : (
        <Chip label="Pendente" color="default" size="small" variant="outlined" />
      ),
    },
    {
      field: "ai_confidence",
      headerName: "Confiança",
      width: 90,
      renderCell: (p) => p.value != null
        ? `${Math.round(p.value * 100)}%`
        : "—",
    },
    {
      field: "high_risk_buyer",
      headerName: "Risco",
      width: 80,
      renderCell: (p) => p.value ? (
        <Tooltip title="Comprador com score < 200 — revisão obrigatória">
          <Chip
            icon={<WarningAmberIcon fontSize="small" />}
            label="ALTO"
            color="error"
            size="small"
          />
        </Tooltip>
      ) : null,
    },
    { field: "reason", headerName: "Motivo", flex: 1, minWidth: 140 },
    {
      field: "opened_at",
      headerName: "Aberta em",
      width: 160,
      valueFormatter: (v) => v ? new Date(v).toLocaleString("pt-BR") : "—",
    },
    {
      field: "deadline_at",
      headerName: "Prazo",
      width: 160,
      renderCell: (p) => {
        if (!p.value) return "—";
        const dl = new Date(p.value);
        const past = dl < new Date();
        return (
          <span style={{ color: past ? "#d32f2f" : "inherit", fontSize: 12 }}>
            {dl.toLocaleString("pt-BR")}
          </span>
        );
      },
    },
    {
      field: "evidence_actions",
      headerName: "Evidências",
      width: 120,
      renderCell: (p) => (
        <Button
          size="small"
          startIcon={<ImageIcon />}
          onClick={() => setEvidenceDialog(p.row)}
        >
          Ver
        </Button>
      ),
    },
    {
      field: "actions",
      headerName: "Resolução",
      width: 120,
      renderCell: (p) =>
        ["open", "under_review"].includes(p.row.status?.toLowerCase()) ? (
          <Button
            size="small"
            variant="outlined"
            color="primary"
            onClick={() => setResolveDialog(p.row)}
          >
            Resolver
          </Button>
        ) : null,
    },
  ];

  const rows = disputes.map((d, i) => ({
    id: d.id ?? i,
    evidence_actions: null,
    ...d,
  }));

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
        sx={{
          "& .MuiDataGrid-row:hover": { backgroundColor: "rgba(0,0,0,0.04)" },
        }}
      />

      {/* ── Dialog: Evidências + IA ─────────────────────────────────────────── */}
      <Dialog
        open={!!evidenceDialog}
        onClose={() => setEvidenceDialog(null)}
        maxWidth="lg"
        fullWidth
      >
        <DialogTitle>
          Evidências — disputa {evidenceDialog?.id?.slice(0, 8)}
          {evidenceDialog?.high_risk_buyer && (
            <Chip
              icon={<WarningAmberIcon />}
              label="COMPRADOR ALTO RISCO"
              color="error"
              size="small"
              sx={{ ml: 2 }}
            />
          )}
        </DialogTitle>
        <DialogContent dividers>
          {evidenceDialog?.ai_verdict && (
            <Paper variant="outlined" sx={{ p: 2, mb: 2, bgcolor: "grey.50" }}>
              <Stack direction="row" alignItems="center" spacing={1} mb={1}>
                <Typography fontWeight={600}>Análise IA:</Typography>
                <Chip
                  label={VERDICT_LABELS[evidenceDialog.ai_verdict] ?? evidenceDialog.ai_verdict}
                  color={VERDICT_COLORS[evidenceDialog.ai_verdict] ?? "default"}
                  size="small"
                />
                {evidenceDialog.ai_confidence != null && (
                  <Typography variant="body2" color="text.secondary">
                    Confiança: {Math.round(evidenceDialog.ai_confidence * 100)}%
                  </Typography>
                )}
              </Stack>
              {evidenceDialog.ai_reasoning && (
                <Typography variant="body2" color="text.secondary">
                  {evidenceDialog.ai_reasoning}
                </Typography>
              )}
            </Paper>
          )}

          <Grid container spacing={2}>
            {/* Fotos do comprador */}
            <Grid item xs={12} md={6}>
              <Typography fontWeight={600} mb={1}>Evidências do Comprador</Typography>
              <EvidenceList
                evidence={(evidenceDialog?.evidence ?? []).filter(
                  (e) => e.party === "buyer"
                )}
              />
            </Grid>
            {/* Contra-evidências do vendedor */}
            <Grid item xs={12} md={6}>
              <Typography fontWeight={600} mb={1}>Evidências do Vendedor</Typography>
              <EvidenceList
                evidence={(evidenceDialog?.evidence ?? []).filter(
                  (e) => e.party === "seller"
                )}
              />
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEvidenceDialog(null)}>Fechar</Button>
          {["open", "under_review"].includes(
            evidenceDialog?.status?.toLowerCase()
          ) && (
            <Button
              variant="contained"
              onClick={() => {
                setResolveDialog(evidenceDialog);
                setEvidenceDialog(null);
              }}
            >
              Resolver Disputa
            </Button>
          )}
        </DialogActions>
      </Dialog>

      {/* ── Dialog: Resolução ──────────────────────────────────────────────── */}
      <Dialog
        open={!!resolveDialog}
        onClose={() => setResolveDialog(null)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Resolver Disputa</DialogTitle>
        <DialogContent>
          <Stack spacing={2} mt={1}>
            {resolveDialog?.ai_verdict && (
              <Alert
                severity={
                  resolveDialog.ai_verdict === "favor_buyer"
                    ? "warning"
                    : resolveDialog.ai_verdict === "favor_seller"
                    ? "success"
                    : "info"
                }
              >
                IA sugere: <strong>{VERDICT_LABELS[resolveDialog.ai_verdict]}</strong>
                {resolveDialog.ai_confidence != null &&
                  ` (confiança ${Math.round(resolveDialog.ai_confidence * 100)}%)`}
              </Alert>
            )}
            <TextField
              select
              label="Resolução"
              value={resolution}
              onChange={(e) => setResolution(e.target.value)}
              fullWidth
            >
              {RESOLUTION_OPTS.map((r) => (
                <MenuItem key={r.value} value={r.value}>{r.label}</MenuItem>
              ))}
            </TextField>
            {resolution === "split" && (
              <Box>
                <Typography gutterBottom>
                  % para o comprador: <strong>{splitPct}%</strong> (vendedor: {100 - splitPct}%)
                </Typography>
                <Slider
                  value={splitPct}
                  onChange={(_, v) => setSplitPct(v)}
                  min={10} max={90} step={5}
                  marks
                  valueLabelDisplay="auto"
                />
              </Box>
            )}
            <TextField
              label="Notas (visível para o time)"
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

function EvidenceList({ evidence }) {
  if (!evidence.length) {
    return (
      <Typography variant="body2" color="text.secondary">
        Nenhuma evidência enviada.
      </Typography>
    );
  }
  return (
    <Stack spacing={1}>
      {evidence.map((ev) => (
        <Paper
          key={ev.id}
          variant="outlined"
          sx={{
            p: 1.5,
            borderColor: ev.ai_flagged ? "error.main" : undefined,
            bgcolor: ev.ai_flagged ? "error.50" : undefined,
          }}
        >
          <Stack direction="row" spacing={1} alignItems="center">
            <Chip label={ev.kind} size="small" variant="outlined" />
            {ev.ai_flagged && (
              <Chip
                icon={<WarningAmberIcon fontSize="small" />}
                label="Suspeito"
                color="error"
                size="small"
              />
            )}
          </Stack>
          {ev.minio_url && (
            <Box mt={1}>
              {ev.kind === "photo" || ev.kind === "video" ? (
                <a href={ev.minio_url} target="_blank" rel="noopener noreferrer">
                  <img
                    src={ev.minio_url}
                    alt="evidência"
                    style={{
                      maxWidth: "100%",
                      maxHeight: 180,
                      borderRadius: 4,
                      cursor: "pointer",
                    }}
                    onError={(e) => { e.target.style.display = "none"; }}
                  />
                </a>
              ) : (
                <a href={ev.minio_url} target="_blank" rel="noopener noreferrer">
                  {ev.minio_url}
                </a>
              )}
            </Box>
          )}
          {ev.content && (
            <Typography variant="body2" mt={0.5} sx={{ wordBreak: "break-all" }}>
              {ev.content}
            </Typography>
          )}
          <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
            SHA-256: {ev.sha256?.slice(0, 16)}…
          </Typography>
        </Paper>
      ))}
    </Stack>
  );
}
