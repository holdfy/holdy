/**
 * Relatório de conciliação e inconsistências
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
import { adminApi, entityApi } from "services/api";

const STATUS_LABELS = { 1: "NEW", 2: "QUEUED", 3: "AWAITING", 4: "COMPLETED", 5: "ERROR", 9: "CANCEL" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportReconciliation() {
  const [transactions, setTransactions] = useState([]);
  const [medOpen, setMedOpen] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;
    Promise.all([
      adminApi.pix.transactions({ limit: 100, page: 1 }),
      entityApi.secMed.list({ limit: 200, offset: 0 }),
    ]).then(([txRes, medRes]) => {
      if (cancelled) return;
      const tx = txRes?.data ?? txRes?.items ?? txRes ?? [];
      const med = medRes?.items ?? medRes ?? [];
      setTransactions(Array.isArray(tx) ? tx : []);
      setMedOpen((Array.isArray(med) ? med : []).filter((m) => m.status_sec_med_id === 1));
    }).catch((e) => { if (!cancelled) setError(e.message); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, []);

  const failed = transactions.filter((t) => t.status_transaction_id === 5 || t.status_transaction_id === 7);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Conciliação e Inconsistências</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={3} display="flex" gap={2} flexWrap="wrap">
              <Card sx={{ p: 2, minWidth: 200 }}>
                <MDTypography variant="button" color="text">Transações com falha</MDTypography>
                <MDTypography variant="h4" color="error">{failed.length}</MDTypography>
              </Card>
              <Card sx={{ p: 2, minWidth: 200 }}>
                <MDTypography variant="button" color="text">MED em aberto</MDTypography>
                <MDTypography variant="h4" color="warning">{medOpen.length}</MDTypography>
              </Card>
            </MDBox>

            <Card sx={{ mb: 3 }}>
              <MDBox p={2}>
                <MDTypography variant="h6" fontWeight="medium" mb={2}>Transações com falha</MDTypography>
                <TableContainer>
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>ID</TableCell>
                        <TableCell>Data</TableCell>
                        <TableCell>Valor</TableCell>
                        <TableCell>Status</TableCell>
                        <TableCell>Erro</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {failed.slice(0, 20).map((row) => (
                        <TableRow key={row.id}>
                          <TableCell>{row.id}</TableCell>
                          <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                          <TableCell>{formatCurrency(row.amount)}</TableCell>
                          <TableCell><Chip label={STATUS_LABELS[row.status_transaction_id]} size="small" color="error" /></TableCell>
                          <TableCell sx={{ maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis" }}>{row.msg_error || "-"}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
                {failed.length === 0 && (
                  <MDBox p={2} textAlign="center">
                    <MDTypography color="text">Nenhuma transação com falha.</MDTypography>
                  </MDBox>
                )}
              </MDBox>
            </Card>

            <Card>
              <MDBox p={2}>
                <MDTypography variant="h6" fontWeight="medium" mb={2}>MED em aberto (pendentes)</MDTypography>
                <TableContainer>
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>ID</TableCell>
                        <TableCell>Conta</TableCell>
                        <TableCell>Valor</TableCell>
                        <TableCell>Liberação</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {medOpen.slice(0, 20).map((row) => (
                        <TableRow key={row.id}>
                          <TableCell>{row.id}</TableCell>
                          <TableCell>{row.account_id}</TableCell>
                          <TableCell>{formatCurrency(row.amount)}</TableCell>
                          <TableCell>{row.scheduled_date?.slice?.(0, 10) || "-"}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
                {medOpen.length === 0 && (
                  <MDBox p={2} textAlign="center">
                    <MDTypography color="text">Nenhum MED em aberto.</MDTypography>
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
