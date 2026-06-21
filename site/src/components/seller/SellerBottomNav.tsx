import { LayoutGrid, FileText, Wallet, User, AlertTriangle, ShoppingBag } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { NavLink, useLocation } from "react-router-dom";

export function SellerBottomNav() {
  const { t } = useTranslation();
  const location = useLocation();

  const tabs = useMemo(
    () => [
      { label: t("navSeller.dashboardShort"), icon: LayoutGrid, path: "/seller" },
      { label: t("navSeller.ordersShort"), icon: FileText, path: "/seller/orders" },
      { label: t("navSeller.disputesShort"), icon: AlertTriangle, path: "/seller/disputes" },
      { label: t("navSeller.walletShort"), icon: Wallet, path: "/seller/wallet" },
      { label: t("navSeller.profileShort"), icon: User, path: "/seller/profile" },
      { label: t("navSeller.buyModeShort"), icon: ShoppingBag, path: "/buyer", isBuySwitch: true },
    ],
    [t],
  );

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 bg-white border-t border-border safe-area-bottom">
      <div className="flex items-center justify-around h-16 max-w-lg mx-auto">
        {tabs.map((tab) => {
          const isBuySwitch = "isBuySwitch" in tab && tab.isBuySwitch;
          const isActive = !isBuySwitch &&
            (location.pathname === tab.path || (tab.path !== "/seller" && location.pathname.startsWith(tab.path)));
          return (
            <NavLink
              key={tab.path}
              to={tab.path}
              className="flex flex-col items-center gap-1 px-1 py-2 min-w-[48px]"
            >
              <div
                className={`flex items-center justify-center w-8 h-8 rounded-xl transition-all ${
                  isBuySwitch
                    ? "bg-primary/10 text-primary"
                    : isActive
                    ? "vault-card shadow-lg"
                    : "text-muted-foreground"
                }`}
              >
                <tab.icon className={`h-3.5 w-3.5 ${isActive ? "text-white" : ""}`} />
              </div>
              <span
                className={`text-[8px] font-semibold tracking-wider ${
                  isBuySwitch ? "text-primary" : isActive ? "text-foreground" : "text-muted-foreground"
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
