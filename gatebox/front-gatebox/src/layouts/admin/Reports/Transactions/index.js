/**
 * Relatório - Transações (PIX in/out, período, tipo)
 */

import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
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
import { adminApi, entityApi } from "services/api";

const SUB_TYPE_LABELS = { 1: "PIX", 2: "DPIX", 3: "P2P", 4: "TPO", 5: "TTO", 6: "TPO", 7: "SMD" };
const STATUS_LABELS = { 1: "NEW", 2: "QUEUED", 3: "AWAITING", 4: "COMPLETED", 5: "ERROR", 9: "CANCEL" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportTransactions() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [source, setSource] = useState("admin"); // admin | entity

  useEffect(() => {
    setLoading(true);
    const load = source === "admin"
      ? adminApi.pix.transactions({ limit: 200, page: 1 })
      : entityApi.transaction.list({ limit: 200, offset: 0 });
    load
      .then((r) => {
        const d = r?.data ?? r?.items ?? r;
        setItems(Array.isArray(d) ? d : []);
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [source]);

  const subTypeLabel = (v) => (v != null ? (SUB_TYPE_LABELS[v] ?? String(v)) : "-");
  const statusLabel = (v) => (v != null ? (STATUS_LABELS[v] ?? String(v)) : "-");

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Transações</MDTypography>
        <MDBox mb={2} display="flex" gap={2} alignItems="center">
          <MDTypography variant="button" color="text">Fonte:</MDTypography>
          <select value={source} onChange={(e) => setSource(e.target.value)} style={{ padding: 8, borderRadius: 4, minWidth: 120 }}>
            <option value="admin">Admin PIX</option>
            <option value="entity">Entity API</option>
          </select>
          <MDTypography variant="caption" color="text">{items.length} transação(ões)</MDTypography>
        </MDBox>
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
                      <TableCell>{subTypeLabel(row.sub_type_transaction_id ?? row.sub_type)}</TableCell>
                      <TableCell>{statusLabel(row.status_transaction_id ?? row.status)}</TableCell>
                      <TableCell align="right" sx={{ color: Number(row.amount) >= 0 ? "success.main" : "error.main" }}>
                        {formatCurrency(row.amount)}
                      </TableCell>
                      <TableCell align="right" onClick={(e) => e.stopPropagation()}>
                        <MDTypography component="span" variant="caption" color="info" sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/pix/transactions/${row.id}`)}>
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
