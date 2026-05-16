/**
 * Admin - Transações PIX
 */

import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
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

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

const SUB_TYPE_LABELS = { 1: "PIX", 2: "DPIX", 3: "TTO", 4: "TPO", 5: "SMD", 6: "TPO", 7: "SMD" };
const STATUS_LABELS = { 1: "NEW", 2: "QUEUED", 3: "AWAITING", 4: "COMPLETED", 5: "ERROR", 6: "REFUNDED", 7: "FAILED", 8: "DROP", 9: "CANCEL" };
function subTypeLabel(v) {
  return v != null ? (SUB_TYPE_LABELS[v] ?? String(v)) : "-";
}
function statusLabel(v) {
  return v != null ? (STATUS_LABELS[v] ?? String(v)) : "-";
}

export default function AdminPixTransactions() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    adminApi.pix.transactions({ limit: 100, page: 1 })
      .then((r) => setItems(Array.isArray(r) ? r : r.data || r.items || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Transações PIX</MDTypography>
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
                    <TableCell>Tipo</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell align="right">Valor</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row, i) => (
                    <TableRow key={row.id || i} sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/pix/transactions/${row.id}`)}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                      <TableCell>{row.description || "-"}</TableCell>
                      <TableCell>{subTypeLabel(row.sub_type_transaction_id ?? row.sub_type) || "-"}</TableCell>
                      <TableCell>{statusLabel(row.status_transaction_id) || row.status || "-"}</TableCell>
                      <TableCell align="right">{formatCurrency(row.amount)}</TableCell>
                      <TableCell align="right" onClick={(e) => e.stopPropagation()}>
                        <MDButton variant="text" color="info" size="small" onClick={() => navigate(`/admin/pix/transactions/${row.id}`)}>
                          Ver
                        </MDButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhuma transação encontrada.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
