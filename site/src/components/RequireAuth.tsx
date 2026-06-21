import { Navigate, useLocation } from "react-router-dom";
import { useUserRole } from "@/contexts/UserRoleContext";
import { tokenStore } from "@/lib/api-client";
import type { ReactNode } from "react";

export default function RequireAuth({ children }: { children: ReactNode }) {
  const { isAuthenticated } = useUserRole();
  const location = useLocation();

  // tokenStore é síncrono — evita race condition entre setUser() e navigate()
  if (!isAuthenticated && !tokenStore.getAccess()) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }
  return <>{children}</>;
}
