import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => ({
  base: mode === "production" ? "/holdfy-admin/" : "/",
  plugins: [react()],
  server: {
    port: 3010,
    proxy: {
      "/admin": { target: "http://localhost:3001", changeOrigin: true },
      "/health": { target: "http://localhost:3001", changeOrigin: true },
    },
  },
}));
