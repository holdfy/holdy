import { LayoutGrid, FileText, Wallet, User } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { NavLink, useLocation } from "react-router-dom";

export function BottomNav() {
  const { t } = useTranslation();
  const location = useLocation();

  const tabs = useMemo(
    () => [
      { label: t("navBuyer.homeShort"), icon: LayoutGrid, path: "/buyer" },
      { label: t("navBuyer.ordersShort"), icon: FileText, path: "/buyer/orders" },
      { label: t("navBuyer.walletShort"), icon: Wallet, path: "/buyer/wallet" },
      { label: t("navBuyer.profileShort"), icon: User, path: "/buyer/profile" },
    ],
    [t],
  );

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 bg-white border-t border-border safe-area-bottom">
      <div className="flex items-center justify-around h-16 max-w-lg mx-auto">
        {tabs.map((tab) => {
          const isActive =
            location.pathname === tab.path || (tab.path !== "/buyer" && location.pathname.startsWith(tab.path));
          return (
            <NavLink
              key={tab.path}
              to={tab.path}
              className="flex flex-col items-center gap-1 px-2 py-2 min-w-[56px]"
            >
              <div
                className={`flex items-center justify-center w-9 h-9 rounded-xl transition-all ${
                  isActive ? "vault-card shadow-lg" : "text-muted-foreground"
                }`}
              >
                <tab.icon className={`h-4 w-4 ${isActive ? "text-white" : ""}`} />
              </div>
              <span
                className={`text-[9px] font-semibold tracking-wider ${
                  isActive ? "text-foreground" : "text-muted-foreground"
                }`}
              >
                {tab.label}
              </span>
            </NavLink>
          );
        })}
      </div>
    </nav>
  );
}
