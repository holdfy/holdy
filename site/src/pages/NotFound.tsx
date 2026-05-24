import { useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";

export default function NotFound() {
  const location = useLocation();
  const { t } = useTranslation();

  useEffect(() => {
    console.error("404: non-existent route:", location.pathname);
  }, [location.pathname]);

  return (
    <div className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden bg-[#0A0D0F] px-4 text-center">
      <p
        className="pointer-events-none absolute bottom-0 select-none font-black leading-none text-[#111518] z-0 text-[10rem] md:text-[12rem]"
        aria-hidden
      >
        404
      </p>
      <div className="relative z-10 max-w-lg">
        <p className="text-xs font-bold uppercase tracking-[0.25em] text-[#00E5A0]">{t("common.holdfy")}</p>
        <h1 className="mt-4 font-display text-2xl font-semibold text-white md:text-3xl">{t("notFound.title")}</h1>
        <p className="mt-3 font-mono text-sm text-white/45">{location.pathname}</p>
        <p className="mt-5 text-base text-white/55 md:text-lg">{t("notFound.hint")}</p>
        <div className="mt-10 flex flex-wrap items-center justify-center gap-3">
          <Link
            to="/"
            className="inline-flex items-center justify-center rounded-full bg-[#00E5A0] px-6 py-3 text-sm font-semibold text-[#0A0D0F] transition hover:bg-[#00c98a]"
          >
            {t("common.home")}
          </Link>
          <Link
            to="/login"
            className="inline-flex items-center justify-center rounded-full border border-white/20 bg-white/5 px-6 py-3 text-sm font-semibold text-white transition hover:bg-white/10"
          >
            {t("common.signIn")}
          </Link>
        </div>
      </div>
    </div>
  );
}
