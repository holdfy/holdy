import { Navigate, useLocation } from "react-router-dom";
import { useUserRole } from "@/contexts/UserRoleContext";
import type { ReactNode } from "react";

export default function RequireAuth({ children }: { children: ReactNode }) {
  const { isAuthenticated } = useUserRole();
  const location = useLocation();

  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }
  return <>{children}</>;
}
