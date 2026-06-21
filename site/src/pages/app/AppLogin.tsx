import { Shield, Lock, User, Eye, EyeOff, Fingerprint, ShieldCheck, HelpCircle, Building2, Loader2, AlertCircle } from "lucide-react";
import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Trans, useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { maskCpfCnpj, stripDoc, validateCpfOrCnpj } from "@/lib/format";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { useUserRole, UserRole, type PersonType } from "@/contexts/UserRoleContext";
import { toast } from "sonner";
import type { ApiError } from "@/lib/api-client";
import { api, tokenStore } from "@/lib/api-client";

export default function AppLogin() {
  const { t } = useTranslation();
  const [showPassword, setShowPassword] = useState(false);
  const [forgotOpen, setForgotOpen] = useState(false);
  const [supportOpen, setSupportOpen] = useState(false);
  const [signUpOpen, setSignUpOpen] = useState(false);
  const [resetEmail, setResetEmail] = useState("");
  const [regEmail, setRegEmail] = useState("");
  const [regPassword, setRegPassword] = useState("");
  const [regPersonType, setRegPersonType] = useState<PersonType>("pf");
  const [regName, setRegName] = useState("");
  const [regDocInfo, setRegDocInfo] = useState<{ name: string | null; situation: string | null } | null>(null);
  const [regDocLoading, setRegDocLoading] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [loginError, setLoginError] = useState<string | null>(null);
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { login, loginFromToken, setRole } = useUserRole();

  // OAuth callback: backend redireciona para /login?token=...&refresh=...&oauth=1
  useEffect(() => {
    const token = searchParams.get("token");
    const refresh = searchParams.get("refresh");
    const oauthError = searchParams.get("oauth_error");

    if (oauthError) {
      toast.error(t("auth.oauthError"));
      // Remove params sem recarregar
      navigate("/login", { replace: true });
      return;
    }
    if (token && refresh) {
      loginFromToken(token, refresh);
      navigate("/buyer", { replace: true });
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleUsernameChange = (val: string) => {
    setLoginError(null);
    if (/[a-zA-Z@]/.test(val)) {
      setUsername(val);
    } else {
      setUsername(maskCpfCnpj(val));
    }
  };

  const handleLogin = async () => {
    if (!username.trim() || !password.trim()) {
      setLoginError(t("auth.toastFillFields"));
      return;
    }
    const digits = stripDoc(username);
    const loginId = (digits.length === 11 || digits.length === 14) ? digits : username.trim();
    setLoading(true);
    setLoginError(null);
    try {
      await login(loginId, password, "buyer");
      navigate("/buyer");
    } catch (err) {
      const apiErr = err as ApiError;
      const msg = apiErr?.status === 401
        ? t("auth.invalidCredentials")
        : t("auth.loginError");
      setLoginError(msg);
    } finally {
      setLoading(false);
    }
  };

  const sendReset = () => {
    if (!resetEmail.trim()) {
      toast.error(t("auth.toastEnterEmail"));
      return;
    }
    toast.success(t("auth.toastResetSent"));
    setForgotOpen(false);
    setResetEmail("");
  };

  const handleRegDocBlur = async () => {
    const digits = stripDoc(regEmail);
    if (digits.length !== 11 && digits.length !== 14) return;
    if (!validateCpfOrCnpj(digits)) return;
    // Lookup Receita Federal requer JWT — pula silenciosamente se não autenticado
    if (!tokenStore.getAccess()) return;
    setRegDocLoading(true);
    setRegDocInfo(null);
    try {
      const result = await api.lookupDocument(digits);
      setRegDocInfo({ name: result.name, situation: result.situation });
    } catch {
      // RF lookup falhou — não bloqueia o cadastro
    } finally {
      setRegDocLoading(false);
    }
  };

  const createAccount = async () => {
    if (!regEmail.trim() || !regPassword.trim()) {
      toast.error(t("auth.toastFillFields"));
      return;
    }
    const docDigits = stripDoc(regEmail);
    if (!validateCpfOrCnpj(docDigits)) {
      toast.error(
        regPersonType === "pj"
          ? t("auth.invalidCnpj", "CNPJ inválido. Verifique os dígitos.")
          : t("auth.invalidCpf", "CPF inválido. Verifique os dígitos.")
      );
      return;
    }
    setLoading(true);
    try {
      await login(docDigits, regPassword, "buyer");
      toast.success(t("auth.toastAccountCreated"));
      setSignUpOpen(false);
      setRegEmail("");
      setRegPassword("");
      setRegName("");
      navigate("/buyer");
    } catch {
      toast.error(t("auth.loginError", "Erro ao entrar. Tente novamente."));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background flex flex-col max-w-lg mx-auto">
      <div className="px-6 pt-8 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Shield className="h-6 w-6 text-primary" />
          <span className="font-display font-bold text-xl">{t("common.holdfy")}</span>
        </div>
        <LanguageSwitcher variant="app" />
      </div>

      <div className="flex-1 px-6 pt-12 pb-8 flex flex-col">
        <div className="text-center mb-6">
          <div className="mx-auto h-16 w-16 rounded-2xl bg-muted flex items-center justify-center mb-6">
            <Lock className="h-8 w-8 text-foreground" />
          </div>
          <h1 className="font-display text-3xl font-bold leading-tight whitespace-pre-line">{t("auth.welcomeTitle")}</h1>
          <p className="text-xs font-semibold tracking-[0.2em] text-muted-foreground uppercase mt-3">{t("auth.welcomeSubtitle")}</p>
        </div>

        <div className="space-y-5 flex-1">
          <div>
            <label className="text-xs font-semibold tracking-wider uppercase text-foreground mb-2 block">{t("auth.cpfCnpjLabel", "CPF ou CNPJ")}</label>
            <div className="relative">
              <User className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="000.000.000-00"
                className="h-14 pl-11 rounded-xl bg-muted border-0 text-sm"
                value={username}
                onChange={(e) => handleUsernameChange(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleLogin()}
                autoComplete="username"
                inputMode="numeric"
                maxLength={18}
              />
            </div>
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-xs font-semibold tracking-wider uppercase text-foreground">{t("auth.password")}</label>
              <button
                type="button"
                className="text-xs font-semibold text-secondary"
                onClick={() => setForgotOpen(true)}
              >
                {t("auth.forgotPassword")}
              </button>
            </div>
            <div className="relative">
              <Lock className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                type={showPassword ? "text" : "password"}
                className="h-14 pl-11 pr-12 rounded-xl bg-muted border-0 text-sm"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleLogin()}
                autoComplete="current-password"
              />
              <button
                type="button"
                className="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground"
                onClick={() => setShowPassword(!showPassword)}
              >
                {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </button>
            </div>
          </div>

          {loginError && (
            <div className="flex items-start gap-2 rounded-xl bg-destructive/10 border border-destructive/30 px-4 py-3 text-sm text-destructive font-medium">
              <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
              {loginError}
            </div>
          )}

          <Button
            className="w-full h-14 rounded-xl vault-card border-0 text-white font-semibold text-base hover:opacity-90"
            onClick={handleLogin}
            disabled={loading}
          >
            <ShieldCheck className="mr-2 h-5 w-5" />
            {loading ? t("auth.loggingIn", "Entrando...") : t("auth.secureLogin")}
          </Button>

          <Button
            type="button"
            variant="outline"
            className="w-full h-14 rounded-xl text-sm font-semibold"
            onClick={() => toast.info(t("auth.toastBiometric"))}
          >
            <Fingerprint className="mr-2 h-5 w-5" />
            {t("auth.biometricLogin")}
          </Button>

          {/* Social login */}
          <div className="relative flex items-center gap-3">
            <div className="flex-1 border-t" />
            <span className="text-xs text-muted-foreground font-medium whitespace-nowrap">{t("auth.orContinueWith")}</span>
            <div className="flex-1 border-t" />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <a
              href="/auth/oauth/google"
              className="flex items-center justify-center gap-2 h-12 rounded-xl border bg-card hover:bg-muted transition text-sm font-semibold"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l3.66-2.84z"/>
                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
              </svg>
              Google
            </a>
            <a
              href="/auth/oauth/apple"
              className="flex items-center justify-center gap-2 h-12 rounded-xl border bg-card hover:bg-muted transition text-sm font-semibold"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.8-.91.65.03 2.47.26 3.64 1.98l-.09.06c-.22.15-2.39 1.39-2.37 4.15.03 3.27 2.87 4.36 2.9 4.37l-.08.1zM13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
              </svg>
              Apple
            </a>
            <a
              href="/auth/oauth/facebook"
              className="flex items-center justify-center gap-2 h-12 rounded-xl border bg-card hover:bg-muted transition text-sm font-semibold"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="#1877F2" aria-hidden="true">
                <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/>
              </svg>
              Facebook
            </a>
            <a
              href="/auth/oauth/linkedin"
              className="flex items-center justify-center gap-2 h-12 rounded-xl border bg-card hover:bg-muted transition text-sm font-semibold"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="#0A66C2" aria-hidden="true">
                <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433a2.062 2.062 0 01-2.063-2.065 2.064 2.064 0 112.063 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
              </svg>
              LinkedIn
            </a>
          </div>
        </div>

        <div className="text-center pt-6">
          <p className="text-sm text-muted-foreground">
            {t("auth.noAccount")}{" "}
            <button type="button" className="font-bold text-foreground" onClick={() => setSignUpOpen(true)}>
              {t("auth.signUp")}
            </button>
          </p>
        </div>
      </div>

      <div className="border-t bg-background">
        <div className="flex items-center justify-center gap-12 py-4 max-w-lg mx-auto">
          <button type="button" className="flex flex-col items-center gap-1" onClick={handleLogin}>
            <div className="h-10 w-10 rounded-xl vault-card flex items-center justify-center">
              <Lock className="h-5 w-5 text-white" />
            </div>
            <span className="text-[10px] font-semibold tracking-wider uppercase">{t("auth.secureLogin")}</span>
          </button>
          <button type="button" className="flex flex-col items-center gap-1" onClick={() => setSupportOpen(true)}>
            <div className="h-10 w-10 rounded-xl bg-muted flex items-center justify-center">
              <HelpCircle className="h-5 w-5 text-muted-foreground" />
            </div>
            <span className="text-[10px] font-semibold tracking-wider uppercase text-muted-foreground">{t("common.support")}</span>
          </button>
        </div>
      </div>

      <Dialog open={forgotOpen} onOpenChange={setForgotOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("auth.resetTitle")}</DialogTitle>
            <DialogDescription>{t("auth.resetDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="reset-email">{t("common.email")}</Label>
            <Input
              id="reset-email"
              type="email"
              placeholder={t("auth.resetPlaceholder")}
              value={resetEmail}
              onChange={(e) => setResetEmail(e.target.value)}
            />
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setForgotOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button type="button" className="vault-card border-0 text-white hover:opacity-90" onClick={sendReset}>
              {t("common.sendLink")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={supportOpen} onOpenChange={setSupportOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("common.support")}</DialogTitle>
            <DialogDescription>
              <Trans
                i18nKey="auth.supportDesc"
                components={{
                  1: (
                    <a
                      className="font-semibold text-foreground underline"
                      href="mailto:support@holdfy.com"
                    />
                  ),
                }}
              />
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button type="button" onClick={() => setSupportOpen(false)}>
              {t("common.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={signUpOpen} onOpenChange={setSignUpOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("auth.signUpTitle")}</DialogTitle>
            <DialogDescription>{t("auth.signUpDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="text-xs font-semibold tracking-wider uppercase text-foreground mb-2 block">
                {t("auth.personType", "Tipo de pessoa")}
              </label>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={() => setRegPersonType("pf")}
                  className={`flex items-center gap-2 p-3 rounded-xl border-2 transition-all ${
                    regPersonType === "pf"
                      ? "border-primary bg-primary/5"
                      : "border-border bg-card hover:border-muted-foreground/30"
                  }`}
                >
                  <User className={`h-4 w-4 ${regPersonType === "pf" ? "text-primary" : "text-muted-foreground"}`} />
                  <span className={`text-sm font-semibold ${regPersonType === "pf" ? "text-primary" : "text-muted-foreground"}`}>
                    {t("auth.personPF", "Pessoa Física")}
                  </span>
                </button>
                <button
                  type="button"
                  onClick={() => setRegPersonType("pj")}
                  className={`flex items-center gap-2 p-3 rounded-xl border-2 transition-all ${
                    regPersonType === "pj"
                      ? "border-primary bg-primary/5"
                      : "border-border bg-card hover:border-muted-foreground/30"
                  }`}
                >
                  <Building2 className={`h-4 w-4 ${regPersonType === "pj" ? "text-primary" : "text-muted-foreground"}`} />
                  <span className={`text-sm font-semibold ${regPersonType === "pj" ? "text-primary" : "text-muted-foreground"}`}>
                    {t("auth.personPJ", "Pessoa Jurídica")}
                  </span>
                </button>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="reg-name">
                {regPersonType === "pj"
                  ? t("auth.companyName", "Razão social")
                  : t("auth.fullName", "Nome completo")}
              </Label>
              <Input
                id="reg-name"
                placeholder={regPersonType === "pj" ? "Ex: Empresa Ltda" : "Ex: João da Silva"}
                value={regName}
                onChange={(e) => setRegName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="reg-email">
                {regPersonType === "pj"
                  ? t("auth.cnpjLabel", "CNPJ")
                  : t("auth.cpfLabel", "CPF")}
              </Label>
              <Input
                id="reg-email"
                placeholder={regPersonType === "pj" ? "00.000.000/0001-00" : "000.000.000-00"}
                value={regEmail}
                onChange={(e) => { setRegEmail(maskCpfCnpj(e.target.value)); setRegDocInfo(null); }}
                onBlur={handleRegDocBlur}
                inputMode="numeric"
                maxLength={18}
              />
              {regDocLoading && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground mt-1">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  Consultando Receita Federal…
                </div>
              )}
              {regDocInfo && !regDocLoading && (
                <div className="flex items-start gap-2 bg-secondary/5 border border-secondary/20 rounded-xl p-3 mt-1">
                  <User className="h-4 w-4 text-secondary mt-0.5 shrink-0" />
                  <div className="text-xs space-y-0.5">
                    {regDocInfo.name && <p className="font-semibold text-foreground">{regDocInfo.name}</p>}
                    {regDocInfo.situation && <p className="text-muted-foreground">Situação: {regDocInfo.situation}</p>}
                  </div>
                </div>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="reg-pass">{t("auth.password")}</Label>
              <Input
                id="reg-pass"
                type="password"
                placeholder="••••••••"
                value={regPassword}
                onChange={(e) => setRegPassword(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setSignUpOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button type="button" className="vault-card border-0 text-white hover:opacity-90" onClick={createAccount}>
              {t("common.createAccount")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
