/**
 * Relatório - Usuários e Contas
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
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi, entityApi } from "services/api";

const STATUS_LABELS = { 1: "Ativo", 2: "Pendente", 3: "Aprovado", 4: "KYC Pendente" };

export default function ReportUsersAccounts() {
  const navigate = useNavigate();
  const [customers, setCustomers] = useState([]);
  const [accounts, setAccounts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [filter, setFilter] = useState("");

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [custRes, accRes] = await Promise.allSettled([
          adminApi.customers.list({ limit: 200, page: 1 }),
          entityApi.accounts.list({ limit: 200, offset: 0 }),
        ]);
        if (!cancelled) {
          if (custRes.status === "fulfilled") {
            const d = custRes.value?.data ?? custRes.value?.items ?? custRes.value;
            setCustomers(Array.isArray(d) ? d : []);
          }
          if (accRes.status === "fulfilled") {
            const d = accRes.value?.items ?? accRes.value?.data ?? accRes.value;
            setAccounts(Array.isArray(d) ? d : []);
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

  const custFiltered = filter
    ? customers.filter(
        (c) =>
          (c.full_name || c.name || "").toLowerCase().includes(filter.toLowerCase()) ||
          (c.email || "").toLowerCase().includes(filter.toLowerCase()) ||
          (c.document_number || "").includes(filter)
      )
    : customers;

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Relatório de Usuários e Contas</MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}><CircularProgress /></MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={2} display="flex" gap={2} alignItems="center">
              <MDInput
                placeholder="Filtrar por nome, email ou documento"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                sx={{ maxWidth: 350 }}
              />
              <MDTypography variant="caption" color="text">
                {custFiltered.length} cliente(s) · {accounts.length} conta(s)
              </MDTypography>
            </MDBox>
            <Card>
              <TableContainer>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>ID</TableCell>
                      <TableCell>Nome</TableCell>
                      <TableCell>Email</TableCell>
                      <TableCell>Documento</TableCell>
                      <TableCell>Status</TableCell>
                      <TableCell align="right">Ações</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {custFiltered.map((row) => (
                      <TableRow key={row.id}>
                        <TableCell>{row.id}</TableCell>
                        <TableCell>{row.full_name || row.name || "-"}</TableCell>
                        <TableCell>{row.email || "-"}</TableCell>
                        <TableCell>{row.document_number || "-"}</TableCell>
                        <TableCell>
                          <Chip
                            label={STATUS_LABELS[row.customer_status_id] ?? row.customer_status_id ?? "-"}
                            size="small"
                            color={row.customer_status_id === 4 ? "warning" : row.customer_status_id === 3 ? "success" : "default"}
                          />
                        </TableCell>
                        <TableCell align="right">
                          <MDTypography
                            component="span"
                            variant="caption"
                            color="info"
                            sx={{ cursor: "pointer" }}
                            onClick={() => navigate(`/admin/customers/${row.id}`)}
                          >
                            Ver
                          </MDTypography>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
              {custFiltered.length === 0 && (
                <MDBox p={3} textAlign="center">
                  <MDTypography color="text">Nenhum cliente encontrado.</MDTypography>
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
