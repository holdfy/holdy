import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import App from "./App.tsx";
import i18n, { i18nReady } from "./i18n";
import "./index.css";

const rootEl = document.getElementById("root");

function mountApp() {
  if (!rootEl || rootEl.dataset.mounted === "true") return;
  rootEl.dataset.mounted = "true";
  createRoot(rootEl).render(
    <StrictMode>
      <I18nextProvider i18n={i18n}>
        <App />
      </I18nextProvider>
    </StrictMode>,
  );
}

const INIT_TIMEOUT_MS = 3000;

void Promise.race([
  i18nReady,
  new Promise<void>((resolve) => {
    window.setTimeout(resolve, INIT_TIMEOUT_MS);
  }),
])
  .catch((err) => {
    console.error("[holdfy-site] i18n init failed; mounting app anyway", err);
  })
  .finally(mountApp);
