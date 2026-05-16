/**
 * Admin - Parceiros (CRUD)
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
import Switch from "@mui/material/Switch";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

const defaultPartner = {
  description: "",
  partners_list_id: 1,
  document: "",
  account: "",
  branch: "",
  authentication_id: 0,
  client_id: "",
  client_secret: "",
  authentication: "",
  password: "",
  whpix_in_id: "",
  whpix_out_id: "",
  type_authorize_id: 1,
  fixed_cash_in: 0,
  fixed_cash_out: 0,
  percent_cashin: 0,
  percent_cashout: 0,
  fixed_ref_cash_in: 0,
  fixed_ref_cash_out: 0,
  percent_ref_cashin: 0,
  percent_ref_cashout: 0,
  active: true,
};

export default function AdminPartners() {
  const { showSuccess, showError } = useSnackbar();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState(null);
  const [form, setForm] = useState({ ...defaultPartner });
  const [submitting, setSubmitting] = useState(false);

  const load = () => {
    setLoading(true);
    adminApi.settings.partners.list({ page: 1, limit: 50 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const openCreate = () => {
    setEditing(null);
    setForm({ ...defaultPartner });
    setOpen(true);
  };

  const openEdit = (row) => {
    setEditing(row);
    setForm({
      description: row.description ?? "",
      partners_list_id: row.partners_list_id ?? 1,
      document: row.document ?? "",
      account: row.account ?? "",
      branch: row.branch ?? "",
      authentication_id: row.authentication_id ?? 0,
      client_id: row.client_id ?? "",
      client_secret: row.client_secret ?? "",
      authentication: row.authentication ?? "",
      password: row.password ?? "",
      whpix_in_id: row.whpix_in_id ?? "",
      whpix_out_id: row.whpix_out_id ?? "",
      type_authorize_id: row.type_authorize_id ?? 1,
      fixed_cash_in: Number(row.fixed_cash_in) ?? 0,
      fixed_cash_out: Number(row.fixed_cash_out) ?? 0,
      percent_cashin: Number(row.percent_cashin) ?? 0,
      percent_cashout: Number(row.percent_cashout) ?? 0,
      fixed_ref_cash_in: Number(row.fixed_ref_cash_in) ?? 0,
      fixed_ref_cash_out: Number(row.fixed_ref_cash_out) ?? 0,
      percent_ref_cashin: Number(row.percent_ref_cashin) ?? 0,
      percent_ref_cashout: Number(row.percent_ref_cashout) ?? 0,
      active: row.active !== false,
    });
    setOpen(true);
  };

  const handleSave = async () => {
    if (!form.description?.trim()) {
      showError("Descrição é obrigatória");
      return;
    }
    setSubmitting(true);
    try {
      const payload = { ...form, id: editing?.id ?? 0 };
      if (editing) {
        await adminApi.settings.partners.update(editing.id, payload);
        showSuccess("Parceiro atualizado");
      } else {
        await adminApi.settings.partners.create(payload);
        showSuccess("Parceiro criado");
      }
      setOpen(false);
      load();
    } catch (e) {
      showError(e.message || "Erro ao salvar");
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (row) => {
    if (!window.confirm(`Remover parceiro "${row.description || row.id}"?`)) return;
    try {
      await adminApi.settings.partners.delete(row.id);
      showSuccess("Parceiro removido");
      load();
    } catch (e) {
      showError(e.message || "Erro ao remover");
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Parceiros</MDTypography>
          <MDButton variant="gradient" color="info" onClick={openCreate}>
            Novo parceiro
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
                    <TableCell>Descrição</TableCell>
                    <TableCell>Taxa IN</TableCell>
                    <TableCell>Taxa OUT</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.description || row.name || "-"}</TableCell>
                      <TableCell>{row.percent_cashin ?? row.fixed_cash_in ?? "-"}</TableCell>
                      <TableCell>{row.percent_cashout ?? row.fixed_cash_out ?? "-"}</TableCell>
                      <TableCell align="right">
                        <MDButton variant="text" color="info" size="small" onClick={() => openEdit(row)} sx={{ mr: 1 }}>
                          Editar
                        </MDButton>
                        <MDButton variant="text" color="error" size="small" onClick={() => handleDelete(row)}>
                          Remover
                        </MDButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhum parceiro encontrado.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{editing ? "Editar parceiro" : "Novo parceiro"}</DialogTitle>
        <DialogContent>
          <MDBox pt={1}>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <MDInput label="Descrição" fullWidth required value={form.description} onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Partners List ID" fullWidth value={form.partners_list_id} onChange={(e) => setForm((f) => ({ ...f, partners_list_id: parseInt(e.target.value, 10) || 1 }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput label="Documento" fullWidth value={form.document} onChange={(e) => setForm((f) => ({ ...f, document: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput label="Conta" fullWidth value={form.account} onChange={(e) => setForm((f) => ({ ...f, account: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput label="Agência" fullWidth value={form.branch} onChange={(e) => setForm((f) => ({ ...f, branch: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="% Cash-in" fullWidth value={form.percent_cashin} onChange={(e) => setForm((f) => ({ ...f, percent_cashin: parseFloat(e.target.value) || 0 }))} inputProps={{ step: 0.01 }} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="% Cash-out" fullWidth value={form.percent_cashout} onChange={(e) => setForm((f) => ({ ...f, percent_cashout: parseFloat(e.target.value) || 0 }))} inputProps={{ step: 0.01 }} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Fix Cash-in" fullWidth value={form.fixed_cash_in} onChange={(e) => setForm((f) => ({ ...f, fixed_cash_in: parseFloat(e.target.value) || 0 }))} inputProps={{ step: 0.01 }} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Fix Cash-out" fullWidth value={form.fixed_cash_out} onChange={(e) => setForm((f) => ({ ...f, fixed_cash_out: parseFloat(e.target.value) || 0 }))} inputProps={{ step: 0.01 }} />
              </Grid>
              <Grid item xs={12}>
                <MDBox display="flex" alignItems="center" gap={2}>
                  <Switch checked={form.active} onChange={(e) => setForm((f) => ({ ...f, active: e.target.checked }))} />
                  <MDTypography variant="body2">Ativo</MDTypography>
                </MDBox>
              </Grid>
            </Grid>
          </MDBox>
        </DialogContent>
        <DialogActions>
          <MDButton onClick={() => setOpen(false)}>Cancelar</MDButton>
          <MDButton variant="gradient" color="info" onClick={handleSave} disabled={submitting}>
            {submitting ? "Salvando..." : "Salvar"}
          </MDButton>
        </DialogActions>
      </Dialog>
      <Footer />
    </DashboardLayout>
  );
}
