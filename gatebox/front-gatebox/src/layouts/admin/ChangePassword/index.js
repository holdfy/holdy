/**
 * Admin - Trocar senha
 */

import { useState } from "react";
import MDBox from "components/MDBox";
import MDTypography from "components/MDTypography";
import MDInput from "components/MDInput";
import MDButton from "components/MDButton";
import Card from "@mui/material/Card";
import DashboardLayout from "examples/LayoutContainers/DashboardLayout";
import DashboardNavbar from "examples/Navbars/DashboardNavbar";
import Footer from "examples/Footer";
import { adminApi } from "services/api";
import { useSnackbar } from "context/SnackbarContext";

export default function AdminChangePassword() {
  const { showSuccess, showError } = useSnackbar();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    if (newPassword !== confirmPassword) {
      setError("A nova senha e a confirmação devem ser iguais.");
      return;
    }
    if (newPassword.length < 6) {
      setError("A nova senha deve ter pelo menos 6 caracteres.");
      return;
    }
    setLoading(true);
    try {
      await adminApi.auth.changePassword(currentPassword, newPassword);
      showSuccess("Senha alterada com sucesso!");
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (err) {
      const msg = err.message || "Erro ao alterar senha";
      setError(msg);
      showError(msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <DashboardLayout>
      <DashboardNavbar />
      <MDBox py={3}>
        <MDTypography variant="h4" fontWeight="medium" mb={3}>Trocar senha</MDTypography>
        <Card sx={{ maxWidth: 500 }}>
          <MDBox p={3} component="form" onSubmit={handleSubmit}>
            {error && <MDTypography color="error" variant="caption" display="block" mb={2}>{error}</MDTypography>}
            <MDBox mb={2}>
              <MDInput type="password" label="Senha atual" fullWidth value={currentPassword} onChange={(e) => setCurrentPassword(e.target.value)} required />
            </MDBox>
            <MDBox mb={2}>
              <MDInput type="password" label="Nova senha" fullWidth value={newPassword} onChange={(e) => setNewPassword(e.target.value)} required />
            </MDBox>
            <MDBox mb={2}>
              <MDInput type="password" label="Confirmar nova senha" fullWidth value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} required />
            </MDBox>
            <MDButton variant="gradient" color="dark" fullWidth type="submit" disabled={loading}>
              {loading ? "Alterando..." : "Alterar senha"}
            </MDButton>
          </MDBox>
        </Card>
      </MDBox>
      <Footer />
    </DashboardLayout>
  );
}
