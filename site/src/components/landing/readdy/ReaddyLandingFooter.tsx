import { useMemo } from "react";
import { Link } from "react-router-dom";
import { Mail, MapPin, Phone } from "lucide-react";
import { useTranslation } from "react-i18next";
import { BrandMark } from "@/components/app/BrandMark";

export function ReaddyLandingFooter() {
  const { t } = useTranslation();

  const links = useMemo(
    () => [
      { label: t("nav.problem"), href: "#how-it-works" },
      { label: t("nav.solution"), href: "#features" },
      { label: t("nav.contact"), href: "#contact" },
    ],
    [t],
  );

  return (
    <footer id="contact" className="scroll-mt-24 border-t border-white/10 bg-[#0A0D0F] py-14">
      <div className="mx-auto max-w-6xl px-5 md:px-8">
        <div className="grid gap-10 md:grid-cols-[1.5fr_1fr_1fr]">
          <div>
            <BrandMark size="nav" className="text-white" textClassName="text-white" />
            <p className="mt-4 max-w-sm text-sm leading-relaxed text-white/45">{t("landing.footer.tagline")}</p>
          </div>
          <div>
            <h4 className="font-display text-sm font-bold uppercase tracking-wider text-white">{t("common.product")}</h4>
            <ul className="mt-4 space-y-3">
              {links.map((l) => (
                <li key={l.href}>
                  <a href={l.href} className="text-sm text-white/50 transition hover:text-[#00E5A0]">
                    {l.label}
                  </a>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <h4 className="font-display text-sm font-bold uppercase tracking-wider text-white">{t("common.contact")}</h4>
            <ul className="mt-4 space-y-3">
              <li>
                <a
                  href="mailto:contato@holdfy.com.br"
                  className="group flex items-center gap-2.5 rounded-lg transition hover:text-[#00E5A0]"
                >
                  <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[#00E5A0]/15 text-[#00E5A0]">
                    <Mail className="h-3.5 w-3.5" />
                  </span>
                  <span className="min-w-0 leading-tight">
                    <span className="block text-[11px] text-white/45 group-hover:text-[#00E5A0]/80">{t("common.email")}</span>
                    <span className="text-sm font-medium text-white group-hover:text-[#00E5A0]">contato@holdfy.com.br</span>
                  </span>
                </a>
              </li>
              <li className="flex items-center gap-2.5">
                <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[#00E5A0]/15 text-[#00E5A0]">
                  <Phone className="h-3.5 w-3.5" />
                </span>
                <span className="min-w-0 leading-tight">
                  <span className="block text-[11px] text-white/45">{t("common.phone")}</span>
                  <a
                    href="tel:+5511983952506"
                    className="text-sm font-medium text-white transition hover:text-[#00E5A0]"
                  >
                    +55 (11) 98395-2506
                  </a>
                </span>
              </li>
              <li className="flex items-center gap-2.5">
                <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[#00E5A0]/15 text-[#00E5A0]">
                  <MapPin className="h-3.5 w-3.5" />
                </span>
                <span className="min-w-0 leading-tight">
                  <span className="block text-[11px] text-white/45">{t("common.address")}</span>
                  <span className="text-sm font-medium text-white">Avenida Paulista, 777 - São Paulo</span>
                </span>
              </li>
            </ul>
          </div>
        </div>
        <div className="mt-12 flex flex-col items-center justify-between gap-4 border-t border-white/10 pt-8 md:flex-row">
          <p className="text-center text-xs text-white/40 md:text-left">
            {t("common.copyright", { year: new Date().getFullYear() })}
          </p>
          <Link to="/login" className="text-sm font-semibold text-[#00E5A0] hover:underline">
            {t("common.signIn")}
          </Link>
        </div>
      </div>
    </footer>
  );
}
