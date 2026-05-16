/**
 * Conta - Limites
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import Card from "@mui/material/Card";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { customersApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function CustomerLimits() {
  const [limits, setLimits] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    customersApi.account.limits()
      .then((r) => setLimits(r?.limits ?? r))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Limites</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <Card sx={{ maxWidth: 500 }}>
            <MDBox p={3}>
              {limits ? (
                <>
                  {limits.max_transaction != null && (
                    <MDBox mb={2}>
                      <MDTypography variant="button" color="text">Limite máximo por transação</MDTypography>
                      <MDTypography variant="h6">{formatCurrency(limits.max_transaction)}</MDTypography>
                    </MDBox>
                  )}
                  {limits.min_transaction != null && (
                    <MDBox mb={2}>
                      <MDTypography variant="button" color="text">Limite mínimo por transação</MDTypography>
                      <MDTypography variant="h6">{formatCurrency(limits.min_transaction)}</MDTypography>
                    </MDBox>
                  )}
                  {limits.daily_limit != null && (
                    <MDBox mb={2}>
                      <MDTypography variant="button" color="text">Limite diário</MDTypography>
                      <MDTypography variant="h6">{formatCurrency(limits.daily_limit)}</MDTypography>
                    </MDBox>
                  )}
                  {!limits.max_transaction && !limits.min_transaction && !limits.daily_limit && (
                    <MDTypography color="text">Nenhum limite configurado.</MDTypography>
                  )}
                </>
              ) : (
                <MDTypography color="text">Nenhum limite configurado.</MDTypography>
              )}
            </MDBox>
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
