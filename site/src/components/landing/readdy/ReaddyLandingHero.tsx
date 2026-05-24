import { useMemo } from "react";
import { Link } from "react-router-dom";
import { ArrowRight, Shield } from "lucide-react";
import { Trans, useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { ReaddyWhatsAppChat, type ReaddyChatMessage } from "./ReaddyWhatsAppChat";

export function ReaddyLandingHero() {
  const { t, i18n } = useTranslation();

  const heroMessages: ReaddyChatMessage[] = useMemo(
    () => [
      { from: "buyer", text: t("landing.hero.chat1") },
      { from: "agent", text: t("landing.hero.chat2") },
      { from: "buyer", text: t("landing.hero.chat3") },
      { from: "agent", text: t("landing.hero.chat4") },
    ],
    [t, i18n.language],
  );

  const stats = useMemo(
    () => [
      { n: "1.5–2.5%", l: t("landing.hero.statFee") },
      { n: "70/10/20", l: t("landing.hero.statYield") },
      { n: "< 5s", l: t("landing.hero.statSettlement") },
    ],
    [t, i18n.language],
  );

  return (
    <section className="relative overflow-hidden bg-[#0A0D0F] pt-24 pb-8 md:pt-28 md:pb-10">
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.35]"
        style={{
          background:
            "radial-gradient(ellipse 80% 50% at 50% -20%, rgba(0,229,160,0.15), transparent), radial-gradient(ellipse 60% 40% at 100% 0%, rgba(0,229,160,0.06), transparent)",
        }}
        aria-hidden
      />
      <div className="pointer-events-none absolute inset-0 landing-grid opacity-[0.08]" aria-hidden />

      <div className="relative z-[1] mx-auto grid max-w-6xl items-center gap-12 px-5 md:px-8 lg:grid-cols-[1.05fr_1fr] lg:gap-16">
        <div>
          <p className="mb-5 inline-flex items-center gap-2 rounded-full border border-[#00E5A0]/25 bg-[#00E5A0]/10 px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-[#00E5A0]">
            {t("landing.hero.badge")}
          </p>
          <h1 className="font-display text-4xl font-bold leading-[1.08] tracking-tight text-white md:text-5xl lg:text-[3.25rem]">
            {t("landing.hero.titleBefore")}{" "}
            <span className="bg-gradient-to-r from-[#00E5A0] to-emerald-300 bg-clip-text text-transparent">
              {t("landing.hero.protect")}
            </span>{" "}
            {t("landing.hero.titleMiddle")}{" "}
            <span className="bg-gradient-to-r from-emerald-300 to-[#00E5A0] bg-clip-text text-transparent">
              {t("landing.hero.earn")}
            </span>
            .
          </h1>
          <p className="mt-6 max-w-xl text-lg leading-relaxed text-white/55 md:text-xl">
            <Trans
              i18nKey="landing.hero.subtitle"
              components={{ strong: <strong className="font-semibold text-white" /> }}
            />
          </p>
          <div className="mt-10 flex flex-wrap items-center gap-4">
            <Button
              size="lg"
              className="h-14 rounded-full border-0 bg-[#00E5A0] px-8 text-base font-semibold text-[#0A0D0F] hover:bg-[#00c98a]"
              asChild
            >
              <Link to="/login">
                {t("common.getStarted")}
                <ArrowRight className="ml-2 h-5 w-5" />
              </Link>
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="h-14 rounded-full border-white/20 bg-white/5 px-8 text-base font-semibold text-white hover:bg-white/10"
              asChild
            >
              <a href="#how-it-works">{t("landing.hero.seeProblem")}</a>
            </Button>
          </div>

          <div className="mt-12 grid w-full max-w-md grid-cols-3 gap-3 sm:max-w-lg">
            {stats.map((item) => (
              <div
                key={item.l}
                className="rounded-2xl border border-white/10 bg-[#111518] px-3 py-4 text-left shadow-lg shadow-black/20"
              >
                <p className="font-mono text-lg font-bold text-[#00E5A0] md:text-xl">{item.n}</p>
                <p className="mt-1 text-[10px] leading-tight text-white/45 md:text-xs">{item.l}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="relative mx-auto w-full max-w-md lg:max-w-none">
          <div
            className="pointer-events-none absolute -inset-8 -z-10 rounded-[2.5rem] blur-2xl md:-inset-12"
            style={{
              background: "radial-gradient(circle at 40% 40%, rgba(0,229,160,0.2), transparent 55%)",
            }}
            aria-hidden
          />
          <div className="absolute -right-1 -top-1 z-10 flex h-14 w-14 items-center justify-center rounded-2xl border border-[#00E5A0]/35 bg-[#111518] shadow-xl md:-right-2 md:-top-2 md:h-16 md:w-16">
            <Shield className="h-7 w-7 text-[#00E5A0] md:h-8 md:w-8" />
          </div>
          <div className="space-y-4 pt-2">
            <ReaddyWhatsAppChat
              key={i18n.language}
              messages={heroMessages}
              label={t("landing.hero.chatLabel")}
              accentLabel={t("landing.hero.chatAccent")}
              chatHeight={300}
            />
            <p className="text-center font-display text-xs font-semibold uppercase tracking-[0.2em] text-white/40 md:text-sm">
              {t("landing.hero.chatCaption")}
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
