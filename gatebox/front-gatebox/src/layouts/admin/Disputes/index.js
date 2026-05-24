/**
 * Admin - Disputas
 */

import { useState, useEffect, useCallback } from "react";
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
import Chip from "@mui/material/Chip";
import Select from "@mui/material/Select";
import MenuItem from "@mui/material/MenuItem";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import TextField from "@mui/material/TextField";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";

const STATUS_COLOR = {
  OPEN: "warning",
  RESOLVED: "success",
  CLOSED: "default",
};

const TYPE_LABEL = {
  INFRACTION: "Infração",
  REVERSAL: "Estorno",
  FRAUD: "Fraude",
};

export default function AdminDisputes() {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [statusFilter, setStatusFilter] = useState("");
  const [resolving, setResolving] = useState(null); // dispute being resolved
  const [resolution, setResolution] = useState("customer");
  const [notes, setNotes] = useState("");
  const [saving, setSaving] = useState(false);

  const load = useCallback(() => {
    setLoading(true);
    const params = statusFilter ? { status: statusFilter } : {};
    adminApi.disputes.list(params)
      .then((r) => setItems(r.items || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [statusFilter]);

  useEffect(() => { load(); }, [load]);

  const openResolve = (dispute) => {
    setResolving(dispute);
    setResolution("customer");
    setNotes("");
  };

  const handleResolve = async () => {
    if (!resolving) return;
    setSaving(true);
    try {
      await adminApi.disputes.resolve(resolving.id, resolution, notes);
      setResolving(null);
      load();
    } catch (e) {
      alert(`Erro: ${e.message}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox display="flex" alignItems="center" justifyContent="space-between" mb={3}>
          <MDTypography variant="h4" fontWeight="medium">Disputas</MDTypography>
          <FormControl size="small" sx={{ minWidth: 160 }}>
            <InputLabel>Status</InputLabel>
            <Select
              label="Status"
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
            >
              <MenuItem value="">Todos</MenuItem>
              <MenuItem value="OPEN">Abertas</MenuItem>
              <MenuItem value="RESOLVED">Resolvidas</MenuItem>
              <MenuItem value="CLOSED">Fechadas</MenuItem>
            </Select>
          </FormControl>
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
                    <TableCell>Status</TableCell>
                    <TableCell>Conta</TableCell>
                    <TableCell>Motivo</TableCell>
                    <TableCell>Abertura</TableCell>
                    <TableCell align="right">Ações</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow key={row.id}>
                      <TableCell>{row.id}</TableCell>
                      <TableCell>{TYPE_LABEL[row.type] || row.type}</TableCell>
                      <TableCell>
                        <Chip
                          label={row.status}
                          color={STATUS_COLOR[row.status] || "default"}
                          size="small"
                        />
                      </TableCell>
                      <TableCell>{row.account_id}</TableCell>
                      <TableCell sx={{ maxWidth: 220, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {row.reason}
                      </TableCell>
                      <TableCell>
                        {row.created_at ? new Date(row.created_at).toLocaleString("pt-BR") : "—"}
                      </TableCell>
                      <TableCell align="right">
                        {row.status === "OPEN" && (
                          <MDButton
                            variant="outlined"
                            color="warning"
                            size="small"
                            onClick={() => openResolve(row)}
                          >
                            Resolver
                          </MDButton>
                        )}
                        {row.status !== "OPEN" && (
                          <MDTypography variant="caption" color="secondary">
                            {row.resolution_notes ? `Resolvida: ${row.resolution_notes.slice(0, 30)}` : "Resolvida"}
                          </MDTypography>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            {items.length === 0 && (
              <MDBox p={3} textAlign="center">
                <MDTypography color="text">Nenhuma disputa encontrada.</MDTypography>
              </MDBox>
            )}
          </Card>
        )}
      </MDBox>

      {/* Resolve Dialog */}
      <Dialog open={!!resolving} onClose={() => setResolving(null)} maxWidth="sm" fullWidth>
        <DialogTitle>Resolver Disputa #{resolving?.id}</DialogTitle>
        <DialogContent>
          <MDTypography variant="body2" mb={2}>
            <strong>Motivo:</strong> {resolving?.reason}
          </MDTypography>
          <FormControl fullWidth sx={{ mb: 2 }} size="small">
            <InputLabel>Resolução</InputLabel>
            <Select
              label="Resolução"
              value={resolution}
              onChange={(e) => setResolution(e.target.value)}
            >
              <MenuItem value="customer">Favor do Cliente</MenuItem>
              <MenuItem value="platform">Favor da Plataforma</MenuItem>
            </Select>
          </FormControl>
          <TextField
            label="Observações"
            multiline
            rows={3}
            fullWidth
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            size="small"
          />
        </DialogContent>
        <DialogActions>
          <MDButton onClick={() => setResolving(null)} color="secondary">Cancelar</MDButton>
          <MDButton onClick={handleResolve} color="warning" disabled={saving}>
            {saving ? "Salvando..." : "Confirmar"}
          </MDButton>
        </DialogActions>
      </Dialog>

      <Footer />
    </DashboardLayout>
  );
}
