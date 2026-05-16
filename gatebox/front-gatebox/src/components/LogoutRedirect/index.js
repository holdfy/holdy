/**
 * LogoutRedirect - limpa token e redireciona para login
 */

import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "context/AuthContext";

export default function LogoutRedirect({ loginPath = "/customer/login" }) {
  const { logout } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    logout();
    navigate(loginPath, { replace: true });
  }, [logout, navigate, loginPath]);

  return null;
}
