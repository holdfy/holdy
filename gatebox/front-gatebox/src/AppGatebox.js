/**
 * App Gatebox - rotas por perfil (customer, admin, backoffice)
 */

import { useState, useEffect } from "react";
import { Routes, Route, Navigate, useLocation } from "react-router-dom";
import { ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import Icon from "@mui/material/Icon";
import MDBox from "components/MDBox";
import Sidenav from "examples/Sidenav";
import Configurator from "examples/Configurator";
import theme from "assets/theme";
import themeDark from "assets/theme-dark";
import {
  useMaterialUIController,
  setMiniSidenav,
  setOpenConfigurator,
} from "context";
import { AuthProvider, useAuth } from "context/AuthContext";
import { SnackbarProvider } from "context/SnackbarContext";
import ProtectedRoute from "components/ProtectedRoute";
import PortalIndex from "layouts/portal/PortalIndex";
import brandWhite from "assets/images/logo-ct.png";
import brandDark from "assets/images/logo-ct-dark.png";
import {
  publicRoutes,
  customerRoutes,
  adminRoutes,
  backofficeRoutes,
  customerSidenav,
  adminSidenav,
  backofficeSidenav,
} from "routes-gatebox";

function getSidenavRoutes(profile) {
  if (profile === "admin") return adminSidenav;
  if (profile === "backoffice") return backofficeSidenav;
  return customerSidenav;
}

function AppContent() {
  const [controller, dispatch] = useMaterialUIController();
  const { miniSidenav, openConfigurator, sidenavColor, transparentSidenav, whiteSidenav, darkMode } = controller;
  const [onMouseEnter, setOnMouseEnter] = useState(false);
  const { pathname } = useLocation();
  const { profile, isAuthenticated } = useAuth();

  const isPublicRoute = pathname === "/" || pathname === "/customer/login" || pathname === "/admin/login" || pathname === "/backoffice/login";
  const showSidenav = isAuthenticated() && !isPublicRoute;
  const sidenavRoutes = getSidenavRoutes(profile);

  const handleOnMouseEnter = () => {
    if (miniSidenav && !onMouseEnter) {
      setMiniSidenav(dispatch, false);
      setOnMouseEnter(true);
    }
  };

  const handleOnMouseLeave = () => {
    if (onMouseEnter) {
      setMiniSidenav(dispatch, true);
      setOnMouseEnter(false);
    }
  };

  const handleConfiguratorOpen = () => setOpenConfigurator(dispatch, !openConfigurator);

  useEffect(() => {
    document.documentElement.scrollTop = 0;
    document.scrollingElement.scrollTop = 0;
  }, [pathname]);

  useEffect(() => {
    const loginPaths = {
      "/": "GateBox — Acesso",
      "/customer/login": "GateBox — Login Cliente",
      "/admin/login": "GateBox — Login Admin",
      "/backoffice/login": "GateBox — Login Backoffice",
    };
    if (loginPaths[pathname]) {
      document.title = loginPaths[pathname];
    } else if (profile === "admin") {
      document.title = "GateBox — Admin";
    } else if (profile === "backoffice") {
      document.title = "GateBox — Backoffice";
    } else if (profile === "customer") {
      document.title = "GateBox — Cliente";
    } else {
      document.title = "GateBox";
    }
  }, [pathname, profile]);

  const configsButton = showSidenav ? (
    <MDBox
      display="flex"
      justifyContent="center"
      alignItems="center"
      width="3.25rem"
      height="3.25rem"
      bgColor="white"
      shadow="sm"
      borderRadius="50%"
      position="fixed"
      right="2rem"
      bottom="2rem"
      zIndex={99}
      color="dark"
      sx={{ cursor: "pointer" }}
      onClick={handleConfiguratorOpen}
    >
      <Icon fontSize="small" color="inherit">settings</Icon>
    </MDBox>
  ) : null;

  return (
    <ThemeProvider theme={darkMode ? themeDark : theme}>
      <CssBaseline />
      {showSidenav && (
        <>
          <Sidenav
            color={sidenavColor}
            brand={(transparentSidenav && !darkMode) || whiteSidenav ? brandDark : brandWhite}
            brandName="Gatebox"
            routes={sidenavRoutes}
            onMouseEnter={handleOnMouseEnter}
            onMouseLeave={handleOnMouseLeave}
          />
          <Configurator />
          {configsButton}
        </>
      )}
      <Routes>
        {publicRoutes.map((r) => (
          <Route key={r.path} path={r.path} element={r.element} />
        ))}
        {customerRoutes.map((r) => (
          <Route
            key={r.path}
            path={r.path}
            element={
              <ProtectedRoute requiredProfile="customer">
                {r.element}
              </ProtectedRoute>
            }
          />
        ))}
        {adminRoutes.map((r) => (
          <Route
            key={r.path}
            path={r.path}
            element={
              <ProtectedRoute requiredProfile="admin">
                {r.element}
              </ProtectedRoute>
            }
          />
        ))}
        {backofficeRoutes.map((r) => (
          <Route
            key={r.path}
            path={r.path}
            element={
              <ProtectedRoute requiredProfile="backoffice">
                {r.element}
              </ProtectedRoute>
            }
          />
        ))}
        <Route path="/" element={<PortalIndex />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </ThemeProvider>
  );
}

export default function AppGatebox() {
  return (
    <AuthProvider>
      <SnackbarProvider>
        <AppContent />
      </SnackbarProvider>
    </AuthProvider>
  );
}
