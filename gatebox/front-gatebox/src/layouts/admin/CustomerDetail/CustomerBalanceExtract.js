/**
 * Saldo e extrato por cliente (admin)
 * Requer endpoints: GET /admin/customers/:id/balance, GET /admin/customers/:id/extract
 * POST /admin/customers/:id/account para criar conta
 */

import { useState, useEffect } from "react";
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
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

const SUB_TYPE_LABELS = { 1: "PIX", 2: "DPIX", 3: "TTO", 4: "TPO", 5: "SMD" };

function formatCurrency(v) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(Number(v) || 0);
}

function subTypeLabel(v) {
  return v != null && v !== "" ? (SUB_TYPE_LABELS[v] ?? String(v)) : "-";
}

export default function CustomerBalanceExtract({ customerId, onRefresh }) {
  const { showSuccess, showError } = useSnackbar();
  const [balance, setBalance] = useState(null);
  const [extract, setExtract] = useState([]);
  const [loading, setLoading] = useState(true);
  const [available, setAvailable] = useState(false);
  const [creating, setCreating] = useState(false);

  const load = () => {
    setLoading(true);
    setAvailable(false);
    Promise.all([
      adminApi.customers.balance(customerId).catch(() => null),
      adminApi.customers.extract(customerId, { limit: 20 }).catch(() => null),
    ]).then(([bal, ext]) => {
      setAvailable(bal !== null || ext !== null);
      setBalance(bal);
      setExtract(Array.isArray(ext) ? ext : ext?.items || ext?.data || []);
    }).finally(() => setLoading(false));
  };

  useEffect(() => {
    load();
  }, [customerId]);

  const handleCreateAccount = async () => {
    setCreating(true);
    try {
      const res = await adminApi.customers.createAccount(customerId);
      showSuccess(res?.account_number ? `Conta ${res.account_number} criada` : "Conta criada");
      if (onRefresh) onRefresh();
      load();
    } catch (e) {
      showError(e.message || "Erro ao criar conta");
    } finally {
      setCreating(false);
    }
  };

  if (loading) {
    return (
      <Card sx={{ mt: 3, maxWidth: 600 }}>
        <MDBox p={3} display="flex" justifyContent="center"><CircularProgress size={24} /></MDBox>
      </Card>
    );
  }

  if (!available) {
    return (
      <Card sx={{ mt: 3, maxWidth: 600 }}>
        <MDBox p={3}>
          <MDTypography variant="button" color="text">Saldo e extrato</MDTypography>
          <MDTypography variant="caption" color="text" display="block" mt={1}>
            Cliente sem conta vinculada ou endpoints indisponíveis.
          </MDTypography>
          <MDButton variant="gradient" color="success" size="small" sx={{ mt: 2 }} onClick={handleCreateAccount} disabled={creating}>
            {creating ? "Criando..." : "Criar conta"}
          </MDButton>
        </MDBox>
      </Card>
    );
  }

  const bal = balance?.balance ?? balance?.availableBalance ?? balance?.available ?? balance;
  const med = balance?.preventiveBlock ?? balance?.med ?? 0;
  const avail = balance?.availableBalance ?? balance?.available ?? bal;

  return (
    <Card sx={{ mt: 3, maxWidth: 600 }}>
      <MDBox p={3}>
        <MDTypography variant="h6" fontWeight="medium" mb={2}>Saldo e extrato</MDTypography>
        {balance != null && (
          <MDBox mb={3}>
            <MDBox display="flex" gap={3} flexWrap="wrap">
              <MDBox>
                <MDTypography variant="button" color="text">Saldo</MDTypography>
                <MDTypography variant="h6">{formatCurrency(bal)}</MDTypography>
              </MDBox>
              {med != null && Number(med) !== 0 && (
                <MDBox>
                  <MDTypography variant="button" color="text">MED</MDTypography>
                  <MDTypography variant="body1">{formatCurrency(med)}</MDTypography>
                </MDBox>
              )}
              {avail != null && (
                <MDBox>
                  <MDTypography variant="button" color="text">Disponível</MDTypography>
                  <MDTypography variant="body1">{formatCurrency(avail)}</MDTypography>
                </MDBox>
              )}
            </MDBox>
          </MDBox>
        )}
        {extract.length > 0 && (
          <TableContainer>
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Data</TableCell>
                  <TableCell>Descrição</TableCell>
                  <TableCell>Tipo</TableCell>
                  <TableCell align="right">Valor</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {extract.map((row, i) => (
                  <TableRow key={row.id || i}>
                    <TableCell>{row.created_at?.slice?.(0, 16) || "-"}</TableCell>
                    <TableCell>{row.description || "-"}</TableCell>
                    <TableCell>{subTypeLabel(row.sub_type_transaction_id ?? row.sub_type) || row.type || "-"}</TableCell>
                    <TableCell align="right" sx={{ color: Number(row.amount) >= 0 ? "success.main" : "error.main" }}>
                      {formatCurrency(row.amount)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        )}
        {balance == null && extract.length === 0 && (
          <MDTypography variant="caption" color="text">Nenhum dado disponível.</MDTypography>
        )}
      </MDBox>
    </Card>
  );
}
