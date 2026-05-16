/**
 * Relatório de MED (listagem, status, prazos)
 */

import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import Card from "@mui/material/Card";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { entityApi } from "services/api";

const STATUS_LABELS = { 1: "OPEN", 2: "RETURNED" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportMed() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    entityApi.secMed.list({ limit: 200, offset: 0 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const total = items.reduce((s, m) => s + (Number(m.amount) || 0), 0);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de MED</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Total em MED: </MDTypography>
              <MDTypography variant="h5" color="warning">{formatCurrency(total)}</MDTypography>
            </MDBox>
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
                      <TableCell align="right">Ações</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {items.map((row) => (
                      <TableRow key={row.id} sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/sec-med/${row.id}`)}>
                        <TableCell>{row.id}</TableCell>
                        <TableCell>{row.account_id}</TableCell>
                        <TableCell>{formatCurrency(row.amount)}</TableCell>
                        <TableCell>
                          <Chip label={STATUS_LABELS[row.status_sec_med_id] ?? row.status_sec_med_id} size="small" color={row.status_sec_med_id === 1 ? "warning" : "success"} />
                        </TableCell>
                        <TableCell>{row.scheduled_date?.slice?.(0, 10) || "-"}</TableCell>
                        <TableCell align="right" onClick={(e) => e.stopPropagation()}>
                          <MDTypography variant="caption" color="info" sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/sec-med/${row.id}`)}>
                            Ver
                          </MDTypography>
                        </TableCell>
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
          </>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
