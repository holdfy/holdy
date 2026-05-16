/**
 * Admin - Detalhe do MED
 */

import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import CircularProgress from "@mui/material/CircularProgress";
import Icon from "@mui/material/Icon";
import Chip from "@mui/material/Chip";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import BlockchainProof from "components/BlockchainProof";
import { entityApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

const STATUS_LABELS = { 1: "OPEN", 2: "RETURNED" };

export default function AdminSecMedDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [med, setMed] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    entityApi.secMed.get(id)
      .then((r) => setMed(Array.isArray(r) ? r[0] : r))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) {
    return (
      <DashboardLayout>
        <DashboardNavbar />
        <MDBox py={3} display="flex" justifyContent="center"><CircularProgress /></MDBox>
        <Footer />
      </DashboardLayout>
    );
  }

  if (error || !med) {
    return (
      <DashboardLayout>
        <DashboardNavbar />
        <MDBox py={3}>
          <MDTypography color="error">{error || "MED não encontrado"}</MDTypography>
          <MDButton variant="text" color="info" onClick={() => navigate("/admin/sec-med")} sx={{ mt: 2 }}>
            Voltar
          </MDButton>
        </MDBox>
        <Footer />
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" alignItems="center" gap={2} mb={3}>
          <MDButton variant="outlined" color="dark" size="small" onClick={() => navigate("/admin/sec-med")}>
            <Icon sx={{ mr: 0.5 }}>arrow_back</Icon> Voltar
          </MDButton>
          <MDTypography variant="h4" fontWeight="medium">MED #{med.id}</MDTypography>
          <Chip label={STATUS_LABELS[med.status_sec_med_id] ?? med.status_sec_med_id} size="small" color={med.status_sec_med_id === 1 ? "warning" : "success"} />
        </MDBox>

        <Card sx={{ maxWidth: 600 }}>
          <MDBox p={3}>
            <MDBox display="grid" gridTemplateColumns="1fr 1fr" gap={2} mb={2}>
              <MDBox>
                <MDTypography variant="button" color="text">Valor</MDTypography>
                <MDTypography variant="h6">{formatCurrency(med.amount)}</MDTypography>
              </MDBox>
              <MDBox>
                <MDTypography variant="button" color="text">Conta</MDTypography>
                <MDTypography variant="body1">{med.account_id}</MDTypography>
              </MDBox>
              <MDBox>
                <MDTypography variant="button" color="text">Data</MDTypography>
                <MDTypography variant="body1">{med.created_at?.slice?.(0, 16) || "-"}</MDTypography>
              </MDBox>
              <MDBox>
                <MDTypography variant="button" color="text">Liberação</MDTypography>
                <MDTypography variant="body1">{med.scheduled_date?.slice?.(0, 10) || "-"}</MDTypography>
              </MDBox>
            </MDBox>

            <MDBox mt={3} p={2} bgcolor="grey.100" borderRadius={1}>
              <MDTypography variant="button" fontWeight="medium" mb={1}>Prova blockchain</MDTypography>
              <BlockchainProof entityType="med" entityId={String(med.id)} />
            </MDBox>
          </MDBox>
        </Card>
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
