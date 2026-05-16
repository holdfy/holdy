/**
 * Admin - Conta (saldo, extrato, MED, PIX in/out)
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
import Grid from "@mui/material/Grid";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi, entityApi } from "services/api";

const SUB_TYPE_LABELS = { 1: "PIX", 2: "DPIX", 3: "P2P", 5: "TTO", 6: "TPO", 7: "SMD" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function AdminAccount() {
  const [balance, setBalance] = useState(null);
  const [transactions, setTransactions] = useState([]);
  const [medTotal, setMedTotal] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [txRes, medRes] = await Promise.allSettled([
          adminApi.pix.transactions({ limit: 50, page: 1 }),
          entityApi.secMed.list({ limit: 500, offset: 0 }),
        ]);
        if (!cancelled) {
          if (txRes.status === "fulfilled") {
            const d = txRes.value?.data ?? txRes.value?.items ?? txRes.value;
            setTransactions(Array.isArray(d) ? d : []);
            const pixIn = (Array.isArray(d) ? d : []).filter((t) => t.type_transaction_id === 2).length;
            const pixOut = (Array.isArray(d) ? d : []).filter((t) => t.type_transaction_id === 1).length;
            setBalance({
              balance: (Array.isArray(d) ? d : []).reduce((s, t) => s + (Number(t.amount) || 0) * (t.type_transaction_id === 2 ? 1 : -1), 0),
              pixInCount: pixIn,
              pixOutCount: pixOut,
            });
          }
          if (medRes.status === "fulfilled") {
            const items = medRes.value?.items ?? medRes.value ?? [];
            const total = (Array.isArray(items) ? items : []).reduce((s, m) => s + (Number(m.amount) || 0), 0);
            setMedTotal(total);
          }
        }
      } catch (e) {
        if (!cancelled) setError(e.message);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, []);

  const subTypeLabel = (v) => (v != null ? (SUB_TYPE_LABELS[v] ?? String(v)) : "-");

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Conta Admin</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <Grid container spacing={3} mb={3}>
              <Grid item xs={12} md={4}>
                <Card>
                  <MDBox p={2}>
                    <MDTypography variant="button" color="text">Transações PIX (últimas)</MDTypography>
                    <MDTypography variant="h4" color="info">{transactions.length}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} md={4}>
                <Card>
                  <MDBox p={2}>
                    <MDTypography variant="button" color="text">Total MED bloqueado</MDTypography>
                    <MDTypography variant="h4" color="warning">{formatCurrency(medTotal ?? 0)}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
              <Grid item xs={12} md={4}>
                <Card>
                  <MDBox p={2}>
                    <MDTypography variant="button" color="text">PIX IN / PIX OUT</MDTypography>
                    <MDTypography variant="h6">{balance?.pixInCount ?? 0} / {balance?.pixOutCount ?? 0}</MDTypography>
                  </MDBox>
                </Card>
              </Grid>
            </Grid>

            <Card>
              <MDBox p={2}>
                <MDTypography variant="h6" fontWeight="medium" mb={2}>Últimas transações</MDTypography>
                <TableContainer>
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>ID</TableCell>
                        <TableCell>Data</TableCell>
                        <TableCell>Descrição</TableCell>
                        <TableCell>Tipo</TableCell>
                        <TableCell align="right">Valor</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {transactions.slice(0, 20).map((row, i) => (
                        <TableRow key={row.id || i}>
                          <TableCell>{row.id}</TableCell>
                          <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                          <TableCell>{row.description || "-"}</TableCell>
                          <TableCell>{subTypeLabel(row.sub_type_transaction_id)}</TableCell>
                          <TableCell align="right" sx={{ color: Number(row.amount) >= 0 ? "success.main" : "error.main" }}>
                            {formatCurrency(row.amount)}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
                {transactions.length === 0 && (
                  <MDBox p={3} textAlign="center">
                    <MDTypography color="text">Nenhuma transação.</MDTypography>
                  </MDBox>
                )}
              </MDBox>
            </Card>
          </>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
