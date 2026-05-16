/**
 * Relatório de logs/auditoria
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
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
import { backofficeApi } from "services/api";

export default function ReportLogs() {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    backofficeApi.logs.list({ limit: 200 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Logs</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDBox>
            <MDTypography color="error">{error}</MDTypography>
            <MDTypography variant="caption" color="text" display="block" mt={1}>
              Logs podem exigir perfil backoffice. Faça login como backoffice para acessar.
            </MDTypography>
          </MDBox>
        ) : (
          <Card>
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>ID</TableCell>
                    <TableCell>Data</TableCell>
                    <TableCell>Nível</TableCell>
                    <TableCell>Mensagem</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row, i) => (
                    <TableRow key={row.id || i}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.created_at?.slice?.(0, 19) || "-"}</TableCell>
                      <TableCell>{row.level || "-"}</TableCell>
                      <TableCell sx={{ maxWidth: 500, overflow: "hidden", textOverflow: "ellipsis" }}>{row.message || row.msg || "-"}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhum log encontrado.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
