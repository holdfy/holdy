/**
 * Verificar transação - página pública para consulta de prova blockchain
 */

import { useState } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import Grid from "@mui/material/Grid";
import PageLayout from "examples/LayoutContainers/PageLayout";
import DefaultNavbar from "examples/Navbars/DefaultNavbar";
import Footer from "layouts/authentication/components/Footer";
import Icon from "@mui/material/Icon";
import { API_BASE } from "services/api";

const pageRoutes = [{ name: "Gatebox", route: "/" }];

export default function VerifyTransaction() {
  const [entityType, setEntityType] = useState("pix_tx");
  const [entityId, setEntityId] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState(null);
  const [error, setError] = useState("");

  const handleSearch = async (e) => {
    e.preventDefault();
    setError("");
    setResult(null);
    if (!entityId.trim()) {
      setError("Informe o ID da transação.");
      return;
    }
    setLoading(true);
    try {
      const res = await fetch(
        `${API_BASE}/anchor/audit?entity_type=${encodeURIComponent(entityType)}&entity_id=${encodeURIComponent(entityId.trim())}&limit=1`
      );
      const json = await res.json().catch(() => ({}));
      if (!res.ok) {
        setError(json.message || "Erro ao buscar");
        setResult(null);
        return;
      }
      const proof = json.items?.[0] || null;
      setResult(proof);
      if (!proof) setError("Nenhum registro de ancoragem encontrado para esta transação.");
    } catch (err) {
      setError(err.message || "Erro ao buscar");
      setResult(null);
    } finally {
      setLoading(false);
    }
  };

  return (
    <PageLayout>
      <DefaultNavbar routes={pageRoutes} transparent light />
      <MDBox
        position="absolute"
        width="100%"
        minHeight="100vh"
        sx={{
          backgroundImage: ({ functions: { linearGradient, rgba }, palette: { gradients } }) =>
            `${linearGradient(rgba(gradients.dark.main, 0.6), rgba(gradients.dark.state, 0.6))}, url(https://images.unsplash.com/photo-1557683316-973673baf926?w=1920)`,
          backgroundSize: "cover",
          backgroundPosition: "center",
        }}
      />
      <MDBox px={1} width="100%" minHeight="100vh" mx="auto" pt={8} pb={4}>
        <Grid container justifyContent="center" spacing={2}>
          <Grid item xs={12} md={6}>
            <Card>
              <MDBox p={3}>
                <MDTypography variant="h4" fontWeight="medium" mb={1}>
                  Verificar transação na blockchain
                </MDTypography>
                <MDTypography variant="body2" color="text" mb={3}>
                  Consulte a prova de ancoragem de uma transação PIX ou MED na blockchain Polygon.
                </MDTypography>
                <MDBox component="form" onSubmit={handleSearch}>
                  {error && (
                    <MDTypography color="error" variant="caption" display="block" mb={2}>
                      {error}
                    </MDTypography>
                  )}
                  <MDBox mb={2}>
                    <MDTypography variant="button" color="text" display="block" mb={1}>Tipo</MDTypography>
                    <select
                      value={entityType}
                      onChange={(e) => setEntityType(e.target.value)}
                      style={{ width: "100%", padding: 12, borderRadius: 8, border: "1px solid #ccc" }}
                    >
                      <option value="pix_tx">PIX (Transação)</option>
                      <option value="med">MED</option>
                    </select>
                  </MDBox>
                  <MDBox mb={2}>
                    <MDInput
                      label="ID da transação"
                      fullWidth
                      value={entityId}
                      onChange={(e) => setEntityId(e.target.value)}
                      placeholder="Ex: 12345"
                    />
                  </MDBox>
                  <MDButton variant="gradient" color="info" fullWidth type="submit" disabled={loading}>
                    {loading ? "Buscando..." : "Verificar"}
                  </MDButton>
                </MDBox>
                {result && (
                  <MDBox mt={3} p={2} bgcolor="success.lighter" borderRadius={1}>
                    <MDTypography variant="button" fontWeight="medium" mb={1} color="success">
                      Registro na blockchain encontrado
                    </MDTypography>
                    <MDBox>
                      {result.tx_hash && (
                        <MDTypography variant="caption" display="block">
                          Tx: {String(result.tx_hash).slice(0, 10)}...{String(result.tx_hash).slice(-8)}
                        </MDTypography>
                      )}
                      {result.block_number != null && (
                        <MDTypography variant="caption" display="block">Bloco: {result.block_number}</MDTypography>
                      )}
                      {result.explorer_url && (
                        <MDButton component="a" href={result.explorer_url} target="_blank" rel="noopener noreferrer" size="small" color="info" sx={{ mt: 1 }}>
                          <Icon sx={{ mr: 0.5 }}>open_in_new</Icon> Ver no Polygonscan
                        </MDButton>
                      )}
                    </MDBox>
                  </MDBox>
                )}
              </MDBox>
            </Card>
          </Grid>
        </Grid>
      </MDBox>
      <Footer light />
    </PageLayout>
  );
}
