/**
 * Relatório - Invoices (QR Code – criadas, pagas, canceladas)
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
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { entityApi } from "services/api";

const INVOICE_STATUS = { 1: "CREATED", 2: "DONE", 3: "CANCEL" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportInvoices() {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    entityApi.invoice.list({ limit: 200, offset: 0 })
      .then((r) => setItems(Array.isArray(r) ? r : (r?.items ?? r?.data ?? []) || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const statusLabel = (v) => (v != null ? (INVOICE_STATUS[v] ?? String(v)) : "-");
  const statusColor = (v) => (v === 2 ? "success" : v === 3 ? "error" : "info");

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Invoices</MDTypography>
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
                    <TableCell>Data</TableCell>
                    <TableCell>Descrição</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell align="right">Valor</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                      <TableCell>{row.description || "-"}</TableCell>
                      <TableCell>
                        <Chip label={statusLabel(row.invoice_status_id)} size="small" color={statusColor(row.invoice_status_id)} />
                      </TableCell>
                      <TableCell align="right">{formatCurrency(row.amount)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhuma invoice encontrada.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
