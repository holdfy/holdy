/**
 * Admin - Configurações
 */

import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import Switch from "@mui/material/Switch";
import Grid from "@mui/material/Grid";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

export default function AdminSettings() {
  const { showSuccess, showError } = useSnackbar();
  const [settings, setSettings] = useState(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);
  const [form, setForm] = useState({});

  useEffect(() => {
    adminApi.settings.get()
      .then((r) => {
        setSettings(r);
        setForm({
          max_transaction: r.max_transaction ?? 10000,
          min_transaction: r.min_transaction ?? 1,
          pix_enabled: r.pix_enabled ?? true,
          gateway_failover: r.gateway_failover ?? true,
          webhook_retry: r.webhook_retry ?? 3,
        });
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const handleSave = async (e) => {
    e.preventDefault();
    setSaving(true);
    try {
      await adminApi.settings.update(form);
      showSuccess("Configurações salvas");
      setSettings((s) => ({ ...s, ...form }));
    } catch (err) {
      showError(err.message || "Erro ao salvar");
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

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Configurações</MDTypography>
        {error ? (
          <MDTypography color="error">{error}</MDTypography>
        ) : (
          <Card sx={{ maxWidth: 600 }}>
            <MDBox p={3} component="form" onSubmit={handleSave}>
              <Grid container spacing={2}>
                <Grid item xs={12} md={6}>
                  <MDInput
                    type="number"
                    label="Transação máxima (R$)"
                    fullWidth
                    value={form.max_transaction ?? ""}
                    onChange={(e) => setForm((f) => ({ ...f, max_transaction: parseFloat(e.target.value) || 0 }))}
                    inputProps={{ step: 0.01, min: 0 }}
                  />
                </Grid>
                <Grid item xs={12} md={6}>
                  <MDInput
                    type="number"
                    label="Transação mínima (R$)"
                    fullWidth
                    value={form.min_transaction ?? ""}
                    onChange={(e) => setForm((f) => ({ ...f, min_transaction: parseFloat(e.target.value) || 0 }))}
                    inputProps={{ step: 0.01, min: 0 }}
                  />
                </Grid>
                <Grid item xs={12}>
                  <MDBox display="flex" alignItems="center" gap={2}>
                    <Switch checked={form.pix_enabled ?? true} onChange={(e) => setForm((f) => ({ ...f, pix_enabled: e.target.checked }))} />
                    <MDTypography variant="body2">PIX habilitado</MDTypography>
                  </MDBox>
                </Grid>
                <Grid item xs={12}>
                  <MDBox display="flex" alignItems="center" gap={2}>
                    <Switch checked={form.gateway_failover ?? true} onChange={(e) => setForm((f) => ({ ...f, gateway_failover: e.target.checked }))} />
                    <MDTypography variant="body2">Failover de gateway</MDTypography>
                  </MDBox>
                </Grid>
                <Grid item xs={12} md={6}>
                  <MDInput
                    type="number"
                    label="Tentativas de webhook"
                    fullWidth
                    value={form.webhook_retry ?? ""}
                    onChange={(e) => setForm((f) => ({ ...f, webhook_retry: parseInt(e.target.value, 10) || 0 }))}
                    inputProps={{ min: 1, max: 10 }}
                  />
                </Grid>
              </Grid>
              <MDButton variant="gradient" color="dark" type="submit" disabled={saving} sx={{ mt: 2 }}>
                {saving ? "Salvando..." : "Salvar"}
              </MDButton>
            </MDBox>
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
