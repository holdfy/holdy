/**
 * Relatório de lucro (receita da plataforma - TTO+TPO)
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import Card from "@mui/material/Card";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportProfit() {
  const [profit, setProfit] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    adminApi.reports.profit()
      .then((r) => setProfit(r?.profit ?? r?.data ?? 0))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Lucro</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <Card sx={{ p: 4, maxWidth: 400 }}>
            <MDTypography variant="button" color="text">Receita da plataforma (TTO + TPO)</MDTypography>
            <MDTypography variant="h3" color="success" fontWeight="bold" mt={1}>
              {formatCurrency(profit)}
            </MDTypography>
            <MDTypography variant="caption" color="text" display="block" mt={2}>
              Soma das taxas operacionais (TTO) e taxas de parceiros (TPO) creditadas na conta admin.
            </MDTypography>
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
