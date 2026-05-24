import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Grid, TextField, Button, Stack,
} from "@mui/material";
import TrendingUpIcon from "@mui/icons-material/TrendingUp";
import InventoryIcon from "@mui/icons-material/Inventory";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import { adminApi } from "../api";
import StatCard from "../components/StatCard";

function fmtBrl(minorStr) {
  const val = parseFloat(minorStr ?? "0") / 100;
  return val.toLocaleString("pt-BR", { style: "currency", currency: "BRL" });
}

export default function YieldReport() {
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [params, setParams] = useState({});

  const { data, isLoading, error } = useQuery({
    queryKey: ["yield", params],
    queryFn: () => adminApi.yieldReport(params),
    refetchInterval: 60_000,
  });

  const apply = () => {
    setParams({
      ...(from ? { from: new Date(from).toISOString() } : {}),
      ...(to ? { to: new Date(to).toISOString() } : {}),
    });
  };

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Yield Report
      </Typography>
      <Stack direction="row" spacing={2} alignItems="flex-end" mb={3}>
        <TextField
          label="De"
          type="date"
          value={from}
          onChange={(e) => setFrom(e.target.value)}
          InputLabelProps={{ shrink: true }}
          size="small"
        />
        <TextField
          label="Até"
          type="date"
          value={to}
          onChange={(e) => setTo(e.target.value)}
          InputLabelProps={{ shrink: true }}
          size="small"
        />
        <Button variant="contained" onClick={apply} size="medium">
          Filtrar
        </Button>
        <Button variant="text" onClick={() => { setFrom(""); setTo(""); setParams({}); }}>
          Limpar
        </Button>
      </Stack>
      {error && <Alert severity="error" sx={{ mb: 2 }}>Erro: {error.message}</Alert>}
      <Grid container spacing={3}>
        <Grid item xs={12} sm={4}>
          <StatCard
            title="Yield Total"
            value={isLoading ? "…" : fmtBrl(data?.total_yield_minor)}
            subtitle="No período selecionado"
            icon={<TrendingUpIcon />}
            color="success.main"
          />
        </Grid>
        <Grid item xs={12} sm={4}>
          <StatCard
            title="Custódias"
            value={isLoading ? "…" : data?.custody_count ?? 0}
            subtitle="Total no período"
            icon={<InventoryIcon />}
            color="primary.main"
          />
        </Grid>
        <Grid item xs={12} sm={4}>
          <StatCard
            title="Liberadas"
            value={isLoading ? "…" : data?.released_count ?? 0}
            subtitle="Entrega confirmada"
            icon={<CheckCircleIcon />}
            color="info.main"
          />
        </Grid>
      </Grid>
    </Box>
  );
}
