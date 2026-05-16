/**
 * Relatório de diferenças de saldo (saldo por conta)
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

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function ReportBalanceDifferences() {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    adminApi.reports
      .balanceDifferences()
      .then((r) => setItems(r?.items ?? []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const totalBalance = items.reduce((s, i) => s + (Number(i.balance) || 0), 0);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>
          Diferenças de Saldo
        </MDTypography>
        <MDTypography variant="body2" color="text" mb={2}>
          Saldo calculado por conta (soma de transações concluídas). Exibe até 500 contas. Use para conferir inconsistências.
        </MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}>
            <CircularProgress />
          </MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={3} display="flex" gap={2} flexWrap="wrap">
              <Card sx={{ p: 2, minWidth: 180 }}>
                <MDTypography variant="button" color="text">
                  Total contas
                </MDTypography>
                <MDTypography variant="h5" color="info">
                  {items.length}
                </MDTypography>
              </Card>
              <Card sx={{ p: 2, minWidth: 180 }}>
                <MDTypography variant="button" color="text">
                  Soma saldos
                </MDTypography>
                <MDTypography variant="h5" color="success">
                  {formatCurrency(totalBalance)}
                </MDTypography>
              </Card>
            </MDBox>

            <Card>
              <TableContainer>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>ID Conta</TableCell>
                      <TableCell>Número</TableCell>
                      <TableCell>Auth ID</TableCell>
                      <TableCell align="right">Saldo</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {items.map((row) => (
                      <TableRow key={row.account_id}>
                        <TableCell>{row.account_id}</TableCell>
                        <TableCell>{row.account_number || "-"}</TableCell>
                        <TableCell>{row.authentication_id}</TableCell>
                        <TableCell align="right">{formatCurrency(row.balance)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
              {items.length === 0 && (
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
