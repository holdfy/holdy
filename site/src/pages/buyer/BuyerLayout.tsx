import { Outlet } from "react-router-dom";
import { BottomNav } from "@/components/app/BottomNav";
import { AppSidebar } from "@/components/app/AppSidebar";
import { ModeSwitcher, ModeSwitcherStrip } from "@/components/app/ModeIndicator";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { useIsMobile } from "@/hooks/use-mobile";

export default function BuyerLayout() {
  const isMobile = useIsMobile();

  if (isMobile) {
    return (
      <div className="min-h-screen bg-background pb-24 max-w-lg mx-auto relative">
        <ModeSwitcherStrip mode="buyer" />
        <Outlet />
        <BottomNav />
      </div>
    );
  }

  return (
    <SidebarProvider>
      <div className="min-h-screen flex w-full">
        <AppSidebar />
        <div className="flex-1 flex flex-col min-w-0">
          <header className="h-14 flex items-center justify-between border-b border-border px-4">
            <SidebarTrigger />
            <div className="flex items-center gap-3">
              <ModeSwitcher mode="buyer" />
              <LanguageSwitcher variant="app" />
            </div>
          </header>
          <main className="flex-1 overflow-auto">
            <div className="max-w-6xl mx-auto p-6">
              <Outlet />
            </div>
          </main>
        </div>
      </div>
    </SidebarProvider>
  );
}
