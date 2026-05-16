/**
 * Relatório de saldos (por conta, total MED bloqueado)
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
import { entityApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportBalances() {
  const [accounts, setAccounts] = useState([]);
  const [medItems, setMedItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;
    Promise.all([
      entityApi.accounts.list({ limit: 200, offset: 0 }),
      entityApi.secMed.list({ limit: 500, offset: 0 }),
    ]).then(([accRes, medRes]) => {
      if (cancelled) return;
      setAccounts(Array.isArray(accRes) ? accRes : accRes?.items || accRes?.data || []);
      setMedItems(Array.isArray(medRes) ? medRes : medRes?.items || medRes?.data || []);
    }).catch((e) => { if (!cancelled) setError(e.message); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, []);

  const medByAccount = medItems.reduce((m, item) => {
    m[item.account_id] = (m[item.account_id] || 0) + Number(item.amount || 0);
    return m;
  }, {});
  const totalMed = Object.values(medByAccount).reduce((s, v) => s + v, 0);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Saldos</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Total MED bloqueado: </MDTypography>
              <MDTypography variant="h5" color="warning">{formatCurrency(totalMed)}</MDTypography>
            </MDBox>
            <Card>
              <TableContainer>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>ID Conta</TableCell>
                      <TableCell>Número</TableCell>
                      <TableCell>MED bloqueado</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {accounts.map((row) => (
                      <TableRow key={row.id}>
                        <TableCell>{row.id}</TableCell>
                        <TableCell>{row.account_number || "-"}</TableCell>
                        <TableCell>{formatCurrency(medByAccount[row.id] || 0)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
              {accounts.length === 0 && (
                <MDBox p={3} textAlign="center">
                  <MDTypography color="text">Nenhuma conta encontrada.</MDTypography>
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
