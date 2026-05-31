/**
 * Admin - Detalhe da transação PIX
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
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function AdminPixTransactionDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { showSuccess, showError } = useSnackbar();
  const [tx, setTx] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [cancelling, setCancelling] = useState(false);

  useEffect(() => {
    adminApi.pix.get(id)
      .then(setTx)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  const handleCancel = async () => {
    if (!window.confirm("Cancelar esta transação?")) return;
    setCancelling(true);
    try {
      await adminApi.pix.cancel(id);
      showSuccess("Transação cancelada");
      setTx((t) => (t ? { ...t, status_transaction_id: 9 } : null));
    } catch (e) {
      showError(e.message);
    } finally {
      setCancelling(false);
    }
  };

  const canCancel = tx && [1, 2, 3, 11].includes(Number(tx.status_transaction_id));

  if (loading) {
    return (
      <DashboardLayout>
        <DashboardNavbar />
        <MDBox py={3} display="flex" justifyContent="center"><CircularProgress /></MDBox>
        <Footer />
      </DashboardLayout>
    );
  }

  if (error || !tx) {
    return (
      <DashboardLayout>
        <DashboardNavbar />
        <MDBox py={3}>
          <MDTypography color="error">{error || "Transação não encontrada"}</MDTypography>
          <MDButton variant="text" color="info" onClick={() => navigate("/admin/pix/transactions")} sx={{ mt: 2 }}>
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
          <MDButton variant="outlined" color="dark" size="small" onClick={() => navigate("/admin/pix/transactions")}>
            <Icon sx={{ mr: 0.5 }}>arrow_back</Icon> Voltar
          </MDButton>
          <MDTypography variant="h4" fontWeight="medium">Transação #{tx.id}</MDTypography>
          <Chip label={tx.status_transaction_id === 4 ? "Concluída" : tx.status_transaction_id === 9 ? "Cancelada" : "Em processamento"} color={tx.status_transaction_id === 4 ? "success" : tx.status_transaction_id === 9 ? "default" : "warning"} size="small" />
        </MDBox>

        <Card sx={{ maxWidth: 700 }}>
          <MDBox p={3}>
            <MDBox display="grid" gridTemplateColumns="1fr 1fr" gap={2} mb={2}>
              <MDBox>
                <MDTypography variant="button" color="text">Valor</MDTypography>
                <MDTypography variant="h6">{formatCurrency(tx.amount)}</MDTypography>
              </MDBox>
              <MDBox>
                <MDTypography variant="button" color="text">Data</MDTypography>
                <MDTypography variant="body1">{tx.created_at?.slice?.(0, 19) || "-"}</MDTypography>
              </MDBox>
              <MDBox>
                <MDTypography variant="button" color="text">Descrição</MDTypography>
                <MDTypography variant="body1">{tx.description || "-"}</MDTypography>
              </MDBox>
              <MDBox>
                <MDTypography variant="button" color="text">End-to-end</MDTypography>
                <MDTypography variant="caption" sx={{ wordBreak: "break-all" }}>{tx.endtoend_id || "-"}</MDTypography>
              </MDBox>
            </MDBox>

            {canCancel && (
              <MDButton variant="gradient" color="error" onClick={handleCancel} disabled={cancelling} sx={{ mt: 2 }}>
                {cancelling ? "Cancelando..." : "Cancelar transação"}
              </MDButton>
            )}
          </MDBox>
        </Card>
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
