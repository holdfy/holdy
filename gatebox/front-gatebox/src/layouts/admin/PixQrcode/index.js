/**
 * Admin - Gerar QR Code PIX
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
import { adminApi } from "services/api";

export default function AdminPixQrcode() {
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    setResult(null);
    const amt = parseFloat(amount?.replace?.(",", "."));
    if (!amt || amt <= 0) {
      setError("Informe o valor.");
      return;
    }
    setLoading(true);
    try {
      const body = { amount: amt };
      if (description.trim()) body.description = description.trim();
      const res = await adminApi.pix.qrcode(body);
      setResult(res);
    } catch (err) {
      setError(err.message || "Erro ao gerar QR Code");
    } finally {
      setLoading(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Gerar QR Code PIX (Admin)</MDTypography>
        <Card sx={{ maxWidth: 500 }}>
          <MDBox p={3} component="form" onSubmit={handleSubmit}>
            {error && <MDTypography color="error" variant="caption" display="block" mb={2}>{error}</MDTypography>}
            <MDBox mb={2}>
              <MDInput label="Valor (R$)" type="number" fullWidth value={amount} onChange={(e) => setAmount(e.target.value)} inputProps={{ step: 0.01, min: 0 }} required />
            </MDBox>
            <MDBox mb={2}>
              <MDInput label="Descrição (opcional)" fullWidth value={description} onChange={(e) => setDescription(e.target.value)} />
            </MDBox>
            <MDButton variant="gradient" color="info" fullWidth type="submit" disabled={loading}>
              {loading ? "Gerando..." : "Gerar QR Code"}
            </MDButton>
          </MDBox>
        </Card>
        {result && (
          <Card sx={{ mt: 3, maxWidth: 500 }}>
            <MDBox p={3}>
              <MDTypography variant="h6" fontWeight="medium" mb={2}>QR Code gerado</MDTypography>
              {(result.brcode || result.qrCode) && (
                <MDTypography variant="caption" component="pre" sx={{ wordBreak: "break-all", display: "block", mb: 2 }}>
                  {result.brcode || result.qrCode}
                </MDTypography>
              )}
              {(result.qrcode_base64 || result.data?.qrcode) && (
                <MDBox component="img" src={`data:image/png;base64,${result.qrcode_base64 || result.data?.qrcode}`} alt="QR Code" sx={{ maxWidth: 200, mb: 2 }} />
              )}
              {(result.txid || result.txId) && <MDTypography variant="caption" color="text">TxID: {result.txid || result.txId}</MDTypography>}
            </MDBox>
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
