/**
 * AuthContext - autenticação por perfil (customer, admin, backoffice)
 */

import { createContext, useContext, useState, useCallback, useEffect } from "react";
import PropTypes from "prop-types";
import { setAuthToken } from "services/api";

const AuthContext = createContext(null);

export function AuthProvider({ children }) {
  const [profile, setProfile] = useState(() => {
    if (localStorage.getItem("customerToken")) return "customer";
    if (localStorage.getItem("adminToken")) return "admin";
    if (localStorage.getItem("backofficeToken")) return "backoffice";
    return null;
  });
  const [userId, setUserId] = useState(null);
  const [username, setUsername] = useState(null);

  const getToken = useCallback(() => {
    return (
      localStorage.getItem("customerToken") ||
      localStorage.getItem("adminToken") ||
      localStorage.getItem("backofficeToken")
    );
  }, []);

  const setToken = useCallback((token, profileType, extra = {}) => {
    setAuthToken(profileType, token);
    setProfile(profileType);
    setUserId(extra.userId ?? null);
    setUsername(extra.username ?? null);
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem("customerToken");
    localStorage.removeItem("adminToken");
    localStorage.removeItem("backofficeToken");
    setProfile(null);
    setUserId(null);
    setUsername(null);
  }, []);

  const isAuthenticated = useCallback(() => {
    return !!getToken();
  }, [getToken]);

  const getLoginPath = useCallback(() => {
    if (profile === "admin") return "/admin/login";
    if (profile === "backoffice") return "/backoffice/login";
    return "/customer/login";
  }, [profile]);

  useEffect(() => {
    const token = getToken();
    if (token && !profile) {
      if (localStorage.getItem("customerToken")) setProfile("customer");
      else if (localStorage.getItem("adminToken")) setProfile("admin");
      else if (localStorage.getItem("backofficeToken")) setProfile("backoffice");
    }
  }, [getToken, profile]);

  const value = {
    profile,
    userId,
    username,
    getToken,
    setToken,
    logout,
    isAuthenticated,
    getLoginPath,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

AuthProvider.propTypes = {
  children: PropTypes.node.isRequired,
};

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return context;
}
