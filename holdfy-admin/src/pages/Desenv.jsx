import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Box, Typography, Alert, Chip, Button, Snackbar, Stack, TextField,
} from "@mui/material";
import BoltIcon from "@mui/icons-material/Bolt";
import LocalShippingIcon from "@mui/icons-material/LocalShipping";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import GavelIcon from "@mui/icons-material/Gavel";
import { DataGrid } from "@mui/x-data-grid";
import { adminApi } from "../api";

const STATUS_COLORS = {
  draft: "default",
  pending_funding: "warning",
  funded: "info",
  in_custody: "info",
  completed: "success",
  cancelled: "default",
  failed: "error",
};

// Próximo passo do ciclo de vida a partir de cada status — o botão avança um degrau
// por clique e muda de nome/função conforme o pedido evolui.
const NEXT_STEP = {
  pending_funding: {
    label: "Forçar pagamento",
    icon: <BoltIcon />,
    call: (id) => adminApi.devSettleOrder(id),
  },
  in_custody: {
    label: "Forçar entrega",
    icon: <LocalShippingIcon />,
    call: (id) => adminApi.devReleaseOrder(id),
  },
};

export default function Desenv() {
  const queryClient = useQueryClient();
  const [toast, setToast] = useState(null);
  // advanceMutation é uma única instância compartilhada por todas as linhas da grid —
  // `isPending`/`variables` só refletem a ÚLTIMA chamada de mutate(), então não dá pra usá-los
  // pra saber se UMA linha específica está em voo quando várias são disparadas em paralelo.
  // Rastreamos os pedidos pendentes à parte, via onMutate/onSettled (que disparam por chamada).
  const [pendingIds, setPendingIds] = useState(() => new Set());
  const [proposalIdInput, setProposalIdInput] = useState("");
  const [buyerIdInput, setBuyerIdInput] = useState("");

  const { data: devStatus, isLoading: statusLoading } = useQuery({
    queryKey: ["dev-status"],
    queryFn: adminApi.devStatus,
  });

  const { data, isLoading, error } = useQuery({
    queryKey: ["dev-orders"],
    queryFn: () => adminApi.listOrders(),
    refetchInterval: 15_000,
    enabled: !!devStatus?.enabled,
  });

  const advanceMutation = useMutation({
    mutationFn: ({ id, status }) => NEXT_STEP[status].call(id),
    onMutate: ({ id }) => {
      setPendingIds((prev) => new Set(prev).add(id));
    },
    onSettled: (_data, _err, { id }) => {
      setPendingIds((prev) => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
    },
    onSuccess: (resp, { id }) => {
      const nextStatus = resp.status ?? "ok";
      setToast({ severity: "success", message: `Pedido ${id.slice(0, 8)}… → ${nextStatus}` });
      queryClient.invalidateQueries({ queryKey: ["dev-orders"] });
    },
    onError: (err) => {
      setToast({ severity: "error", message: err.message });
    },
  });

  // Aceita uma proposta pendente ignorando bloqueio anti-fraude (velocidade/volume/CPF) —
  // destrava testes quando a política de risco bloqueia legitimamente um comprador de teste.
  const forceAcceptMutation = useMutation({
    mutationFn: () => adminApi.devForceAcceptProposal(proposalIdInput.trim(), buyerIdInput.trim()),
    onSuccess: (resp) => {
      setToast({ severity: "success", message: `Proposta aceita → pedido ${resp.order_id?.slice(0, 8)}…` });
      setProposalIdInput("");
      queryClient.invalidateQueries({ queryKey: ["dev-orders"] });
    },
    onError: (err) => {
      setToast({ severity: "error", message: err.message });
    },
  });

  if (statusLoading) return null;

  if (!devStatus?.enabled) {
    return (
      <Box>
        <Typography variant="h5" fontWeight={700} mb={3}>
          Dev TestNet
        </Typography>
        <Alert severity="warning">
          Página de desenvolvimento desabilitada — rede atual é <b>{devStatus?.network}</b>.
          Só funciona em testnet.
        </Alert>
      </Box>
    );
  }

  const rows = (data?.orders ?? [])
    .slice()
    .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
    .map((o, i) => ({ id: i, ...o }));

  const columns = [
    { field: "order_id", headerName: "Pedido", width: 280, renderCell: (p) => p.value },
    {
      field: "amount_minor",
      headerName: "Valor (R$)",
      width: 130,
      valueFormatter: (v) => parseFloat(v).toLocaleString("pt-BR", { minimumFractionDigits: 2 }),
    },
    {
      field: "status",
      headerName: "Status",
      width: 150,
      renderCell: (p) => <Chip label={p.value} color={STATUS_COLORS[p.value] ?? "default"} size="small" />,
    },
    {
      field: "created_at",
      headerName: "Criado em",
      width: 190,
      valueFormatter: (v) => (v ? new Date(v).toLocaleString("pt-BR") : "—"),
    },
    {
      field: "action",
      headerName: "Ação",
      width: 200,
      sortable: false,
      renderCell: (p) => {
        const step = NEXT_STEP[p.row.status];
        if (!step) {
          return (
            <Button size="small" variant="outlined" disabled startIcon={<CheckCircleIcon />}>
              {p.row.status === "completed" ? "Concluído" : "—"}
            </Button>
          );
        }
        const isRowPending = pendingIds.has(p.row.order_id);
        return (
          <Button
            size="small"
            variant="contained"
            color="warning"
            startIcon={step.icon}
            disabled={isRowPending}
            onClick={() => advanceMutation.mutate({ id: p.row.order_id, status: p.row.status })}
          >
            {isRowPending ? "Aguarde…" : step.label}
          </Button>
        );
      },
    },
  ];

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={1}>
        Dev TestNet
      </Typography>
      <Alert severity="info" sx={{ mb: 2 }}>
        Rede: <b>{devStatus.network}</b> — só disponível fora de mainnet. Cada clique avança o
        pedido um passo no ciclo de vida: <b>pending_funding → in_custody</b> (força pagamento)
        → <b>completed</b> (força entrega/liberação de custódia, dispara off-ramp PIX ao vendedor
        se ele tiver chave cadastrada).
      </Alert>
      {error && <Alert severity="error" sx={{ mb: 2 }}>Erro: {error.message}</Alert>}

      <Alert severity="warning" sx={{ mb: 1 }}>
        Forçar aceite de proposta — ignora bloqueio anti-fraude (velocidade/volume/CPF).
        Use pra destravar um comprador de teste que a política de risco bloqueou de propósito.
      </Alert>
      <Stack direction="row" spacing={1.5} alignItems="center" mb={3} flexWrap="wrap" rowGap={1.5}>
        <TextField
          size="small"
          label="ID da proposta"
          value={proposalIdInput}
          onChange={(e) => setProposalIdInput(e.target.value)}
          sx={{ minWidth: 320 }}
        />
        <TextField
          size="small"
          label="ID do comprador (se proposta aberta)"
          value={buyerIdInput}
          onChange={(e) => setBuyerIdInput(e.target.value)}
          sx={{ minWidth: 320 }}
        />
        <Button
          variant="contained"
          color="warning"
          startIcon={<GavelIcon />}
          disabled={!proposalIdInput.trim() || forceAcceptMutation.isPending}
          onClick={() => forceAcceptMutation.mutate()}
        >
          {forceAcceptMutation.isPending ? "Forçando…" : "Forçar aceite"}
        </Button>
      </Stack>

      <DataGrid
        rows={rows}
        columns={columns}
        loading={isLoading}
        autoHeight
        pageSizeOptions={[25, 50, 100]}
        initialState={{ pagination: { paginationModel: { pageSize: 25 } } }}
        disableRowSelectionOnClick
      />
      <Snackbar
        open={!!toast}
        autoHideDuration={4000}
        onClose={() => setToast(null)}
        message={toast?.message}
      />
    </Box>
  );
}
