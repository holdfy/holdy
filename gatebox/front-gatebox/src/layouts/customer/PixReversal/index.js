/**
 * PIX - Reversão (DPIX)
 */

import { useState } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { customersApi } from "services/api";

export default function CustomerPixReversal() {
  const [end2end, setEnd2end] = useState("");
  const [amount, setAmount] = useState("");
  const [externalId, setExternalId] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    setSuccess("");
    const amt = parseFloat(amount?.replace?.(",", "."));
    if (!end2end.trim() || !amt || amt <= 0) {
      setError("Preencha o end-to-end e o valor da devolução.");
      return;
    }
    setLoading(true);
    try {
      const body = { end2end: end2end.trim(), amount: amt };
      if (externalId.trim()) body.externalId = externalId.trim();
      await customersApi.pix.reversal(body);
      setSuccess("Devolução solicitada com sucesso!");
      setEnd2end("");
      setAmount("");
      setExternalId("");
    } catch (err) {
      setError(err.message || "Erro ao solicitar devolução");
    } finally {
      setLoading(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Devolução PIX (DPIX)</MDTypography>
        <MDTypography variant="body2" color="text" mb={2}>
          Solicite a devolução de um PIX enviado. Informe o end-to-end da transação original e o valor a devolver.
        </MDTypography>
        <Card sx={{ maxWidth: 500 }}>
          <MDBox p={3} component="form" onSubmit={handleSubmit}>
            {error && <MDTypography color="error" variant="caption" display="block" mb={2}>{error}</MDTypography>}
            {success && <MDTypography color="success" variant="caption" display="block" mb={2}>{success}</MDTypography>}
            <MDBox mb={2}>
              <MDInput label="End-to-end (E2E)" fullWidth value={end2end} onChange={(e) => setEnd2end(e.target.value)} required placeholder="E12345678901234567890123456789012" />
            </MDBox>
            <MDBox mb={2}>
              <MDInput label="Valor a devolver (R$)" type="number" fullWidth value={amount} onChange={(e) => setAmount(e.target.value)} inputProps={{ step: 0.01, min: 0 }} required />
            </MDBox>
            <MDBox mb={2}>
              <MDInput label="ID externo (opcional)" fullWidth value={externalId} onChange={(e) => setExternalId(e.target.value)} />
            </MDBox>
            <MDButton variant="gradient" color="warning" fullWidth type="submit" disabled={loading}>
              {loading ? "Solicitando..." : "Solicitar devolução"}
            </MDButton>
          </MDBox>
        </Card>
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
