import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from "react";
import { api, tokenStore, type LoginResponse } from "@/lib/api-client";

export type UserRole = "buyer" | "seller" | "admin";
export type PersonType = "pf" | "pj";

interface UserIdentity {
  id: string;
  username: string;
  role: UserRole;
  personType: PersonType;
  riskScore: number | null;
  email: string | null;
  name: string | null;
  avatarUrl: string | null;
  document: string;
  hasDocument: boolean;
}

interface UserRoleContextType {
  role: UserRole;
  user: UserIdentity | null;
  isAuthenticated: boolean;
  login: (username: string, password: string, roleHint: "buyer" | "seller") => Promise<void>;
  loginFromToken: (accessToken: string, refreshToken: string) => void;
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

function buildIdentityFromClaims(claims: Record<string, unknown>, roleHint: "buyer" | "seller"): UserIdentity {
  const jwtRole = (claims.role as string | undefined) ?? roleHint;
  const role: UserRole = jwtRole === "seller" ? "seller" : jwtRole === "admin" ? "admin" : "buyer";
  const riskScore = typeof claims.risk_score === "number" ? claims.risk_score : null;
  const personType: PersonType = (claims.person_type as string | undefined) === "legal" ? "pj" : "pf";
  const document = (claims.document as string | undefined) ?? "";
  return {
    id: (claims.sub as string | undefined) ?? "",
    username: (claims.username as string | undefined) ?? "",
    role,
    personType,
    riskScore,
    email: (claims.email as string | undefined) ?? null,
    name: (claims.name as string | undefined) ?? null,
    avatarUrl: (claims.avatar_url as string | undefined) ?? null,
    document,
    hasDocument: document.replace(/\D/g, "").length >= 11,
  };
}

function buildIdentity(resp: LoginResponse, roleHint: "buyer" | "seller"): UserIdentity {
  const claims = decodeJwtPayload(resp.access_token);
  return buildIdentityFromClaims(claims, roleHint);
}

export function UserRoleProvider({ children }: { children: ReactNode }) {
  const [role, setRole] = useState<UserRole>("buyer");
  const [user, setUser] = useState<UserIdentity | null>(null);

  // Restaura sessão de token já armazenado (ex: reload da página)
  useEffect(() => {
    const token = tokenStore.getAccess();
    if (token && !user) {
      const claims = decodeJwtPayload(token);
      if (claims.sub) {
        const identity = buildIdentityFromClaims(claims, "buyer");
        setUser(identity);
        setRole(identity.role === "admin" ? "buyer" : identity.role);
      }
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const login = useCallback(async (username: string, password: string, roleHint: "buyer" | "seller") => {
    const resp = await api.login(username, password);
    tokenStore.set(resp.access_token, resp.refresh_token);
    const identity = buildIdentity(resp, roleHint);
    setUser(identity);
    setRole(identity.role === "admin" ? roleHint : identity.role);
  }, []);

  /** Usado após OAuth callback: tokens já chegaram via query-param. */
  const loginFromToken = useCallback((accessToken: string, refreshToken: string) => {
    tokenStore.set(accessToken, refreshToken);
    const claims = decodeJwtPayload(accessToken);
    const identity = buildIdentityFromClaims(claims, "buyer");
    setUser(identity);
    setRole(identity.role === "admin" ? "buyer" : identity.role);
  }, []);

  const logout = useCallback(() => {
    tokenStore.clear();
    setUser(null);
    setRole("buyer");
  }, []);

  return (
    <UserRoleContext.Provider value={{ role, user, isAuthenticated: !!user, login, loginFromToken, logout, setRole }}>
      {children}
    </UserRoleContext.Provider>
  );
}

export function useUserRole() {
  const context = useContext(UserRoleContext);
  if (!context) throw new Error("useUserRole must be used within UserRoleProvider");
  return context;
}
