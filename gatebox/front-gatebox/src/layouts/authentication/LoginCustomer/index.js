/**
 * Login Customer - Acesso ao painel do cliente
 */

import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { Link } from "react-router-dom";
import Card from "@mui/material/Card";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
import MDButton from "components/MDButton";
import GateboxAuthLayout from "layouts/authentication/GateboxAuthLayout";
import { useAuth } from "context/AuthContext";
import { customersApi } from "services/api";

export default function LoginCustomer() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const { setToken } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const from = location.state?.from?.pathname || "/customer/dashboard";

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    if (!username.trim() || !password) {
      setError("Usuário e senha são obrigatórios");
      return;
    }
    setLoading(true);
    try {
      const res = await customersApi.auth.login(username.trim(), password);
      const token = res.accessToken || res.access_token || res.token;
      if (!token) {
        setError(res.message || "Resposta inválida do servidor");
        return;
      }
      setToken(token, "customer", {
        userId: res.userId ?? res.user?.id ?? res.user_id,
        username: res.username ?? res.user?.username ?? username,
      });
      navigate(from, { replace: true });
    } catch (err) {
      setError(err.message || "Credenciais inválidas");
    } finally {
      setLoading(false);
    }
  };

  return (
    <GateboxAuthLayout>
      <Card>
        <MDBox variant="gradient" bgColor="info" borderRadius="lg" mx={2} mt={2} p={2} mb={1} textAlign="center">
          <MDTypography variant="h4" fontWeight="medium" color="white" mt={1}>
            Gatebox
          </MDTypography>
          <MDTypography variant="body2" color="white" opacity={0.9}>
            Área do Cliente
          </MDTypography>
        </MDBox>
        <MDBox pt={4} pb={3} px={3}>
          <MDBox component="form" role="form" onSubmit={handleSubmit}>
            {error && (
              <MDBox mb={2}>
                <MDTypography variant="caption" color="error">
                  {error}
                </MDTypography>
              </MDBox>
            )}
            <MDBox mb={2}>
              <MDInput
                type="text"
                label="Usuário"
                fullWidth
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={loading}
              />
            </MDBox>
            <MDBox mb={2}>
              <MDInput
                type="password"
                label="Senha"
                fullWidth
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={loading}
              />
            </MDBox>
            <MDBox mt={4} mb={1}>
              <MDButton variant="gradient" color="info" fullWidth type="submit" disabled={loading}>
                {loading ? "Entrando..." : "Entrar"}
              </MDButton>
            </MDBox>
            <MDBox mt={3} mb={1} textAlign="center">
              <MDTypography variant="button" color="text">
                Não tem conta?{" "}
                <MDTypography component={Link} to="/register" variant="button" color="info" fontWeight="medium" textGradient>
                  Cadastre-se
                </MDTypography>
              </MDTypography>
            </MDBox>
            <MDBox mt={1} textAlign="center">
              <MDTypography variant="caption" color="text">
                <Link to="/admin/login" style={{ color: "inherit" }}>Admin</Link>
                {" · "}
                <Link to="/backoffice/login" style={{ color: "inherit" }}>Backoffice</Link>
              </MDTypography>
            </MDBox>
          </MDBox>
        </MDBox>
      </Card>
    </GateboxAuthLayout>
  );
}
