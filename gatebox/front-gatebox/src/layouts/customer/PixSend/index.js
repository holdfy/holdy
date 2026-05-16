/**
 * PIX - Enviar
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
import { useSnackbar } from "context/SnackbarContext";

export default function CustomerPixSend() {
  const { showSuccess, showError } = useSnackbar();
  const [key, setKey] = useState("");
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [externalId, setExternalId] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    setSuccess("");
    const amt = parseFloat(amount?.replace?.(",", "."));
    if (!key.trim() || !amt || amt <= 0) {
      setError("Preencha a chave e o valor.");
      return;
    }
    setLoading(true);
    try {
      const body = { key: key.trim(), amount: amt };
      if (description.trim()) body.description = description.trim();
      if (externalId.trim()) body.externalId = externalId.trim();
      await customersApi.pix.send(body);
      showSuccess("PIX enviado com sucesso!");
      setSuccess("PIX enviado com sucesso!");
      setKey("");
      setAmount("");
      setDescription("");
      setExternalId("");
    } catch (err) {
      const msg = err.message || "Erro ao enviar PIX";
      setError(msg);
      showError(msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Enviar PIX</MDTypography>
        <Card sx={{ maxWidth: 500 }}>
          <MDBox p={3} component="form" onSubmit={handleSubmit}>
            {error && <MDTypography color="error" variant="caption" display="block" mb={2}>{error}</MDTypography>}
            {success && <MDTypography color="success" variant="caption" display="block" mb={2}>{success}</MDTypography>}
            <MDBox mb={2}>
              <MDInput label="Chave PIX (email, CPF, telefone ou aleatória)" fullWidth value={key} onChange={(e) => setKey(e.target.value)} required />
            </MDBox>
            <MDBox mb={2}>
              <MDInput label="Valor (R$)" type="number" fullWidth value={amount} onChange={(e) => setAmount(e.target.value)} inputProps={{ step: 0.01, min: 0 }} required />
            </MDBox>
            <MDBox mb={2}>
              <MDInput label="Descrição (opcional)" fullWidth value={description} onChange={(e) => setDescription(e.target.value)} />
            </MDBox>
            <MDBox mb={2}>
              <MDInput label="ID externo (opcional, idempotência)" fullWidth value={externalId} onChange={(e) => setExternalId(e.target.value)} />
            </MDBox>
            <MDButton variant="gradient" color="info" fullWidth type="submit" disabled={loading}>
              {loading ? "Enviando..." : "Enviar PIX"}
            </MDButton>
          </MDBox>
        </Card>
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
