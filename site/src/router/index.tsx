import { useRoutes } from "react-router-dom";
import routes from "./config";

/** Readdy-style `AppRoutes` — single `useRoutes` tree (see preview `router/index.ts`). */
export function AppRoutes() {
  return useRoutes(routes);
}
