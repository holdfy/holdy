/**
 * Admin - Webhooks (CRUD)
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
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

const defaultWebhook = {
  callback_url: "",
  username: "",
  password: "",
  api_key: "",
  webhook_type_id: 1,
  account_id: 1,
};

export default function AdminWebhooks() {
  const { showSuccess, showError } = useSnackbar();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState(null);
  const [form, setForm] = useState({ ...defaultWebhook });
  const [submitting, setSubmitting] = useState(false);
  const [testing, setTesting] = useState(null);

  const load = () => {
    setLoading(true);
    adminApi.webhooks.list({ page: 1, limit: 50 })
      .then((r) => setItems(Array.isArray(r) ? r : r.items || r.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const openCreate = () => {
    setEditing(null);
    setForm({ ...defaultWebhook });
    setOpen(true);
  };

  const openEdit = (row) => {
    setEditing(row);
    setForm({
      callback_url: row.callback_url ?? "",
      username: row.username ?? "",
      password: row.password ?? "",
      api_key: row.api_key ?? "",
      webhook_type_id: row.webhook_type_id ?? 1,
      account_id: row.account_id ?? 1,
    });
    setOpen(true);
  };

  const handleSave = async () => {
    if (!form.callback_url?.trim()) {
      showError("URL de callback é obrigatória");
      return;
    }
    setSubmitting(true);
    try {
      const payload = { ...form };
      if (editing) {
        await adminApi.webhooks.update(editing.id, payload);
        showSuccess("Webhook atualizado");
      } else {
        await adminApi.webhooks.create(payload);
        showSuccess("Webhook criado");
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
    if (!window.confirm(`Remover webhook "${row.callback_url || row.id}"?`)) return;
    try {
      await adminApi.webhooks.delete(row.id);
      showSuccess("Webhook removido");
      load();
    } catch (e) {
      showError(e.message || "Erro ao remover");
    }
  };

  const handleTest = async (id) => {
    setTesting(id);
    try {
      await adminApi.webhooks.test(id);
      showSuccess("Teste enviado");
    } catch (e) {
      showError(e.message || "Erro no teste");
    } finally {
      setTesting(null);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Webhooks</MDTypography>
          <MDButton variant="gradient" color="info" onClick={openCreate}>
            Novo webhook
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
                    <TableCell>URL</TableCell>
                    <TableCell>Tipo</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell sx={{ maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis" }}>{row.callback_url || row.url || "-"}</TableCell>
                      <TableCell>{row.webhook_type_id ?? row.type ?? "-"}</TableCell>
                      <TableCell align="right">
                        <MDButton variant="text" color="secondary" size="small" onClick={() => handleTest(row.id)} disabled={testing === row.id} sx={{ mr: 1 }}>
                          {testing === row.id ? "..." : "Testar"}
                        </MDButton>
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
                <MDTypography color="text">Nenhum webhook configurado.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{editing ? "Editar webhook" : "Novo webhook"}</DialogTitle>
        <DialogContent>
          <MDBox pt={1}>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <MDInput label="URL de callback" fullWidth required value={form.callback_url} onChange={(e) => setForm((f) => ({ ...f, callback_url: e.target.value }))} placeholder="https://..." />
              </Grid>
              <Grid item xs={6}>
                <MDInput label="Usuário" fullWidth value={form.username} onChange={(e) => setForm((f) => ({ ...f, username: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="password" label="Senha" fullWidth value={form.password} onChange={(e) => setForm((f) => ({ ...f, password: e.target.value }))} />
              </Grid>
              <Grid item xs={12}>
                <MDInput label="API Key" fullWidth value={form.api_key} onChange={(e) => setForm((f) => ({ ...f, api_key: e.target.value }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Tipo (webhook_type_id)" fullWidth value={form.webhook_type_id} onChange={(e) => setForm((f) => ({ ...f, webhook_type_id: parseInt(e.target.value, 10) || 1 }))} />
              </Grid>
              <Grid item xs={6}>
                <MDInput type="number" label="Conta (account_id)" fullWidth value={form.account_id} onChange={(e) => setForm((f) => ({ ...f, account_id: parseInt(e.target.value, 10) || 1 }))} />
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
