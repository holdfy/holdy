/**
 * Relatório de webhooks (configurações, testes, falhas)
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

export default function ReportWebhooks() {
  const { showSuccess, showError } = useSnackbar();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [testing, setTesting] = useState(null);

  useEffect(() => {
    adminApi.webhooks.list({ page: 1, limit: 100 })
      .then((r) => setItems(Array.isArray(r) ? r : r.data || r.items || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const handleTest = async (id) => {
    setTesting(id);
    try {
      await adminApi.webhooks.test(id);
      showSuccess("Teste enviado");
    } catch (e) {
      showError(e.message || "Erro no teste");
    } finally {
      setTesting(null);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Webhooks</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <Card>
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>ID</TableCell>
                    <TableCell>URL</TableCell>
                    <TableCell>Tipo</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell sx={{ maxWidth: 350, overflow: "hidden", textOverflow: "ellipsis" }}>{row.callback_url || row.url || "-"}</TableCell>
                      <TableCell>{row.webhook_type_id ?? row.type ?? "-"}</TableCell>
                      <TableCell align="right">
                        <MDButton variant="text" color="secondary" size="small" onClick={() => handleTest(row.id)} disabled={testing === row.id}>
                          {testing === row.id ? "..." : "Testar"}
                        </MDButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhum webhook configurado.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
