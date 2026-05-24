import { Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { useState, useEffect } from "react";
import {
  Box, Drawer, List, ListItemButton, ListItemIcon, ListItemText,
  AppBar, Toolbar, Typography, Avatar, Divider,
} from "@mui/material";
import DashboardIcon from "@mui/icons-material/Dashboard";
import ReceiptLongIcon from "@mui/icons-material/ReceiptLong";
import GavelIcon from "@mui/icons-material/Gavel";
import PeopleIcon from "@mui/icons-material/People";
import TrendingUpIcon from "@mui/icons-material/TrendingUp";
import LogoutIcon from "@mui/icons-material/Logout";
import { adminKeyStore } from "./api";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import Orders from "./pages/Orders";
import Disputes from "./pages/Disputes";
import Scores from "./pages/Scores";
import YieldReport from "./pages/YieldReport";

const DRAWER_WIDTH = 240;

const NAV = [
  { label: "Dashboard", icon: <DashboardIcon />, path: "/dashboard" },
  { label: "Pedidos", icon: <ReceiptLongIcon />, path: "/orders" },
  { label: "Disputas", icon: <GavelIcon />, path: "/disputes" },
  { label: "Usuários / Score", icon: <PeopleIcon />, path: "/scores" },
  { label: "Yield Report", icon: <TrendingUpIcon />, path: "/yield" },
];

function AdminLayout({ children }) {
  const navigate = useNavigate();
  const path = window.location.pathname;

  const logout = () => {
    adminKeyStore.clear();
    navigate("/login");
  };

  return (
    <Box sx={{ display: "flex" }}>
      <AppBar position="fixed" sx={{ zIndex: (t) => t.zIndex.drawer + 1, bgcolor: "primary.main" }}>
        <Toolbar>
          <Typography variant="h6" fontWeight={700} sx={{ flexGrow: 1 }}>
            Holdfy Admin
          </Typography>
          <Avatar sx={{ bgcolor: "white", color: "primary.main", width: 32, height: 32, fontSize: 14 }}>
            A
          </Avatar>
        </Toolbar>
      </AppBar>
      <Drawer
        variant="permanent"
        sx={{
          width: DRAWER_WIDTH,
          "& .MuiDrawer-paper": { width: DRAWER_WIDTH, boxSizing: "border-box", pt: "64px" },
        }}
      >
        <List>
          {NAV.map((item) => (
            <ListItemButton
              key={item.path}
              selected={path.startsWith(item.path)}
              onClick={() => navigate(item.path)}
            >
              <ListItemIcon sx={{ color: path.startsWith(item.path) ? "primary.main" : "text.secondary" }}>
                {item.icon}
              </ListItemIcon>
              <ListItemText primary={item.label} />
            </ListItemButton>
          ))}
        </List>
        <Divider />
        <List>
          <ListItemButton onClick={logout}>
            <ListItemIcon><LogoutIcon /></ListItemIcon>
            <ListItemText primary="Sair" />
          </ListItemButton>
        </List>
      </Drawer>
      <Box component="main" sx={{ flexGrow: 1, p: 3, pt: "80px", ml: `${DRAWER_WIDTH}px` }}>
        {children}
      </Box>
    </Box>
  );
}

function RequireAdminAuth({ children }) {
  const key = adminKeyStore.get();
  if (!key) return <Navigate to="/login" replace />;
  return children;
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route
        path="/*"
        element={
          <RequireAdminAuth>
            <AdminLayout>
              <Routes>
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/orders" element={<Orders />} />
                <Route path="/disputes" element={<Disputes />} />
                <Route path="/scores" element={<Scores />} />
                <Route path="/yield" element={<YieldReport />} />
                <Route path="*" element={<Navigate to="/dashboard" replace />} />
              </Routes>
            </AdminLayout>
          </RequireAdminAuth>
        }
      />
    </Routes>
  );
}
