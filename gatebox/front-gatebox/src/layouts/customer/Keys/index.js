/**
 * Chaves PIX - listar, cadastrar, remover
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import MDInput from "components/MDInput";
import Card from "@mui/material/Card";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { customersApi } from "services/api";

export default function CustomerKeys() {
  const [keys, setKeys] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [open, setOpen] = useState(false);
  const [newKey, setNewKey] = useState("");
  const [keyType] = useState("email");
  const [submitting, setSubmitting] = useState(false);

  const load = () => {
    setLoading(true);
    customersApi.account.keys.list()
      .then((r) => setKeys(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const handleAdd = async () => {
    if (!newKey.trim()) return;
    setSubmitting(true);
    try {
      await customersApi.account.keys.create({ key: newKey.trim(), type: keyType });
      setOpen(false);
      setNewKey("");
      load();
    } catch (e) {
      setError(e.message);
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id) => {
    if (!window.confirm("Remover esta chave?")) return;
    try {
      await customersApi.account.keys.delete(id);
      load();
    } catch (e) {
      setError(e.message);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Chaves PIX</MDTypography>
          <MDButton variant="gradient" color="info" onClick={() => setOpen(true)}>
            Cadastrar chave
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
                    <TableCell>Chave</TableCell>
                    <TableCell>Tipo</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {keys.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.key || row.pix_key || "-"}</TableCell>
                      <TableCell>{row.type || row.pix_key_type || "-"}</TableCell>
                      <TableCell align="right">
                        <MDButton variant="text" color="error" size="small" onClick={() => handleDelete(row.id)}>
                          Remover
                        </MDButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {keys.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhuma chave cadastrada.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>

      <Dialog open={open} onClose={() => setOpen(false)}>
        <DialogTitle>Cadastrar chave PIX</DialogTitle>
        <DialogContent>
          <MDBox pt={1}>
            <MDInput
              label="Chave (email, CPF, telefone ou aleatória)"
              fullWidth
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              sx={{ mb: 2 }}
            />
            <MDTypography variant="caption" color="text">
              Tipo: {keyType}
            </MDTypography>
          </MDBox>
        </DialogContent>
        <DialogActions>
          <MDButton onClick={() => setOpen(false)}>Cancelar</MDButton>
          <MDButton variant="gradient" color="info" onClick={handleAdd} disabled={submitting}>
            {submitting ? "Salvando..." : "Cadastrar"}
          </MDButton>
        </DialogActions>
      </Dialog>
      <Footer />
    </DashboardLayout>
  );
}
