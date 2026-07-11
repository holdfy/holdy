/**
 * Admin — Transações HoldFy
 * Lista pedidos APICash processados pelo Gatebox com nome, CPF/CNPJ, valor, data e rede.
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
import TablePagination from "@mui/material/TablePagination";
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import Tooltip from "@mui/material/Tooltip";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";

// ── helpers ───────────────────────────────────────────────────────────────────

function fmtBRL(v) {
  if (v == null) return "—";
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v));
}

function fmtDoc(doc) {
  if (!doc) return "—";
  const d = doc.replace(/\D/g, "");
  if (d.length === 11)
    return d.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, "$1.$2.$3-$4");
  if (d.length === 14)
    return d.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, "$1.$2.$3/$4-$5");
  return doc;
}

function fmtDate(v) {
  if (!v) return "—";
  return new Intl.DateTimeFormat("pt-BR", {
    day: "2-digit", month: "2-digit", year: "numeric",
    hour: "2-digit", minute: "2-digit", second: "2-digit",
  }).format(new Date(v));
}

function shortRef(ref) {
  if (!ref) return "—";
  const clean = ref.replace(/^order[_:]?/i, "");
  return clean.length > 12 ? clean.slice(0, 8) + "…" : clean;
}

function shortHash(hash) {
  if (!hash) return "—";
  return hash.length > 14 ? `${hash.slice(0, 6)}…${hash.slice(-6)}` : hash;
}

function stellarExplorerUrl(network, hash) {
  if (!hash) return null;
  const net = network === "mainnet" ? "public" : "testnet";
  return `https://stellar.expert/explorer/${net}/tx/${hash}`;
}

function NetworkBadge({ network }) {
  if (!network || network === "simulated")
    return <Chip label="Simulado" size="small" sx={{ bgcolor: "#6b7280", color: "#fff", fontWeight: 600, fontSize: 11 }} />;
  if (network === "testnet")
    return <Chip label="Testnet" size="small" sx={{ bgcolor: "#7c3aed", color: "#fff", fontWeight: 600, fontSize: 11 }} />;
  return <Chip label="Mainnet" size="small" sx={{ bgcolor: "#059669", color: "#fff", fontWeight: 600, fontSize: 11 }} />;
}

function StatusBadge({ status }) {
  const map = {
    "Concluído": { bg: "#059669", label: "Concluído" },
    "Aguardando": { bg: "#d97706", label: "Aguardando" },
    "Erro": { bg: "#dc2626", label: "Erro" },
    "Estornado": { bg: "#6b7280", label: "Estornado" },
    "Novo": { bg: "#2563eb", label: "Novo" },
    "Fila": { bg: "#0891b2", label: "Fila" },
  };
  const cfg = map[status] || { bg: "#6b7280", label: status || "—" };
  return <Chip label={cfg.label} size="small" sx={{ bgcolor: cfg.bg, color: "#fff", fontWeight: 600, fontSize: 11 }} />;
}

// ── component ─────────────────────────────────────────────────────────────────

export default function HoldfyTransactions() {
  const [items, setItems] = useState([]);
  const [total, setTotal] = useState(0);
  const [network, setNetwork] = useState("");
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(25);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const load = useCallback(() => {
    setLoading(true);
    setError(null);
    adminApi.holdfy
      .transactions({ limit: rowsPerPage, page: page + 1 })
      .then((r) => {
        setItems(r.data || []);
        setTotal(r.pagination?.total || 0);
        setNetwork(r.network || "");
      })
      .catch((e) => setError(e.message || "Erro ao carregar transações"))
      .finally(() => setLoading(false));
  }, [page, rowsPerPage]);

  useEffect(() => { load(); }, [load]);

  const handleChangePage = (_, newPage) => setPage(newPage);
  const handleChangeRowsPerPage = (e) => {
    setRowsPerPage(parseInt(e.target.value, 10));
    setPage(0);
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        {/* Cabeçalho */}
        <MDBox display="flex" justifyContent="space-between" alignItems="center" mb={3}>
          <MDBox>
            <MDTypography variant="h4" fontWeight="bold">
              Transações HoldFy
            </MDTypography>
            <MDTypography variant="body2" color="text" mt={0.5}>
              Pedidos APICash processados por este gateway
              {network && (
                <MDBox component="span" ml={1}>
                  <NetworkBadge network={network} />
                </MDBox>
              )}
            </MDTypography>
          </MDBox>
          <MDButton variant="outlined" color="info" size="small" onClick={load}>
            Atualizar
          </MDButton>
        </MDBox>

        {/* Tabela */}
        {loading ? (
          <MDBox display="flex" justifyContent="center" py={6}>
            <CircularProgress />
          </MDBox>
        ) : error ? (
          <MDBox py={4} textAlign="center">
            <MDTypography color="error" variant="body2">{error}</MDTypography>
            <MDButton variant="outlined" color="error" size="small" onClick={load} sx={{ mt: 1 }}>
              Tentar novamente
            </MDButton>
          </MDBox>
        ) : items.length === 0 ? (
          <MDBox py={6} textAlign="center">
            <MDTypography variant="body2" color="text">
              Nenhuma transação HoldFy encontrada neste gateway.
            </MDTypography>
          </MDBox>
        ) : (
          <Card>
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow sx={{ "& th": { fontWeight: 700, fontSize: 12, whiteSpace: "nowrap" } }}>
                    <TableCell>ID</TableCell>
                    <TableCell>Nome</TableCell>
                    <TableCell>CPF / CNPJ</TableCell>
                    <TableCell align="right">Valor</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell>Rede</TableCell>
                    <TableCell>Ref. Pedido</TableCell>
                    <TableCell>Data / Hora</TableCell>
                    <TableCell>Gateway</TableCell>
                    <TableCell>Gateway TX</TableCell>
                    <TableCell>Hash on-chain</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {items.map((row) => (
                    <TableRow
                      key={row.id}
                      hover
                      sx={{ "& td": { fontSize: 12, py: 0.8 } }}
                    >
                      <TableCell sx={{ fontFamily: "monospace", color: "#6b7280" }}>
                        {row.id ?? "—"}
                      </TableCell>

                      <TableCell sx={{ maxWidth: 160, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        <Tooltip title={row.name || ""} placement="top">
                          <span>{row.name || "—"}</span>
                        </Tooltip>
                      </TableCell>

                      <TableCell sx={{ fontFamily: "monospace", whiteSpace: "nowrap" }}>
                        {fmtDoc(row.document)}
                      </TableCell>

                      <TableCell align="right" sx={{ fontWeight: 600, whiteSpace: "nowrap" }}>
                        {fmtBRL(row.amount)}
                      </TableCell>

                      <TableCell>
                        <StatusBadge status={row.status} />
                      </TableCell>

                      <TableCell>
                        <NetworkBadge network={row.network} />
                      </TableCell>

                      <TableCell sx={{ fontFamily: "monospace", fontSize: 11 }}>
                        <Tooltip title={row.order_ref || ""} placement="top">
                          <span>{shortRef(row.order_ref)}</span>
                        </Tooltip>
                      </TableCell>

                      <TableCell sx={{ whiteSpace: "nowrap" }}>
                        {fmtDate(row.created_at)}
                      </TableCell>

                      <TableCell sx={{ color: "#6b7280", fontSize: 11 }}>
                        {row.gateway || "—"}
                      </TableCell>

                      <TableCell sx={{ fontFamily: "monospace", fontSize: 11 }}>
                        <Tooltip title={row.gateway_tx_id || ""} placement="top">
                          <span>{row.gateway_tx_id || "—"}</span>
                        </Tooltip>
                      </TableCell>

                      <TableCell sx={{ fontFamily: "monospace", fontSize: 11 }}>
                        {row.chain_tx_hash ? (
                          <Tooltip title={row.chain_tx_hash} placement="top">
                            <a
                              href={stellarExplorerUrl(row.network, row.chain_tx_hash)}
                              target="_blank"
                              rel="noopener noreferrer"
                              style={{ color: "#2563eb" }}
                            >
                              {shortHash(row.chain_tx_hash)}
                            </a>
                          </Tooltip>
                        ) : (
                          "—"
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>

            <TablePagination
              component="div"
              count={total}
              page={page}
              onPageChange={handleChangePage}
              rowsPerPage={rowsPerPage}
              onRowsPerPageChange={handleChangeRowsPerPage}
              rowsPerPageOptions={[10, 25, 50, 100]}
              labelRowsPerPage="Linhas:"
              labelDisplayedRows={({ from, to, count }) => `${from}–${to} de ${count}`}
            />
          </Card>
        )}
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
