/**
 * Conta - Saldo (balance, MED, disponível)
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import Card from "@mui/material/Card";
import Grid from "@mui/material/Grid";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { customersApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function CustomerBalance() {
  const [balance, setBalance] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    customersApi.account.balance(true)
      .then(setBalance)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>
          Saldo
        </MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <Grid container spacing={3}>
            <Grid item xs={12} md={4}>
              <Card><MDBox p={3}>
                <MDTypography variant="button" color="text">Saldo disponível</MDTypography>
                <MDTypography variant="h4" color="info">{formatCurrency(balance?.availableBalance ?? 0)}</MDTypography>
              </MDBox></Card>
            </Grid>
            <Grid item xs={12} md={4}>
              <Card><MDBox p={3}>
                <MDTypography variant="button" color="text">MED bloqueado</MDTypography>
                <MDTypography variant="h4" color="warning">{formatCurrency(balance?.preventiveBlock ?? 0)}</MDTypography>
              </MDBox></Card>
            </Grid>
            <Grid item xs={12} md={4}>
              <Card><MDBox p={3}>
                <MDTypography variant="button" color="text">Saldo bruto</MDTypography>
                <MDTypography variant="h4" color="success">{formatCurrency(balance?.balance ?? 0)}</MDTypography>
              </MDBox></Card>
            </Grid>
          </Grid>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
