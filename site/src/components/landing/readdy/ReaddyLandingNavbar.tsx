import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Menu, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { BrandMark } from "@/components/app/BrandMark";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function ReaddyLandingNavbar() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  const NAV = [
    { label: t("nav.problem"), href: "#how-it-works" },
    { label: t("nav.solution"), href: "#features" },
    { label: t("nav.integration"), href: "#integration" },
    { label: t("nav.contact"), href: "#contact" },
  ] as const;

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={cn(
        "fixed inset-x-0 top-0 z-50 border-b transition-colors duration-300",
        scrolled ? "border-white/10 bg-[#0A0D0F]/95 backdrop-blur-md" : "border-transparent bg-transparent",
      )}
    >
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-5 md:h-[4.25rem] md:px-8">
        <Link to="/" className="text-white transition hover:opacity-90" aria-label={t("nav.homeAria")}>
          <BrandMark size="nav" textClassName="text-white" />
        </Link>

        <nav className="hidden items-center gap-10 md:flex" aria-label={t("nav.sections")}>
          {NAV.map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="text-sm font-medium text-white/60 transition hover:text-[#00E5A0]"
            >
              {item.label}
            </a>
          ))}
        </nav>

        <div className="hidden items-center gap-3 md:flex">
          <LanguageSwitcher variant="landing" />
          <Button variant="ghost" size="sm" className="text-white/80 hover:bg-white/10 hover:text-white" asChild>
            <Link to="/login">{t("common.signIn")}</Link>
          </Button>
          <Button
            size="sm"
            className="rounded-full border-0 bg-[#00E5A0] px-5 font-semibold text-[#0A0D0F] hover:bg-[#00c98a]"
            asChild
          >
            <Link to="/login">{t("common.getStarted")}</Link>
          </Button>
        </div>

        <button
          type="button"
          className="flex h-10 w-10 items-center justify-center rounded-lg text-white md:hidden"
          aria-expanded={open}
          aria-controls="readdy-mobile-nav"
          onClick={() => setOpen((v) => !v)}
        >
          {open ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
        </button>
      </div>

      {open && (
        <div
          id="readdy-mobile-nav"
          className="border-t border-white/10 bg-[#0A0D0F]/98 px-5 py-4 backdrop-blur-md md:hidden"
        >
          <div className="flex flex-col gap-1">
            {NAV.map((item) => (
              <a
                key={item.href}
                href={item.href}
                className="rounded-lg px-3 py-3 text-sm font-medium text-white/80 hover:bg-white/5 hover:text-[#00E5A0]"
                onClick={() => setOpen(false)}
              >
                {item.label}
              </a>
            ))}
            <div className="mt-3 pb-2">
              <LanguageSwitcher variant="landing" className="w-full" />
            </div>
            <div className="mt-3 flex flex-col gap-2 border-t border-white/10 pt-4">
              <Button variant="outline" className="border-white/20 bg-transparent text-white hover:bg-white/10" asChild>
                <Link to="/login" onClick={() => setOpen(false)}>
                  {t("common.signIn")}
                </Link>
              </Button>
              <Button className="bg-[#00E5A0] font-semibold text-[#0A0D0F] hover:bg-[#00c98a]" asChild>
                <Link to="/login" onClick={() => setOpen(false)}>
                  {t("common.getStarted")}
                </Link>
              </Button>
            </div>
          </div>
        </div>
      )}
    </header>
  );
}
