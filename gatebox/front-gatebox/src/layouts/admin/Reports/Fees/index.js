/**
 * Relatório de taxas (TTO, TPO) por período
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
import { adminApi } from "services/api";

const SUB_TYPE_LABELS = { 3: "TTO", 4: "TPO", 5: "TTO", 6: "TPO" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportFees() {
  const [items, setItems] = useState([]);
  const [totals, setTotals] = useState({ tto: 0, tpo: 0 });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    adminApi.pix.transactions({ limit: 500, page: 1 })
      .then((r) => {
        const d = r?.data ?? r?.items ?? r ?? [];
        const list = Array.isArray(d) ? d : [];
        const fees = list.filter(
          (t) =>
            t.sub_type_transaction_id === 3 ||
            t.sub_type_transaction_id === 4 ||
            t.sub_type_transaction_id === 5 ||
            t.sub_type_transaction_id === 6
        );
        setItems(fees);
        const tto = fees
          .filter((t) => t.sub_type_transaction_id === 3 || t.sub_type_transaction_id === 5)
          .reduce((s, t) => s + (Number(t.amount) || 0), 0);
        const tpo = fees
          .filter((t) => t.sub_type_transaction_id === 4 || t.sub_type_transaction_id === 6)
          .reduce((s, t) => s + (Number(t.amount) || 0), 0);
        setTotals({ tto, tpo });
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Taxas (TTO / TPO)</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={3} display="flex" gap={2} flexWrap="wrap">
              <Card sx={{ p: 2, minWidth: 180 }}>
                <MDTypography variant="button" color="text">Total TTO</MDTypography>
                <MDTypography variant="h5" color="info">{formatCurrency(totals.tto)}</MDTypography>
              </Card>
              <Card sx={{ p: 2, minWidth: 180 }}>
                <MDTypography variant="button" color="text">Total TPO</MDTypography>
                <MDTypography variant="h5" color="success">{formatCurrency(totals.tpo)}</MDTypography>
              </Card>
              <Card sx={{ p: 2, minWidth: 180 }}>
                <MDTypography variant="button" color="text">Total geral</MDTypography>
                <MDTypography variant="h5" color="dark">{formatCurrency(totals.tto + totals.tpo)}</MDTypography>
              </Card>
            </MDBox>

            <Card>
              <TableContainer>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>ID</TableCell>
                      <TableCell>Data</TableCell>
                      <TableCell>Tipo</TableCell>
                      <TableCell>Conta</TableCell>
                      <TableCell>Descrição</TableCell>
                      <TableCell align="right">Valor</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {items.map((row, i) => (
                      <TableRow key={row.id || i}>
                        <TableCell>{row.id}</TableCell>
                        <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                        <TableCell>{SUB_TYPE_LABELS[row.sub_type_transaction_id] ?? row.sub_type_transaction_id}</TableCell>
                        <TableCell>{row.account_id}</TableCell>
                        <TableCell>{row.description || "-"}</TableCell>
                        <TableCell align="right">{formatCurrency(row.amount)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
              {items.length === 0 && (
                <MDBox p={3} textAlign="center">
                  <MDTypography color="text">Nenhuma taxa encontrada.</MDTypography>
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
