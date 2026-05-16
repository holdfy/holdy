/**
 * P2P - Histórico
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import CircularProgress from "@mui/material/CircularProgress";
import { useNavigate } from "react-router-dom";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { customersApi } from "services/api";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function CustomerP2PHistory() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    customersApi.p2p.history({ limit: 50 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Histórico P2P</MDTypography>
          <MDButton variant="gradient" color="info" onClick={() => navigate("/customer/p2p/send")}>
            Enviar P2P
          </MDButton>
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
                    <TableCell>Data</TableCell>
                    <TableCell>Transfer ID</TableCell>
                    <TableCell>Destinatário</TableCell>
                    <TableCell>Descrição</TableCell>
                    <TableCell align="right">Valor</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row, i) => (
                    <TableRow key={row.id || row.transfer_id || i}>
                      <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                      <TableCell>{row.transfer_id ?? row.id ?? "-"}</TableCell>
                      <TableCell>{row.receiver_name || row.receiver_id || "-"}</TableCell>
                      <TableCell>{row.description || "-"}</TableCell>
                      <TableCell align="right" sx={{ color: "error.main" }}>
                        -{formatCurrency(row.amount)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhuma transferência P2P.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
