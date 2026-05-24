import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Box, Typography, Alert, Chip, Slider, Stack, ToggleButton, ToggleButtonGroup } from "@mui/material";
import { DataGrid } from "@mui/x-data-grid";
import { adminApi } from "../api";

const RISK_COLORS = { Low: "success", Medium: "warning", High: "error", Critical: "error" };

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

  const { data, isLoading, error } = useQuery({
    queryKey: ["scores"],
    queryFn: adminApi.listScores,
    refetchInterval: 60_000,
  });

  const users = (data?.users ?? [])
    .filter((u) => u.score <= maxScore && matchesPersonFilter(u, personFilter));
  const rows = users.map((u, i) => ({ id: i, ...u }));

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
    </Box>
  );
}
