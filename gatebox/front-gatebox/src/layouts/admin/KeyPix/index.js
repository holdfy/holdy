/**
 * Admin - Chaves PIX (listar / cadastrar)
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
import Grid from "@mui/material/Grid";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { entityApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

export default function AdminKeyPix() {
  const { showSuccess, showError } = useSnackbar();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState({ key: "", pix_key_type_id: 1, document_number: "", description: "", account_id: 1, partners_id: 1 });
  const [submitting, setSubmitting] = useState(false);

  const load = () => {
    setLoading(true);
    entityApi.keyPix.list({ limit: 100, offset: 0 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const handleCreate = async () => {
    if (!form.key?.trim()) {
      showError("Chave é obrigatória");
      return;
    }
    setSubmitting(true);
    try {
      await entityApi.keyPix.create(form);
      showSuccess("Chave cadastrada");
      setOpen(false);
      setForm({ key: "", pix_key_type_id: 1, document_number: "", description: "", account_id: 1, partners_id: 1 });
      load();
    } catch (e) {
      showError(e.message || "Erro ao cadastrar");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Chaves PIX (Admin)</MDTypography>
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
                    <TableCell>ID</TableCell>
                    <TableCell>Chave</TableCell>
                    <TableCell>Documento</TableCell>
                    <TableCell>Conta</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.key || "-"}</TableCell>
                      <TableCell>{row.document_number || "-"}</TableCell>
                      <TableCell>{row.account_id}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhuma chave cadastrada.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Cadastrar chave PIX</DialogTitle>
        <DialogContent>
          <MDBox pt={1}>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <MDInput label="Chave" fullWidth required value={form.key} onChange={(e) => setForm((f) => ({ ...f, key: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Tipo (pix_key_type_id)" fullWidth value={form.pix_key_type_id} onChange={(e) => setForm((f) => ({ ...f, pix_key_type_id: parseInt(e.target.value, 10) || 1 }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput label="Documento" fullWidth value={form.document_number} onChange={(e) => setForm((f) => ({ ...f, document_number: e.target.value }))} />
              </Grid>
              <Grid item xs={12}>
                <MDInput label="Descrição" fullWidth value={form.description} onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Conta (account_id)" fullWidth value={form.account_id} onChange={(e) => setForm((f) => ({ ...f, account_id: parseInt(e.target.value, 10) || 1 }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Parceiro (partners_id)" fullWidth value={form.partners_id} onChange={(e) => setForm((f) => ({ ...f, partners_id: parseInt(e.target.value, 10) || 1 }))} />
              </Grid>
            </Grid>
          </MDBox>
        </DialogContent>
        <DialogActions>
          <MDButton onClick={() => setOpen(false)}>Cancelar</MDButton>
          <MDButton variant="gradient" color="info" onClick={handleCreate} disabled={submitting}>
            {submitting ? "Cadastrando..." : "Cadastrar"}
          </MDButton>
        </DialogActions>
      </Dialog>
      <Footer />
    </DashboardLayout>
  );
}
