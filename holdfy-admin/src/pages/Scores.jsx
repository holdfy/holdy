import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Chip, Slider, Stack, ToggleButton, ToggleButtonGroup,
  Button, Dialog, DialogTitle, DialogContent, DialogActions, Paper,
} from "@mui/material";
import { DataGrid } from "@mui/x-data-grid";
import ListAltIcon from "@mui/icons-material/ListAlt";
import { adminApi } from "../api";

const RISK_COLORS = { Low: "success", Medium: "warning", High: "error", Critical: "error" };

const DECISION_COLORS = { approve: "success", review: "warning", block: "error" };
const DECISION_LABELS = { approve: "Aprovado", review: "Revisão", block: "Bloqueado" };

const FACTOR_LABELS = {
  sefaz_status: "Status do documento (Receita/Sefaz)",
  social_account_age: "Idade da rede social",
  dispute_history: "Disputas abertas pelo usuário",
  counterparty_disputes: "Disputas abertas contra o usuário",
  dispute_rate: "Taxa de disputas",
  velocity_check: "Velocidade de transações",
  high_volume: "Volume alto",
  structuring: "Fracionamento (estruturação)",
  account_maturity: "Maturidade da conta",
  value_anomaly: "Anomalia de valor",
  cnpj_status: "Status do CNPJ",
  company_age: "Idade da empresa",
  other: "Outro",
};

function factorDetail(f) {
  switch (f.kind) {
    case "social_account_age": return `${f.platform} · ${f.months} meses`;
    case "dispute_history": return `${f.open_disputes} disputa(s) aberta(s)`;
    case "counterparty_disputes": return `${f.count} disputa(s) contra o usuário`;
    case "dispute_rate": return `${f.rate_pct}% de taxa de disputas`;
    case "velocity_check": return `${f.tx_count} transação(ões) em ${f.window_hours}h`;
    case "high_volume": return `R$ ${f.amount_brl} em ${f.window_hours}h`;
    case "structuring": return `R$ ${f.amount_brl} — próximo de limite de reporte COAF`;
    case "account_maturity": return `${f.tx_count} transação(ões) · conta com ${f.age_days} dia(s)`;
    case "value_anomaly": return `${f.ratio_pct}% da média histórica do usuário`;
    case "cnpj_status": return f.active ? "CNPJ ativo" : "CNPJ inativo/suspenso";
    case "company_age": return `${f.months} meses de empresa`;
    case "other": return f.code;
    default: return null;
  }
}

const columns = [
  { field: "user_id", headerName: "User ID", width: 280, renderCell: (p) => String(p.value).slice(0, 8) + "…" },
  {
    field: "person_type",
    headerName: "Tipo",
    width: 90,
    renderCell: (p) => {
      const v = p.value;
      if (!v) return <Chip label="PF" size="small" />;
      const isPJ = v === "legal" || v === "pj" || v === 2 || v === "2";
      return <Chip label={isPJ ? "PJ" : "PF"} color={isPJ ? "primary" : "default"} size="small" />;
    },
  },
  {
    field: "score",
    headerName: "Score",
    width: 120,
    type: "number",
    renderCell: (p) => (
      <Box sx={{ fontWeight: 700, color: p.value >= 650 ? "success.main" : p.value >= 350 ? "warning.main" : "error.main" }}>
        {p.value}
      </Box>
    ),
  },
  {
    field: "risk_level",
    headerName: "Risco",
    width: 120,
    renderCell: (p) => <Chip label={p.value} color={RISK_COLORS[p.value] ?? "default"} size="small" />,
  },
  {
    field: "decision",
    headerName: "Decisão",
    width: 120,
    renderCell: (p) => (
      <Chip
        label={DECISION_LABELS[p.value] ?? p.value ?? "—"}
        color={DECISION_COLORS[p.value] ?? "default"}
        size="small"
      />
    ),
  },
  {
    field: "factors_action",
    headerName: "Fatores",
    width: 130,
    sortable: false,
    renderCell: (p) => (
      <Button
        size="small"
        startIcon={<ListAltIcon />}
        onClick={() => p.row.__openFactors(p.row)}
        disabled={!p.row.factors?.length}
      >
        Ver ({p.row.factors?.length ?? 0})
      </Button>
    ),
  },
];

function matchesPersonFilter(user, filter) {
  if (filter === "all") return true;
  const isPJ =
    user.person_type === "legal" ||
    user.person_type === "pj" ||
    user.person_type === 2 ||
    user.person_type === "2";
  return filter === "pj" ? isPJ : !isPJ;
}

export default function Scores() {
  const [maxScore, setMaxScore] = useState(1000);
  const [personFilter, setPersonFilter] = useState("all");
  const [factorsDialog, setFactorsDialog] = useState(null);

  const { data, isLoading, error } = useQuery({
    queryKey: ["scores"],
    queryFn: adminApi.listScores,
    refetchInterval: 60_000,
  });

  const users = (data?.users ?? [])
    .filter((u) => u.score <= maxScore && matchesPersonFilter(u, personFilter));
  const rows = users.map((u, i) => ({ id: i, ...u, __openFactors: setFactorsDialog }));

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Usuários / Score Antifraude
      </Typography>
      <Stack direction="row" alignItems="center" spacing={3} mb={3} flexWrap="wrap" rowGap={2}>
        <Stack direction="row" alignItems="center" spacing={2} sx={{ minWidth: 300 }}>
          <Typography variant="body2" color="text.secondary" whiteSpace="nowrap">
            Score máximo:
          </Typography>
          <Slider
            value={maxScore}
            min={0}
            max={1000}
            step={50}
            onChange={(_, v) => setMaxScore(v)}
            valueLabelDisplay="auto"
            sx={{ width: 200 }}
          />
          <Typography variant="body2" fontWeight={700} whiteSpace="nowrap">
            {maxScore}
          </Typography>
        </Stack>
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
      <Typography variant="body2" color="text.secondary" mt={1}>
        {rows.length} usuário(s) com score ≤ {maxScore}
        {personFilter !== "all" ? ` (${personFilter.toUpperCase()})` : ""}
      </Typography>

      {/* ── Dialog: fatores do score — dado sensível, só admin vê o breakdown ── */}
      <Dialog open={!!factorsDialog} onClose={() => setFactorsDialog(null)} maxWidth="sm" fullWidth>
        <DialogTitle>
          Fatores do score — {factorsDialog?.user_id?.slice(0, 8)}…
          {factorsDialog?.decision && (
            <Chip
              label={DECISION_LABELS[factorsDialog.decision] ?? factorsDialog.decision}
              color={DECISION_COLORS[factorsDialog.decision] ?? "default"}
              size="small"
              sx={{ ml: 2 }}
            />
          )}
        </DialogTitle>
        <DialogContent dividers>
          <Typography variant="body2" color="text.secondary" mb={2}>
            Score final: <strong>{factorsDialog?.score}</strong> / 1000 — soma de todos os fatores abaixo.
          </Typography>
          <Stack spacing={1}>
            {(factorsDialog?.factors ?? []).map((f, i) => (
              <Paper key={i} variant="outlined" sx={{ p: 1.5 }}>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Box>
                    <Typography variant="body2" fontWeight={600}>
                      {FACTOR_LABELS[f.kind] ?? f.kind}
                    </Typography>
                    {factorDetail(f) && (
                      <Typography variant="caption" color="text.secondary">
                        {factorDetail(f)}
                      </Typography>
                    )}
                  </Box>
                  <Typography
                    variant="body2"
                    fontWeight={700}
                    color={f.weight >= 0 ? "success.main" : "error.main"}
                  >
                    {f.weight >= 0 ? "+" : ""}{f.weight}
                  </Typography>
                </Stack>
              </Paper>
            ))}
            {!(factorsDialog?.factors ?? []).length && (
              <Typography variant="body2" color="text.secondary">
                Sem fatores registrados.
              </Typography>
            )}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setFactorsDialog(null)}>Fechar</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
