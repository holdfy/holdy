import { Shield, Lock, User, Eye, EyeOff, Fingerprint, ShieldCheck, HelpCircle, Store, ShoppingBag } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Trans, useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { useUserRole, UserRole } from "@/contexts/UserRoleContext";
import { toast } from "sonner";
import type { ApiError } from "@/lib/api-client";

export default function AppLogin() {
  const { t } = useTranslation();
  const [showPassword, setShowPassword] = useState(false);
  const [selectedRole, setSelectedRole] = useState<UserRole>("buyer");
  const [forgotOpen, setForgotOpen] = useState(false);
  const [supportOpen, setSupportOpen] = useState(false);
  const [signUpOpen, setSignUpOpen] = useState(false);
  const [resetEmail, setResetEmail] = useState("");
  const [regEmail, setRegEmail] = useState("");
  const [regPassword, setRegPassword] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const { login } = useUserRole();

  const handleLogin = async () => {
    if (!username.trim() || !password.trim()) {
      toast.error(t("auth.toastFillFields"));
      return;
    }
    setLoading(true);
    try {
      await login(username.trim(), password, selectedRole);
      navigate(selectedRole === "seller" ? "/seller" : "/buyer");
    } catch (err) {
      const apiErr = err as ApiError;
      if (apiErr?.status === 401) {
        toast.error(t("auth.invalidCredentials", "Usuário ou senha incorretos"));
      } else {
        toast.error(t("auth.loginError", "Erro ao entrar. Tente novamente."));
      }
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

  const createAccount = async () => {
    if (!regEmail.trim() || !regPassword.trim()) {
      toast.error(t("auth.toastFillFields"));
      return;
    }
    setLoading(true);
    try {
      await login(regEmail.trim(), regPassword, "buyer");
      toast.success(t("auth.toastAccountCreated"));
      setSignUpOpen(false);
      setRegEmail("");
      setRegPassword("");
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

        <div className="mb-5">
          <label className="text-xs font-semibold tracking-wider uppercase text-foreground mb-2 block">{t("auth.accessType")}</label>
          <div className="grid grid-cols-2 gap-3">
            <button
              type="button"
              onClick={() => setSelectedRole("buyer")}
              className={`flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all ${
                selectedRole === "buyer"
                  ? "border-primary bg-primary/5"
                  : "border-border bg-card hover:border-muted-foreground/30"
              }`}
            >
              <ShoppingBag className={`h-6 w-6 ${selectedRole === "buyer" ? "text-primary" : "text-muted-foreground"}`} />
              <span className={`text-sm font-semibold ${selectedRole === "buyer" ? "text-primary" : "text-muted-foreground"}`}>{t("common.buyer")}</span>
            </button>
            <button
              type="button"
              onClick={() => setSelectedRole("seller")}
              className={`flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all ${
                selectedRole === "seller"
                  ? "border-primary bg-primary/5"
                  : "border-border bg-card hover:border-muted-foreground/30"
              }`}
            >
              <Store className={`h-6 w-6 ${selectedRole === "seller" ? "text-primary" : "text-muted-foreground"}`} />
              <span className={`text-sm font-semibold ${selectedRole === "seller" ? "text-primary" : "text-muted-foreground"}`}>{t("common.seller")}</span>
            </button>
          </div>
        </div>

        <div className="space-y-5 flex-1">
          <div>
            <label className="text-xs font-semibold tracking-wider uppercase text-foreground mb-2 block">{t("auth.emailOrTaxId")}</label>
            <div className="relative">
              <User className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder={t("auth.credentialsPlaceholder")}
                className="h-14 pl-11 rounded-xl bg-muted border-0 text-sm"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleLogin()}
                autoComplete="username"
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
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="reg-email">{t("common.email")}</Label>
              <Input
                id="reg-email"
                type="email"
                placeholder={t("auth.resetPlaceholder")}
                value={regEmail}
                onChange={(e) => setRegEmail(e.target.value)}
              />
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
