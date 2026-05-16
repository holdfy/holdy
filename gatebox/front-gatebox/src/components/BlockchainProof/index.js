/**
 * Comprovante blockchain – exibe registro de ancoragem (tx_hash, block, link explorer).
 * Usar na tela de detalhe de transação PIX ou MED.
 * Usa GET /api/v1/anchor/audit com filtros entity_type e entity_id.
 */
import { useState, useEffect } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDButton from "components/MDButton";
import Icon from "@mui/material/Icon";
import CircularProgress from "@mui/material/CircularProgress";
import { anchorApi } from "services/api";

export default function BlockchainProof({ entityType, entityId }) {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (!entityType || !entityId) {
      setLoading(false);
      return;
    }
    let cancelled = false;
    anchorApi.proof(entityType, entityId)
      .then((proof) => {
        if (!cancelled) setData(proof);
      })
      .catch((e) => {
        if (!cancelled) setError(e.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, [entityType, entityId]);

  if (loading) {
    return (
      <MDBox display="flex" alignItems="center" gap={1}>
        <CircularProgress size={20} />
        <MDTypography variant="caption">Carregando comprovante blockchain...</MDTypography>
      </MDBox>
    );
  }

  if (error) {
    return (
      <MDTypography variant="caption" color="error">
        Erro ao carregar comprovante: {error}
      </MDTypography>
    );
  }

  if (!data) {
    return (
      <MDTypography variant="caption" color="text">
        Nenhum registro de ancoragem para esta entidade.
      </MDTypography>
    );
  }

  return (
    <MDBox>
      <MDTypography variant="caption" fontWeight="medium" color="text" display="block" mb={0.5}>
        Registro na blockchain
      </MDTypography>
      <MDBox display="flex" flexDirection="column" gap={0.5}>
        {data.tx_hash && (
          <MDTypography variant="caption" color="text">
            Tx: {String(data.tx_hash).slice(0, 10)}...{String(data.tx_hash).slice(-8)}
          </MDTypography>
        )}
        {data.block_number != null && (
          <MDTypography variant="caption" color="text">
            Bloco: {data.block_number}
          </MDTypography>
        )}
        {data.explorer_url && (
          <MDButton
            component="a"
            href={data.explorer_url}
            target="_blank"
            rel="noopener noreferrer"
            size="small"
            variant="outlined"
            color="info"
            sx={{ alignSelf: "flex-start" }}
          >
            <Icon sx={{ mr: 0.5 }}>open_in_new</Icon>
            Ver no explorer
          </MDButton>
        )}
        {data.dry_run && (
          <MDTypography variant="caption" color="warning">
            (dry-run)
          </MDTypography>
        )}
      </MDBox>
    </MDBox>
  );
}
