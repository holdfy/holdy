/**
 * PIX - Status da Invoice (QR Code)
 * Consulta status: CREATED (1), DONE (2), CANCEL (3)
 */

import { useState } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import Chip from "@mui/material/Chip";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { entityApi } from "services/api";

const INVOICE_STATUS = { 1: "CREATED", 2: "DONE", 3: "CANCEL" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function CustomerPixInvoiceStatus() {
  const [invoiceId, setInvoiceId] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [invoice, setInvoice] = useState(null);

  const handleConsult = async (e) => {
    e.preventDefault();
    const id = invoiceId.trim();
    if (!id) {
      setError("Informe o ID da invoice.");
      return;
    }
    setError("");
    setInvoice(null);
    setLoading(true);
    try {
      const res = await entityApi.invoice.get(id);
      setInvoice(Array.isArray(res) ? res[0] : res);
    } catch (err) {
      setError(err.message || "Invoice não encontrada.");
    } finally {
      setLoading(false);
    }
  };

  const statusLabel = invoice ? (INVOICE_STATUS[invoice.invoice_status_id] ?? `#${invoice.invoice_status_id}`) : null;
  const statusColor = invoice?.invoice_status_id === 2 ? "success" : invoice?.invoice_status_id === 3 ? "error" : "info";

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Status da Invoice (QR Code)</MDTypography>
        <Card sx={{ maxWidth: 500 }}>
          <MDBox p={3} component="form" onSubmit={handleConsult}>
            {error && <MDTypography color="error" variant="caption" display="block" mb={2}>{error}</MDTypography>}
            <MDBox mb={2}>
              <MDInput
                label="ID da Invoice"
                fullWidth
                value={invoiceId}
                onChange={(e) => setInvoiceId(e.target.value)}
                placeholder="Ex: 123"
              />
            </MDBox>
            <MDButton variant="gradient" color="info" fullWidth type="submit" disabled={loading}>
              {loading ? "Consultando..." : "Consultar"}
            </MDButton>
          </MDBox>
        </Card>
        {invoice && (
          <Card sx={{ mt: 3, maxWidth: 500 }}>
            <MDBox p={3}>
              <MDTypography variant="h6" fontWeight="medium" mb={2}>Invoice #{invoice.id}</MDTypography>
              <MDBox mb={2}>
                <MDTypography variant="button" color="text">Status</MDTypography>
                <MDBox mt={0.5}>
                  <Chip label={statusLabel} color={statusColor} size="small" />
                </MDBox>
              </MDBox>
              <MDBox mb={2}>
                <MDTypography variant="button" color="text">Valor</MDTypography>
                <MDTypography variant="body1">{formatCurrency(invoice.amount)}</MDTypography>
              </MDBox>
              {invoice.description && (
                <MDBox mb={2}>
                  <MDTypography variant="button" color="text">Descrição</MDTypography>
                  <MDTypography variant="body1">{invoice.description}</MDTypography>
                </MDBox>
              )}
              {invoice.external_id && (
                <MDBox>
                  <MDTypography variant="button" color="text">External ID</MDTypography>
                  <MDTypography variant="caption" component="pre" sx={{ wordBreak: "break-all" }}>{invoice.external_id}</MDTypography>
                </MDBox>
              )}
            </MDBox>
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
