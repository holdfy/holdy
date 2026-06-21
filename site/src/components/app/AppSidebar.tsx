import { LayoutGrid, FileText, Wallet, User, Shield, LogOut, Store } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { NavLink } from "@/components/NavLink";
import { useLocation } from "react-router-dom";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarHeader,
  SidebarFooter,
  useSidebar,
} from "@/components/ui/sidebar";

export function AppSidebar() {
  const { t } = useTranslation();
  const { state } = useSidebar();
  const collapsed = state === "collapsed";
  const location = useLocation();

  const menuItems = useMemo(
    () => [
      { title: t("navBuyer.home"), url: "/buyer", icon: LayoutGrid },
      { title: t("navBuyer.orders"), url: "/buyer/orders", icon: FileText },
      { title: t("navBuyer.wallet"), url: "/buyer/wallet", icon: Wallet },
      { title: t("navBuyer.profile"), url: "/buyer/profile", icon: User },
    ],
    [t],
  );

  const isActive = (path: string) =>
    location.pathname === path || (path !== "/buyer" && location.pathname.startsWith(path));

  return (
    <Sidebar collapsible="icon" className="border-r border-border">
      <SidebarHeader className="p-4">
        <div className="flex items-center gap-3">
          <div className="h-9 w-9 rounded-xl vault-card flex items-center justify-center flex-shrink-0">
            <Shield className="h-4 w-4 text-white" />
          </div>
          {!collapsed && (
            <div>
              <span className="font-display font-bold text-lg">{t("common.holdfy")}</span>
              <span className="ml-2 text-[10px] bg-secondary/10 text-secondary px-1.5 py-0.5 rounded-full font-semibold">{t("common.buyer")}</span>
            </div>
          )}
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>{t("common.menu")}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {menuItems.map((item) => (
                <SidebarMenuItem key={item.url}>
                  <SidebarMenuButton asChild isActive={isActive(item.url)} tooltip={item.title}>
                    <NavLink
                      to={item.url}
                      end={item.url === "/buyer"}
                      className="hover:bg-muted/50"
                      activeClassName="bg-primary/10 text-primary font-medium"
                    >
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="p-4 space-y-1">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild tooltip={t("navBuyer.sellMode")}>
              <a href="/seller" className="text-secondary hover:text-secondary/80">
                <Store className="h-4 w-4" />
                <span>{t("navBuyer.sellMode")}</span>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton asChild tooltip={t("common.logout")}>
              <a href="/login" className="text-muted-foreground hover:text-foreground">
                <LogOut className="h-4 w-4" />
                <span>{t("common.logout")}</span>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
