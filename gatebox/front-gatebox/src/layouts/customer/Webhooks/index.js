/**
 * Webhooks do cliente - listar e cadastrar
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

export default function CustomerWebhooks() {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [open, setOpen] = useState(false);
  const [callbackUrl, setCallbackUrl] = useState("");
  const [webhookTypeId, setWebhookTypeId] = useState(1);
  const [submitting, setSubmitting] = useState(false);

  const load = () => {
    setLoading(true);
    customersApi.account.webhooks
      .list()
      .then((r) => setItems(Array.isArray(r?.items) ? r.items : []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const handleAdd = async () => {
    const url = callbackUrl.trim();
    if (!url) return;
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      setError("URL deve começar com http:// ou https://");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await customersApi.account.webhooks.create({
        callbackUrl: url,
        webhookTypeId,
      });
      setOpen(false);
      setCallbackUrl("");
      load();
    } catch (e) {
      setError(e.message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">
            Webhooks
          </MDTypography>
          <MDButton variant="gradient" color="info" onClick={() => setOpen(true)}>
            Cadastrar webhook
          </MDButton>
        </MDBox>
        {error && (
          <MDTypography color="error" mb={2}>
            {error}
          </MDTypography>
        )}
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={4}>
            <CircularProgress />
          </MDBox>
        ) : (
          <Card>
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>ID</TableCell>
                    <TableCell>URL de callback</TableCell>
                    <TableCell>Tipo</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{row.callbackUrl || row.callback_url || "-"}</TableCell>
                      <TableCell>{row.webhookTypeId ?? row.webhook_type_id ?? "-"}</TableCell>
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

      <Dialog open={open} onClose={() => { setOpen(false); setError(null); }} maxWidth="sm" fullWidth>
        <DialogTitle>Cadastrar webhook</DialogTitle>
        <DialogContent>
          <MDInput
            label="URL de callback"
            value={callbackUrl}
            onChange={(e) => setCallbackUrl(e.target.value)}
            fullWidth
            sx={{ mt: 1 }}
            placeholder="https://..."
          />
          <MDInput
            label="Tipo (ID)"
            type="number"
            value={webhookTypeId}
            onChange={(e) => setWebhookTypeId(Number(e.target.value) || 1)}
            fullWidth
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <MDButton onClick={() => setOpen(false)}>Cancelar</MDButton>
          <MDButton variant="gradient" color="info" onClick={handleAdd} disabled={submitting || !callbackUrl.trim()}>
            {submitting ? "Salvando..." : "Salvar"}
          </MDButton>
        </DialogActions>
      </Dialog>

      <Footer />
    </DashboardLayout>
  );
}
