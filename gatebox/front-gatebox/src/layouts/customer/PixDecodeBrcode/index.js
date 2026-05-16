/**
 * PIX - Decodificar BR Code
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

export default function CustomerPixDecodeBrcode() {
  const [brcode, setBrcode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    setResult(null);
    if (!brcode.trim()) {
      setError("Cole o código PIX Copia e Cola.");
      return;
    }
    setLoading(true);
    try {
      const res = await customersApi.pix.decodeBrcode(brcode.trim());
      setResult(res);
    } catch (err) {
      setError(err.message || "Erro ao decodificar");
    } finally {
      setLoading(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Decodificar BR Code</MDTypography>
        <MDTypography variant="body2" color="text" mb={2}>
          Cole o código PIX Copia e Cola para visualizar os dados do pagamento.
        </MDTypography>
        <Card sx={{ maxWidth: 600 }}>
          <MDBox p={3} component="form" onSubmit={handleSubmit}>
            {error && <MDTypography color="error" variant="caption" display="block" mb={2}>{error}</MDTypography>}
            <MDBox mb={2}>
              <MDInput label="Código PIX (Copia e Cola)" fullWidth multiline rows={4} value={brcode} onChange={(e) => setBrcode(e.target.value)} placeholder="00020126580014br.gov.bcb.pix..." />
            </MDBox>
            <MDButton variant="gradient" color="info" fullWidth type="submit" disabled={loading}>
              {loading ? "Decodificando..." : "Decodificar"}
            </MDButton>
          </MDBox>
        </Card>
        {result && (
          <Card sx={{ mt: 3, maxWidth: 600 }}>
            <MDBox p={3}>
              <MDTypography variant="h6" fontWeight="medium" mb={2}>Dados detectados</MDTypography>
              <MDBox component="pre" sx={{ fontSize: 12, overflow: "auto", p: 2, bgcolor: "grey.100", borderRadius: 1 }}>
                {JSON.stringify(result, null, 2)}
              </MDBox>
            </MDBox>
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
