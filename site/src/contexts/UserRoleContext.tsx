import { createContext, useContext, useState, useCallback, ReactNode } from "react";
import { api, tokenStore, type LoginResponse } from "@/lib/api-client";

export type UserRole = "buyer" | "seller" | "admin";
export type PersonType = "pf" | "pj";

interface UserIdentity {
  id: string;
  username: string;
  role: UserRole;
  personType: PersonType;
  riskScore: number | null;
}

interface UserRoleContextType {
  role: UserRole;
  user: UserIdentity | null;
  isAuthenticated: boolean;
  login: (username: string, password: string, roleHint: "buyer" | "seller") => Promise<void>;
  logout: () => void;
  setRole: (role: UserRole) => void;
}

const UserRoleContext = createContext<UserRoleContextType | undefined>(undefined);

function decodeJwtPayload(token: string): Record<string, unknown> {
  try {
    const [, payload] = token.split(".");
    return JSON.parse(atob(payload.replace(/-/g, "+").replace(/_/g, "/")));
  } catch {
    return {};
  }
}

function buildIdentity(resp: LoginResponse, roleHint: "buyer" | "seller"): UserIdentity {
  const claims = decodeJwtPayload(resp.access_token);
  const jwtRole = (claims.role as string | undefined) ?? roleHint;
  const role: UserRole = jwtRole === "seller" ? "seller" : jwtRole === "admin" ? "admin" : "buyer";
  const riskScore = typeof claims.risk_score === "number" ? claims.risk_score : null;
  const personType: PersonType = (claims.person_type as string | undefined) === "legal" ? "pj" : "pf";
  return {
    id: (claims.sub as string | undefined) ?? "",
    username: (claims.username as string | undefined) ?? "",
    role,
    personType,
    riskScore,
  };
}

export function UserRoleProvider({ children }: { children: ReactNode }) {
  const [role, setRole] = useState<UserRole>("buyer");
  const [user, setUser] = useState<UserIdentity | null>(null);

  const login = useCallback(async (username: string, password: string, roleHint: "buyer" | "seller") => {
    const resp = await api.login(username, password);
    tokenStore.set(resp.access_token, resp.refresh_token);
    const identity = buildIdentity(resp, roleHint);
    setUser(identity);
    setRole(identity.role === "admin" ? roleHint : identity.role);
  }, []);

  const logout = useCallback(() => {
    tokenStore.clear();
    setUser(null);
    setRole("buyer");
  }, []);

  return (
    <UserRoleContext.Provider value={{ role, user, isAuthenticated: !!user, login, logout, setRole }}>
      {children}
    </UserRoleContext.Provider>
  );
}

export function useUserRole() {
  const context = useContext(UserRoleContext);
  if (!context) throw new Error("useUserRole must be used within UserRoleProvider");
  return context;
}
