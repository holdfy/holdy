import { Outlet } from "react-router-dom";
import { SellerBottomNav } from "@/components/seller/SellerBottomNav";
import { SellerSidebar } from "@/components/seller/SellerSidebar";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { useIsMobile } from "@/hooks/use-mobile";

export default function SellerLayout() {
  const isMobile = useIsMobile();

  if (isMobile) {
    return (
      <div className="min-h-screen bg-background pb-24 max-w-lg mx-auto relative">
        <Outlet />
        <SellerBottomNav />
      </div>
    );
  }

  return (
    <SidebarProvider>
      <div className="min-h-screen flex w-full">
        <SellerSidebar />
        <div className="flex-1 flex flex-col min-w-0">
          <header className="h-14 flex items-center justify-between border-b border-border px-4">
            <SidebarTrigger />
            <LanguageSwitcher variant="app" />
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
