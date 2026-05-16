/**
 * Conta - Extrato
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
import { customersApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

const SUB_TYPE_LABELS = { 1: "PIX", 2: "DPIX", 3: "TTO", 4: "TPO", 5: "SMD" };
function subTypeLabel(v) {
  if (v == null || v === "") return "-";
  return SUB_TYPE_LABELS[v] ?? String(v);
}

export default function CustomerExtract() {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    customersApi.account.extract({ limit: 50 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Extrato</MDTypography>
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
                    <TableCell>Data</TableCell>
                    <TableCell>Descrição</TableCell>
                    <TableCell>Tipo</TableCell>
                    <TableCell align="right">Valor</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row, i) => (
                    <TableRow key={row.id || i}>
                      <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                      <TableCell>{row.description || "-"}</TableCell>
                      <TableCell>{subTypeLabel(row.sub_type) || row.type || "-"}</TableCell>
                      <TableCell align="right" sx={{ color: Number(row.amount) >= 0 ? "success.main" : "error.main" }}>
                        {formatCurrency(row.amount)}
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
