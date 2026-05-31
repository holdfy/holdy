/**
 * Dashboard Admin - métricas consolidadas e atalhos
 */

import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import Grid from "@mui/material/Grid";
import Icon from "@mui/material/Icon";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { useAuth } from "context/AuthContext";
import { adminApi, entityApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function AdminDashboard() {
  const { username } = useAuth();
  const navigate = useNavigate();
  const [metrics, setMetrics] = useState({
    customers: 0,
    accounts: 0,
    activeAccounts: 0,
    transactionsToday: 0,
    medTotal: 0,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const today = new Date().toISOString().slice(0, 10);
        const [custRes, accRes, txRes, medRes] = await Promise.allSettled([
          adminApi.customers.list(),
          entityApi.accounts.list({ limit: 500, offset: 0 }),
          adminApi.pix.transactions({ limit: 500, page: 1 }),
          entityApi.secMed.list({ limit: 500, offset: 0 }),
        ]);
        if (!cancelled) {
          const customers = custRes.status === "fulfilled"
            ? (Array.isArray(custRes.value) ? custRes.value : custRes.value?.items || [])
            : [];
          const accounts = accRes.status === "fulfilled"
            ? (Array.isArray(accRes.value) ? accRes.value : accRes.value?.items || accRes.value?.data || [])
            : [];
          const txs = txRes.status === "fulfilled"
            ? (txRes.value?.data ?? txRes.value?.items ?? txRes.value ?? [])
            : [];
          const meds = medRes.status === "fulfilled"
            ? (medRes.value?.items ?? medRes.value ?? [])
            : [];
          const medTotal = (Array.isArray(meds) ? meds : []).reduce((s, m) => s + (Number(m.amount) || 0), 0);
          const txToday = (Array.isArray(txs) ? txs : []).filter((t) => (t.created_at || "").slice(0, 10) === today);
          const active = (Array.isArray(accounts) ? accounts : []).filter((a) => a.account_status_id === 1 || a.status === "ACTIVE").length;
          setMetrics({
            customers: customers.length,
            accounts: accounts.length,
            activeAccounts: active || accounts.length,
            transactionsToday: txToday.length,
            medTotal,
          });
        }
      } catch (e) {
        if (!cancelled) setError(e.message);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>
          Admin - {username || "Operador"}
        </MDTypography>

        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDTypography variant="button" color="text" mb={2}>Métricas consolidadas</MDTypography>
            <Grid container spacing={3} mb={3}>
              <Grid item xs={12} sm={6} md={4} lg={2}>
                <Card sx={{ cursor: "pointer" }} onClick={() => navigate("/admin/customers")}>
                  <MDBox p={2}>
                    <MDTypography variant="caption" color="text">Clientes</MDTypography>
                    <MDTypography variant="h4" color="info">{metrics.customers}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} sm={6} md={4} lg={2}>
                <Card sx={{ cursor: "pointer" }} onClick={() => navigate("/admin/reports/balances")}>
                  <MDBox p={2}>
                    <MDTypography variant="caption" color="text">Contas ativas</MDTypography>
                    <MDTypography variant="h4" color="success">{metrics.activeAccounts}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} sm={6} md={4} lg={2}>
                <Card sx={{ cursor: "pointer" }} onClick={() => navigate("/admin/pix/transactions")}>
                  <MDBox p={2}>
                    <MDTypography variant="caption" color="text">Transações hoje</MDTypography>
                    <MDTypography variant="h4" color="primary">{metrics.transactionsToday}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} sm={6} md={4} lg={2}>
                <Card sx={{ cursor: "pointer" }} onClick={() => navigate("/admin/sec-med")}>
                  <MDBox p={2}>
                    <MDTypography variant="caption" color="text">MED bloqueado</MDTypography>
                    <MDTypography variant="h6" color="warning">{formatCurrency(metrics.medTotal)}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
            </Grid>

            <MDTypography variant="button" color="text" mb={2}>Atalhos</MDTypography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="gradient" color="info" fullWidth onClick={() => navigate("/admin/customers")}>
                  <Icon sx={{ mr: 1 }}>people</Icon> Clientes
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="outlined" color="info" fullWidth onClick={() => navigate("/admin/pix/transactions")}>
                  <Icon sx={{ mr: 1 }}>payment</Icon> Transações PIX
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="outlined" color="dark" fullWidth onClick={() => navigate("/admin/partners")}>
                  <Icon sx={{ mr: 1 }}>handshake</Icon> Parceiros
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="outlined" color="info" fullWidth onClick={() => navigate("/admin/reports/users-accounts")}>
                  <Icon sx={{ mr: 1 }}>assessment</Icon> Relatório Usuários
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="outlined" color="success" fullWidth onClick={() => navigate("/admin/reports/balances")}>
                  <Icon sx={{ mr: 1 }}>account_balance</Icon> Relatório Saldos
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="outlined" color="success" fullWidth onClick={() => navigate("/admin/reports/reconciliation")}>
                  <Icon sx={{ mr: 1 }}>compare_arrows</Icon> Conciliação
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton variant="outlined" color="success" fullWidth onClick={() => navigate("/admin/reports/profit")}>
                  <Icon sx={{ mr: 1 }}>trending_up</Icon> Relatório Lucro
                </MDButton>
              </Grid>
            </Grid>
          </>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
