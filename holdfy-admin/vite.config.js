import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  base: "/holdfy-admin/",
  plugins: [react()],
  server: {
    port: 3011,
    proxy: {
      // Espelha o proxy do apicash-frontend em produção: /svc/admin/* -> :3001, stripando o prefixo.
      "/svc/admin": {
        target: "http://localhost:3001",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/svc\/admin/, ""),
      },
    },
  },
});
