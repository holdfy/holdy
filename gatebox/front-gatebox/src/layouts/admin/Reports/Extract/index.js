/**
 * Extrato consolidado (admin) - todas as transações com filtros
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
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";

const SUB_TYPE_LABELS = { 1: "PIX", 2: "DPIX", 3: "P2P", 4: "TPO", 5: "TTO", 7: "SMD" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportExtract() {
  const navigate = useNavigate();
  const [allItems, setAllItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [filterType, setFilterType] = useState("");

  useEffect(() => {
    adminApi.pix.transactions({ limit: 300, page: 1 })
      .then((r) => {
        const d = r?.data ?? r?.items ?? r ?? [];
        setAllItems(Array.isArray(d) ? d : []);
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const items = filterType
    ? allItems.filter((t) => t.sub_type_transaction_id === parseInt(filterType, 10))
    : allItems;

  const subTypeLabel = (v) => (v != null ? (SUB_TYPE_LABELS[v] ?? String(v)) : "-");

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Extrato Consolidado</MDTypography>
        <MDBox mb={2} display="flex" gap={2} alignItems="center" flexWrap="wrap">
          <MDTypography variant="button" color="text">Filtrar por tipo:</MDTypography>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            style={{ padding: 8, borderRadius: 4, minWidth: 120 }}
          >
            <option value="">Todos</option>
            <option value="1">PIX</option>
            <option value="2">DPIX</option>
            <option value="3">P2P</option>
            <option value="4">TPO</option>
            <option value="5">TTO</option>
            <option value="7">SMD</option>
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
                    <TableCell>Conta</TableCell>
                    <TableCell>Descrição</TableCell>
                    <TableCell>Tipo</TableCell>
                    <TableCell align="right">Valor</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row, i) => (
                    <TableRow key={row.id || i} sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/pix/transactions/${row.id}`)}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                      <TableCell>{row.account_id}</TableCell>
                      <TableCell>{row.description || "-"}</TableCell>
                      <TableCell>{subTypeLabel(row.sub_type_transaction_id)}</TableCell>
                      <TableCell align="right" sx={{ color: Number(row.amount) >= 0 ? "success.main" : "error.main" }}>
                        {formatCurrency(row.amount)}
                      </TableCell>
                      <TableCell align="right" onClick={(e) => e.stopPropagation()}>
                        <MDTypography variant="caption" color="info" sx={{ cursor: "pointer" }} onClick={() => navigate(`/admin/pix/transactions/${row.id}`)}>
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
