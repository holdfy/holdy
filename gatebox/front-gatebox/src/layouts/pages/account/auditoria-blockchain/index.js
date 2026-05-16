/**
 * Tela de auditoria blockchain – lista ancoragens com filtro por período e export.
 * Requer autenticação admin (token no header).
 */
import { useState, useEffect, useCallback } from "react";
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
import Icon from "@mui/material/Icon";
import CircularProgress from "@mui/material/CircularProgress";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { anchorApi } from "services/api";

function AuditoriaBlockchain() {
  const [items, setItems] = useState([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [entityType, setEntityType] = useState("");
  const [limit] = useState(50);
  const [offset, setOffset] = useState(0);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const params = { limit, offset };
      if (from) params.from = `${from}T00:00:00Z`;
      if (to) params.to = `${to}T23:59:59Z`;
      if (entityType) params.entity_type = entityType;
      const json = await anchorApi.audit(params);
      setItems(json.items || []);
      setTotal(json.total ?? 0);
    } catch (e) {
      setError(e.message);
      setItems([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  }, [from, to, entityType, limit, offset]);

  useEffect(() => {
    load();
  }, [load]);

  const exportCsv = () => {
    const headers = ["id", "idempotency_key", "entity_type", "entity_id", "tx_hash", "block_number", "anchored_at", "explorer_url"];
    const rows = items.map((r) =>
      [
        r.id,
        r.idempotency_key,
        r.entity_type,
        r.entity_id,
        r.tx_hash || "",
        r.block_number ?? "",
        r.anchored_at || "",
        r.explorer_url || "",
      ].join(",")
    );
    const csv = [headers.join(","), ...rows].join("\n");
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = `auditoria-blockchain-${new Date().toISOString().slice(0, 10)}.csv`;
    link.click();
    URL.revokeObjectURL(link.href);
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDBox mb={3}>
          <MDTypography variant="h4" fontWeight="medium">
            Auditoria blockchain
          </MDTypography>
          <MDTypography variant="body2" color="text">
            Listagem de eventos ancorados (PIX, MED). Filtre por período e exporte em CSV.
          </MDTypography>
        </MDBox>

        <Card sx={{ overflow: "visible" }}>
          <MDBox p={2} display="flex" flexWrap="wrap" gap={2} alignItems="center">
            <MDInput
              type="date"
              label="De"
              value={from}
              onChange={(e) => setFrom(e.target.value)}
              variant="outlined"
              size="small"
            />
            <MDInput
              type="date"
              label="Até"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              variant="outlined"
              size="small"
            />
            <MDInput
              label="Tipo (pix_tx, med)"
              value={entityType}
              onChange={(e) => setEntityType(e.target.value)}
              variant="outlined"
              size="small"
              sx={{ minWidth: 120 }}
            />
            <MDButton variant="gradient" color="info" onClick={load} disabled={loading}>
              {loading ? <CircularProgress size={20} color="inherit" /> : "Filtrar"}
            </MDButton>
            <MDButton variant="outlined" color="dark" onClick={exportCsv} disabled={items.length === 0}>
              <Icon sx={{ mr: 0.5 }}>download</Icon>
              Exportar CSV
            </MDButton>
          </MDBox>

          {error && (
            <MDBox px={2} pb={1}>
              <MDTypography variant="caption" color="error">
                {error}
              </MDTypography>
            </MDBox>
          )}

          <TableContainer sx={{ maxHeight: 500 }}>
            <Table stickyHeader size="small">
              <TableHead>
                <TableRow>
                  <TableCell>ID</TableCell>
                  <TableCell>Entidade</TableCell>
                  <TableCell>Tx Hash</TableCell>
                  <TableCell>Bloco</TableCell>
                  <TableCell>Ancorado em</TableCell>
                  <TableCell>Dry-run</TableCell>
                  <TableCell>Explorer</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {items.map((row) => (
                  <TableRow key={row.id}>
                    <TableCell>{row.id}</TableCell>
                    <TableCell>
                      {row.entity_type} / {row.entity_id}
                    </TableCell>
                    <TableCell>{row.tx_hash ? `${String(row.tx_hash).slice(0, 12)}...` : "-"}</TableCell>
                    <TableCell>{row.block_number ?? "-"}</TableCell>
                    <TableCell>{row.anchored_at || "-"}</TableCell>
                    <TableCell>{row.dry_run ? "Sim" : "Não"}</TableCell>
                    <TableCell>
                      {row.explorer_url ? (
                        <MDButton
                          component="a"
                          href={row.explorer_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          size="small"
                          color="info"
                        >
                          Ver
                        </MDButton>
                      ) : (
                        "-"
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
          <MDBox p={2}>
            <MDTypography variant="caption" color="text">
              Total: {total} | Exibindo {items.length}
            </MDTypography>
            {offset > 0 && (
              <MDButton size="small" onClick={() => setOffset((o) => Math.max(0, o - limit))} sx={{ ml: 1 }}>
                Anterior
              </MDButton>
            )}
            {offset + items.length < total && (
              <MDButton size="small" onClick={() => setOffset((o) => o + limit)} sx={{ ml: 1 }}>
                Próxima
              </MDButton>
            )}
          </MDBox>
        </Card>
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}

export default AuditoriaBlockchain;
