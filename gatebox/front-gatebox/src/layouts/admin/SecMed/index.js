/**
 * Admin - MED (Reserva de Segurança)
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
import { useNavigate } from "react-router-dom";
import { entityApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function AdminSecMed() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    entityApi.secMed.list({ limit: 100, offset: 0 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>MED - Reserva de Segurança</MDTypography>
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
                    <TableCell>Conta</TableCell>
                    <TableCell>Valor</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell>Liberação</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id} sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/sec-med/${row.id}`)}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.account_id}</TableCell>
                      <TableCell>{formatCurrency(row.amount)}</TableCell>
                      <TableCell>
                      {row.status_sec_med_id === 1 ? "OPEN" : row.status_sec_med_id === 2 ? "RETURNED" : row.status_sec_med_id || "-"}
                    </TableCell>
                      <TableCell>{row.scheduled_date?.slice?.(0, 10) || "-"}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhum MED encontrado.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
