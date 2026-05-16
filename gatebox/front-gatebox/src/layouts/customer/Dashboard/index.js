/**
 * Dashboard Customer - resumo de saldo e últimas transações
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
import { customersApi } from "services/api";

function formatCurrency(value) {
  return new Intl.NumberFormat("pt-BR", {
    style: "currency",
    currency: "BRL",
  }).format(Number(value) || 0);
}

export default function CustomerDashboard() {
  const { username } = useAuth();
  const navigate = useNavigate();
  const [balance, setBalance] = useState(null);
  const [transactions, setTransactions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [bal, tx] = await Promise.all([
          customersApi.account.balance(true),
          customersApi.pix.transactions({ limit: 10 }),
        ]);
        if (!cancelled) {
          setBalance(bal);
          setTransactions(Array.isArray(tx) ? tx : tx.items || tx.data || []);
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
        <MDBox mb={3} display="flex" justifyContent="space-between" alignItems="center">
          <MDTypography variant="h4" fontWeight="medium">
            Olá, {username || "Cliente"}
          </MDTypography>
        </MDBox>

        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}>
            <CircularProgress />
          </MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <Grid container spacing={3} mb={3}>
              <Grid item xs={12} md={4}>
                <Card>
                  <MDBox p={2}>
                    <MDTypography variant="button" color="text" fontWeight="medium">
                      Saldo disponível
                    </MDTypography>
                    <MDTypography variant="h4" color="info" fontWeight="bold">
                      {formatCurrency(balance?.availableBalance ?? balance?.balance ?? 0)}
                    </MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} md={4}>
                <Card>
                  <MDBox p={2}>
                    <MDTypography variant="button" color="text" fontWeight="medium">
                      MED bloqueado
                    </MDTypography>
                    <MDTypography variant="h4" color="warning" fontWeight="bold">
                      {formatCurrency(balance?.preventiveBlock ?? 0)}
                    </MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} md={4}>
                <Card>
                  <MDBox p={2}>
                    <MDTypography variant="button" color="text" fontWeight="medium">
                      Saldo bruto
                    </MDTypography>
                    <MDTypography variant="h4" color="success" fontWeight="bold">
                      {formatCurrency(balance?.balance ?? 0)}
                    </MDTypography>
                  </MDBox>
                </Card>
              </Grid>
            </Grid>

            <Grid container spacing={2}>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton
                  variant="gradient"
                  color="info"
                  fullWidth
                  onClick={() => navigate("/customer/pix/send")}
                >
                  <Icon sx={{ mr: 1 }}>send</Icon>
                  Enviar PIX
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton
                  variant="outlined"
                  color="info"
                  fullWidth
                  onClick={() => navigate("/customer/pix/qrcode")}
                >
                  <Icon sx={{ mr: 1 }}>qr_code</Icon>
                  Gerar QR Code
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton
                  variant="outlined"
                  color="dark"
                  fullWidth
                  onClick={() => navigate("/customer/keys")}
                >
                  <Icon sx={{ mr: 1 }}>vpn_key</Icon>
                  Chaves PIX
                </MDButton>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <MDButton
                  variant="outlined"
                  color="dark"
                  fullWidth
                  onClick={() => navigate("/customer/extract")}
                >
                  <Icon sx={{ mr: 1 }}>list_alt</Icon>
                  Extrato
                </MDButton>
              </Grid>
            </Grid>

            <Card sx={{ mt: 3 }}>
              <MDBox p={2}>
                <MDTypography variant="h6" fontWeight="medium" mb={2}>
                  Últimas transações
                </MDTypography>
                {transactions.length === 0 ? (
                  <MDTypography variant="body2" color="text">
                    Nenhuma transação recente.
                  </MDTypography>
                ) : (
                  <MDBox component="ul" sx={{ listStyle: "none", p: 0, m: 0 }}>
                    {transactions.slice(0, 5).map((t, i) => (
                      <MDBox
                        key={t.id || i}
                        component="li"
                        display="flex"
                        justifyContent="space-between"
                        py={1}
                        borderBottom="1px solid"
                        borderColor="divider"
                      >
                        <MDTypography variant="body2">
                          {t.description || t.type || "PIX"} - {t.created_at?.slice?.(0, 10) || ""}
                        </MDTypography>
                        <MDTypography
                          variant="body2"
                          fontWeight="medium"
                          color={Number(t.amount) >= 0 ? "success" : "error"}
                        >
                          {formatCurrency(t.amount)}
                        </MDTypography>
                      </MDBox>
                    ))}
                  </MDBox>
                )}
                <MDButton
                  variant="text"
                  color="info"
                  size="small"
                  onClick={() => navigate("/customer/pix/transactions")}
                  sx={{ mt: 1 }}
                >
                  Ver todas
                </MDButton>
              </MDBox>
            </Card>
          </>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
