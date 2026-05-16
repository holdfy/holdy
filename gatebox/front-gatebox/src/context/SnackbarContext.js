/**
 * SnackbarContext - notificações globais (sucesso, erro, info)
 */

import { createContext, useContext, useState, useCallback } from "react";
import PropTypes from "prop-types";
import Snackbar from "@mui/material/Snackbar";
import Alert from "@mui/material/Alert";

const SnackbarContext = createContext(null);

export function SnackbarProvider({ children }) {
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [severity, setSeverity] = useState("info");

  const show = useCallback((msg, sev = "info") => {
    setMessage(msg);
    setSeverity(sev);
    setOpen(true);
  }, []);

  const showSuccess = useCallback((msg) => show(msg, "success"), [show]);
  const showError = useCallback((msg) => show(msg, "error"), [show]);
  const showWarning = useCallback((msg) => show(msg, "warning"), [show]);
  const showInfo = useCallback((msg) => show(msg, "info"), [show]);

  const handleClose = useCallback((_, reason) => {
    if (reason !== "clickaway") setOpen(false);
  }, []);

  const value = { show, showSuccess, showError, showWarning, showInfo };

  return (
    <SnackbarContext.Provider value={value}>
      {children}
      <Snackbar open={open} autoHideDuration={5000} onClose={handleClose} anchorOrigin={{ vertical: "bottom", horizontal: "right" }}>
        <Alert onClose={handleClose} severity={severity} variant="filled" sx={{ width: "100%" }}>
          {message}
        </Alert>
      </Snackbar>
    </SnackbarContext.Provider>
  );
}

SnackbarProvider.propTypes = {
  children: PropTypes.node.isRequired,
};

export function useSnackbar() {
  const context = useContext(SnackbarContext);
  return context || { show: () => {}, showSuccess: () => {}, showError: () => {}, showWarning: () => {}, showInfo: () => {} };
}
