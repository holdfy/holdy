import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Paper, Grid, TextField, Button, Stack, Snackbar, Skeleton,
} from "@mui/material";
import AddCardIcon from "@mui/icons-material/AddCard";
import { adminApi } from "../api";

function fmtBrlx(v) {
  const n = parseFloat(v ?? "0");
  return n.toLocaleString("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 7 });
}

function Field({ label, value, mono }) {
  return (
    <Box mb={1.5}>
      <Typography variant="caption" color="text.secondary" display="block">
        {label}
      </Typography>
      <Typography variant="body2" sx={mono ? { fontFamily: "monospace", wordBreak: "break-all" } : undefined}>
        {value ?? "—"}
      </Typography>
    </Box>
  );
}

export default function Carteira() {
  const queryClient = useQueryClient();
  const [amount, setAmount] = useState("1000000000");
  const [toast, setToast] = useState(null);

  const { data: devStatus, isLoading: statusLoading } = useQuery({
    queryKey: ["dev-status"],
    queryFn: adminApi.devStatus,
  });

  const { data: wallet, isLoading, error } = useQuery({
    queryKey: ["dev-wallet"],
    queryFn: adminApi.devWallet,
    enabled: !!devStatus?.enabled,
  });

  const mintMutation = useMutation({
    mutationFn: (amt) => adminApi.devWalletMint(amt),
    onSuccess: (resp) => {
      setToast({ severity: "success", message: `Mint ok — novo saldo: ${fmtBrlx(resp.balance_brlx)} BRLx` });
      queryClient.invalidateQueries({ queryKey: ["dev-wallet"] });
    },
    onError: (err) => setToast({ severity: "error", message: err.message }),
  });

  if (statusLoading) return null;

  if (!devStatus?.enabled) {
    return (
      <Box>
        <Typography variant="h5" fontWeight={700} mb={3}>
          Carteira TestNet
        </Typography>
        <Alert severity="warning">
          Página desabilitada — rede atual é <b>{devStatus?.network}</b>. Só funciona em testnet.
        </Alert>
      </Box>
    );
  }

  const handleMint = () => {
    const n = parseFloat(amount.replace(",", "."));
    if (!n || n <= 0) {
      setToast({ severity: "error", message: "Informe um valor positivo." });
      return;
    }
    mintMutation.mutate(n);
  };

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={1}>
        Carteira TestNet
      </Typography>
      <Alert severity="info" sx={{ mb: 3 }}>
        Saldo de BRLx (token de teste Soroban, sem valor real) da carteira usada como comprador
        em todos os pedidos locais. Mint aqui pra nunca faltar saldo ao forçar pagamentos em{" "}
        <b>Dev TestNet</b>.
      </Alert>
      {error && <Alert severity="error" sx={{ mb: 2 }}>Erro: {error.message}</Alert>}

      <Grid container spacing={3}>
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="subtitle1" fontWeight={700} mb={2}>
              Dados da conta
            </Typography>
            {isLoading ? (
              <Skeleton variant="rounded" height={160} />
            ) : (
              <>
                <Field label="Rede" value={wallet?.network} />
                <Field label="Endereço do comprador (buyer)" value={wallet?.buyer_address} mono />
                <Field label="Endereço do issuer (deployer)" value={wallet?.deployer_address} mono />
                <Field label="Contrato do token BRLx" value={wallet?.token_contract_id} mono />
              </>
            )}
          </Paper>
        </Grid>

        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="subtitle1" fontWeight={700} mb={2}>
              Saldo atual
            </Typography>
            {isLoading ? (
              <Skeleton variant="rounded" height={60} />
            ) : (
              <Typography variant="h4" fontWeight={700} color="primary.main" mb={3}>
                {fmtBrlx(wallet?.balance_brlx)} BRLx
              </Typography>
            )}
            <Typography variant="subtitle2" fontWeight={700} mb={1}>
              Adicionar saldo (mint)
            </Typography>
            <Stack direction="row" spacing={2} alignItems="center">
              <TextField
                label="Valor em BRLx"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                size="small"
                fullWidth
              />
              <Button
                variant="contained"
                startIcon={<AddCardIcon />}
                disabled={mintMutation.isPending}
                onClick={handleMint}
                sx={{ whiteSpace: "nowrap" }}
              >
                {mintMutation.isPending ? "Mintando…" : "Mintar"}
              </Button>
            </Stack>
          </Paper>
        </Grid>
      </Grid>

      <Snackbar
        open={!!toast}
        autoHideDuration={5000}
        onClose={() => setToast(null)}
        message={toast?.message}
      />
    </Box>
  );
}
