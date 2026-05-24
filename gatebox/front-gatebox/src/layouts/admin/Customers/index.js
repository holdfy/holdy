/**
 * Admin - Clientes (com filtro PF / PJ)
 */

import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
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
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Chip from "@mui/material/Chip";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";

// type_person_id: 1=PF (NATURAL_PERSON), 2=PJ (LEGAL_PERSON)
function personTypeLabel(typeId) {
  if (typeId === 2 || typeId === "2") return { label: "PJ", color: "primary" };
  return { label: "PF", color: "default" };
}

export default function AdminCustomers() {
  const navigate = useNavigate();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [personFilter, setPersonFilter] = useState("all"); // "all" | "pf" | "pj"

  useEffect(() => {
    adminApi.customers.list()
      .then((r) => setItems(Array.isArray(r) ? r : r.data || r.items || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const filtered = items.filter((row) => {
    if (personFilter === "pf") return row.type_person_id !== 2 && row.type_person_id !== "2";
    if (personFilter === "pj") return row.type_person_id === 2 || row.type_person_id === "2";
    return true;
  });

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" alignItems="center" justifyContent="space-between" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Clientes</MDTypography>
          <ToggleButtonGroup
            value={personFilter}
            exclusive
            onChange={(_, v) => v && setPersonFilter(v)}
            size="small"
          >
            <ToggleButton value="all">Todos</ToggleButton>
            <ToggleButton value="pf">PF</ToggleButton>
            <ToggleButton value="pj">PJ</ToggleButton>
          </ToggleButtonGroup>
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
                    <TableCell>Tipo</TableCell>
                    <TableCell>Nome</TableCell>
                    <TableCell>Documento</TableCell>
                    <TableCell>Email</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {filtered.map((row) => {
                    const { label, color } = personTypeLabel(row.type_person_id);
                    return (
                      <TableRow
                        key={row.id}
                        sx={{ cursor: "pointer" }}
                        onClick={() => navigate(`/admin/customers/${row.id}`)}
                      >
                        <TableCell>{row.id}</TableCell>
                        <TableCell>
                          <Chip label={label} color={color} size="small" />
                        </TableCell>
                        <TableCell>{row.full_name || row.name || row.username || "-"}</TableCell>
                        <TableCell>{row.document_number || "-"}</TableCell>
                        <TableCell>{row.email || "-"}</TableCell>
                        <TableCell>{row.status || row.customer_status_id || "-"}</TableCell>
                        <TableCell align="right" onClick={(e) => e.stopPropagation()}>
                          <MDButton
                            variant="text"
                            color="info"
                            size="small"
                            onClick={() => navigate(`/admin/customers/${row.id}`)}
                          >
                            Ver
                          </MDButton>
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </TableContainer>
            {filtered.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">
                  {personFilter === "all" ? "Nenhum cliente encontrado." : `Nenhum cliente ${personFilter.toUpperCase()} encontrado.`}
                </MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
