/**
 * Admin - Detalhe do cliente (visualizar + editar)
 */

import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import MDInput from "components/MDInput";
import Card from "@mui/material/Card";
import CircularProgress from "@mui/material/CircularProgress";
import Icon from "@mui/material/Icon";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import Grid from "@mui/material/Grid";
import Switch from "@mui/material/Switch";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";
import CustomerBalanceExtract from "./CustomerBalanceExtract";

export default function AdminCustomerDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { showSuccess, showError } = useSnackbar();
  const [customer, setCustomer] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [approving, setApproving] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [form, setForm] = useState({});
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    adminApi.customers.get(id)
      .then(setCustomer)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  const handleApproveKyc = async () => {
    if (!window.confirm("Aprovar KYC deste cliente?")) return;
    setApproving(true);
    try {
      await adminApi.customers.approveKyc(id);
      showSuccess("KYC aprovado");
      setCustomer((c) => (c ? { ...c, customer_status_id: 1 } : null));
    } catch (e) {
      showError(e.message);
    } finally {
      setApproving(false);
    }
  };

  const openEdit = () => {
    setForm({
      full_name: customer.full_name ?? customer.name ?? "",
      social_name: customer.social_name ?? "",
      document_number: customer.document_number ?? "",
      email: customer.email ?? "",
      phone_number: customer.phone_number ?? "",
      responsible_name: customer.responsible_name ?? "",
      type_person_id: customer.type_person_id ?? 1,
      company_id: customer.company_id ?? 1,
      customer_status_id: customer.customer_status_id ?? 1,
      is_politically_exposed_person: customer.is_politically_exposed_person ?? false,
      birth_date: customer.birth_date ?? "",
    });
    setEditOpen(true);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await adminApi.customers.update(id, { ...customer, ...form });
      showSuccess("Cliente atualizado");
      setCustomer((c) => (c ? { ...c, ...form } : null));
      setEditOpen(false);
    } catch (e) {
      showError(e.message || "Erro ao salvar");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <DashboardLayout>
        <DashboardNavbar />
        <MDBox py={3} display="flex" justifyContent="center"><CircularProgress /></MDBox>
        <Footer />
      </DashboardLayout>
    );
  }

  if (error || !customer) {
    return (
      <DashboardLayout>
        <DashboardNavbar />
        <MDBox py={3}>
          <MDTypography color="error">{error || "Cliente não encontrado"}</MDTypography>
          <MDButton variant="text" color="info" onClick={() => navigate("/admin/customers")} sx={{ mt: 2 }}>
            Voltar
          </MDButton>
        </MDBox>
        <Footer />
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" alignItems="center" gap={2} mb={3}>
          <MDButton variant="outlined" color="dark" size="small" onClick={() => navigate("/admin/customers")}>
            <Icon sx={{ mr: 0.5 }}>arrow_back</Icon> Voltar
          </MDButton>
          <MDTypography variant="h4" fontWeight="medium">Cliente #{customer.id}</MDTypography>
          <MDButton variant="gradient" color="info" size="small" onClick={openEdit}>
            Editar
          </MDButton>
        </MDBox>

        <Card sx={{ maxWidth: 600 }}>
          <MDBox p={3}>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Nome</MDTypography>
              <MDTypography variant="h6">{customer.full_name || customer.name || customer.username || "-"}</MDTypography>
            </MDBox>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Email</MDTypography>
              <MDTypography variant="body1">{customer.email || "-"}</MDTypography>
            </MDBox>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Documento</MDTypography>
              <MDTypography variant="body1">{customer.document_number || "-"}</MDTypography>
            </MDBox>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Telefone</MDTypography>
              <MDTypography variant="body1">{customer.phone_number || "-"}</MDTypography>
            </MDBox>
            <MDBox mb={2}>
              <MDTypography variant="button" color="text">Status</MDTypography>
              <MDTypography variant="body1">{customer.customer_status_id ?? customer.status ?? "-"}</MDTypography>
            </MDBox>
            {customer.customer_status_id === 4 && (
              <MDButton variant="gradient" color="success" onClick={handleApproveKyc} disabled={approving}>
                {approving ? "Aprovando..." : "Aprovar KYC"}
              </MDButton>
            )}
          </MDBox>
        </Card>

        <CustomerBalanceExtract customerId={id} />
      </MDBox>

      <Dialog open={editOpen} onClose={() => setEditOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Editar cliente</DialogTitle>
        <DialogContent>
          <MDBox pt={1}>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <MDInput label="Nome completo" fullWidth value={form.full_name ?? ""} onChange={(e) => setForm((f) => ({ ...f, full_name: e.target.value }))} />
              </Grid>
              <Grid item xs={12}>
                <MDInput label="Nome social" fullWidth value={form.social_name ?? ""} onChange={(e) => setForm((f) => ({ ...f, social_name: e.target.value }))} />
              </Grid>
              <Grid item xs={12} md={6}>
                <MDInput label="Documento" fullWidth value={form.document_number ?? ""} onChange={(e) => setForm((f) => ({ ...f, document_number: e.target.value }))} />
              </Grid>
              <Grid item xs={12} md={6}>
                <MDInput label="Email" type="email" fullWidth value={form.email ?? ""} onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))} />
              </Grid>
              <Grid item xs={12} md={6}>
                <MDInput label="Telefone" fullWidth value={form.phone_number ?? ""} onChange={(e) => setForm((f) => ({ ...f, phone_number: e.target.value }))} />
              </Grid>
              <Grid item xs={12} md={6}>
                <MDInput label="Responsável" fullWidth value={form.responsible_name ?? ""} onChange={(e) => setForm((f) => ({ ...f, responsible_name: e.target.value }))} />
              </Grid>
              <Grid item xs={12}>
                <MDBox display="flex" alignItems="center" gap={2}>
                  <Switch checked={form.is_politically_exposed_person ?? false} onChange={(e) => setForm((f) => ({ ...f, is_politically_exposed_person: e.target.checked }))} />
                  <MDTypography variant="body2">PEP (Pessoa politicamente exposta)</MDTypography>
                </MDBox>
              </Grid>
            </Grid>
          </MDBox>
        </DialogContent>
        <DialogActions>
          <MDButton onClick={() => setEditOpen(false)}>Cancelar</MDButton>
          <MDButton variant="gradient" color="info" onClick={handleSave} disabled={saving}>
            {saving ? "Salvando..." : "Salvar"}
          </MDButton>
        </DialogActions>
      </Dialog>
      <Footer />
    </DashboardLayout>
  );
}
