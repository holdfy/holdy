import { useQuery } from "@tanstack/react-query";
import { Grid, Typography, Alert, Skeleton, Box } from "@mui/material";
import AttachMoneyIcon from "@mui/icons-material/AttachMoney";
import TrendingUpIcon from "@mui/icons-material/TrendingUp";
import GavelIcon from "@mui/icons-material/Gavel";
import LockIcon from "@mui/icons-material/Lock";
import { adminApi } from "../api";
import StatCard from "../components/StatCard";

function fmtBrl(minorStr) {
  const val = parseFloat(minorStr ?? "0") / 100;
  return val.toLocaleString("pt-BR", { style: "currency", currency: "BRL" });
}

export default function Dashboard() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["dashboard"],
    queryFn: adminApi.dashboard,
    refetchInterval: 30_000,
  });

  if (isLoading) {
    return (
      <Grid container spacing={3}>
        {[...Array(4)].map((_, i) => (
          <Grid item xs={12} sm={6} md={3} key={i}>
            <Skeleton variant="rounded" height={120} />
          </Grid>
        ))}
      </Grid>
    );
  }

  if (error) {
    return <Alert severity="error">Erro ao carregar dashboard: {error.message}</Alert>;
  }

  const stats = [
    {
      title: "Volume Total",
      value: fmtBrl(data?.total_volume_minor),
      subtitle: "Soma de custódias ativas",
      icon: <AttachMoneyIcon />,
      color: "primary.main",
    },
    {
      title: "Yield Acumulado",
      value: fmtBrl(data?.total_yield_accrued_minor),
      subtitle: "Rendimento distribuído",
      icon: <TrendingUpIcon />,
      color: "success.main",
    },
    {
      title: "Disputas Abertas",
      value: data?.open_disputes ?? 0,
      subtitle: "Em revisão ou abertas",
      icon: <GavelIcon />,
      color: data?.open_disputes > 0 ? "error.main" : "success.main",
    },
    {
      title: "Custódias Travadas",
      value: data?.locked_custodies ?? 0,
      subtitle: "Locked + Disputadas",
      icon: <LockIcon />,
      color: "warning.main",
    },
  ];

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Dashboard
      </Typography>
      <Grid container spacing={3}>
        {stats.map((s) => (
          <Grid item xs={12} sm={6} md={3} key={s.title}>
            <StatCard {...s} />
          </Grid>
        ))}
      </Grid>
    </Box>
  );
}
