import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Box, Paper, Typography, TextField, Button, Alert } from "@mui/material";
import LockIcon from "@mui/icons-material/Lock";
import { adminKeyStore, adminApi } from "../api";

export default function Login() {
  const [apiKey, setApiKey] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const handleLogin = async () => {
    if (!apiKey.trim()) return;
    setLoading(true);
    setError("");
    adminKeyStore.set(apiKey.trim());
    try {
      await adminApi.dashboard();
      navigate("/dashboard");
    } catch (e) {
      adminKeyStore.clear();
      setError(e.status === 401 || e.status === 403 ? "Chave inválida." : "Erro ao conectar. Verifique se o servidor está rodando na porta 3001.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box
      sx={{
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        bgcolor: "grey.100",
      }}
    >
      <Paper elevation={3} sx={{ p: 4, width: 380 }}>
        <Box textAlign="center" mb={3}>
          <LockIcon sx={{ fontSize: 48, color: "primary.main" }} />
          <Typography variant="h5" fontWeight={700} mt={1}>
            Holdfy Admin
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Insira a API Key de administrador
          </Typography>
        </Box>
        {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}
        <TextField
          fullWidth
          label="API Key"
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleLogin()}
          sx={{ mb: 2 }}
        />
        <Button
          fullWidth
          variant="contained"
          size="large"
          onClick={handleLogin}
          disabled={loading || !apiKey.trim()}
        >
          {loading ? "Verificando..." : "Entrar"}
        </Button>
      </Paper>
    </Box>
  );
}
