/**
 * ProtectedRoute - redireciona para login se não autenticado
 */

import { Navigate, useLocation } from "react-router-dom";
import PropTypes from "prop-types";
import { useAuth } from "context/AuthContext";

function ProtectedRoute({ children, requiredProfile }) {
  const { isAuthenticated, getLoginPath, profile } = useAuth();
  const location = useLocation();

  if (!isAuthenticated()) {
    return <Navigate to={getLoginPath()} state={{ from: location }} replace />;
  }

  if (requiredProfile && profile !== requiredProfile) {
    const base = profile === "admin" ? "/admin" : profile === "backoffice" ? "/backoffice" : "/customer";
    return <Navigate to={base} replace />;
  }

  return children;
}

ProtectedRoute.propTypes = {
  children: PropTypes.node.isRequired,
  requiredProfile: PropTypes.oneOf(["customer", "admin", "backoffice"]),
};

export default ProtectedRoute;
