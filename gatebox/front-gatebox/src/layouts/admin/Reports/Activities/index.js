/**
 * Relatório de atividades por cliente
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

function formatDate(v) {
  if (!v) return "-";
  const d = new Date(v);
  return d.toLocaleString("pt-BR");
}

export default function ReportActivities() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [pagination, setPagination] = useState({ page: 1, limit: 50, total: 0 });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const load = (page = 1) => {
    setLoading(true);
    adminApi.reports
      .customerActivities({ page, limit: 50 })
      .then((r) => {
        setItems(r?.items ?? []);
        setPagination(r?.pagination ?? { page: 1, limit: 50, total: 0 });
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    load(1);
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>
          Relatório de Atividades por Cliente
        </MDTypography>
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}>
            <CircularProgress />
          </MDBox>
        ) : error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <>
            <MDBox mb={2}>
              <MDTypography variant="caption" color="text">
                {pagination.total} cliente(s) com conta
              </MDTypography>
            </MDBox>
            <Card>
              <TableContainer>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>ID</TableCell>
                      <TableCell>Nome</TableCell>
                      <TableCell align="right">Transações</TableCell>
                      <TableCell>Última atividade</TableCell>
                      <TableCell align="right">Ações</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {items.map((row) => (
                      <TableRow key={row.customer_id}>
                        <TableCell>{row.customer_id}</TableCell>
                        <TableCell>{row.full_name || "-"}</TableCell>
                        <TableCell align="right">{row.tx_count ?? 0}</TableCell>
                        <TableCell>{formatDate(row.last_activity)}</TableCell>
                        <TableCell align="right">
                          <MDTypography
                            component="span"
                            variant="caption"
                            color="info"
                            sx={{ cursor: "pointer" }}
                            onClick={() => navigate(`/admin/customers/${row.customer_id}`)}
                          >
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
                  <MDTypography color="text">Nenhum cliente com conta encontrado.</MDTypography>
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
