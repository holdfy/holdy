import { useMemo } from "react";
import {
  AlertTriangle,
  MessageCircle,
  ShieldCheck,
  Store,
  Users,
  Zap,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { LandingScrollCue } from "./LandingScrollCue";

export function ReaddyLandingHowItWorks() {
  const { t } = useTranslation();

  const steps = useMemo(
    () => [
      { n: "01", title: t("landing.problem.step1Title"), body: t("landing.problem.step1Body"), icon: MessageCircle },
      { n: "02", title: t("landing.problem.step2Title"), body: t("landing.problem.step2Body"), icon: Zap },
      { n: "03", title: t("landing.problem.step3Title"), body: t("landing.problem.step3Body"), icon: ShieldCheck },
      { n: "04", title: t("landing.problem.step4Title"), body: t("landing.problem.step4Body"), icon: Users },
    ],
    [t],
  );

  const problemCards = useMemo(
    () => [
      { icon: AlertTriangle, title: t("landing.problem.card1Title"), body: t("landing.problem.card1Body") },
      { icon: MessageCircle, title: t("landing.problem.card2Title"), body: t("landing.problem.card2Body") },
      { icon: Users, title: t("landing.problem.card3Title"), body: t("landing.problem.card3Body") },
      { icon: Store, title: t("landing.problem.card4Title"), body: t("landing.problem.card4Body") },
    ],
    [t],
  );

  return (
    <>
      <div className="bg-[#0A0D0F]">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-center px-5 py-5 md:py-6">
          <LandingScrollCue targetId="how-it-works" />
        </div>
      </div>

      <section id="how-it-works" className="scroll-mt-20 border-t border-white/[0.06] bg-[#0D1114] pt-10 pb-16 md:pt-12 md:pb-20">
        <div className="mx-auto max-w-6xl px-5 md:px-8">
          <div className="mx-auto max-w-2xl text-center">
            <span className="text-xs font-bold uppercase tracking-widest text-[#ff6b6b]">{t("landing.problem.eyebrow")}</span>
            <h2 className="mt-3 font-display text-3xl font-bold text-white md:text-4xl">{t("landing.problem.title")}</h2>
            <p className="mt-4 text-lg text-white/50">{t("landing.problem.subtitle")}</p>
          </div>

          <div className="mt-10 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
            {problemCards.map((card) => (
              <div
                key={card.title}
                className="rounded-2xl border border-white/10 bg-[#111518] p-6 transition hover:border-[#00E5A0]/25"
              >
                <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-[#ff6b6b]/15 text-[#ff6b6b]">
                  <card.icon className="h-5 w-5" />
                </div>
                <h3 className="mt-4 font-display text-lg font-bold text-white">{card.title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-white/50">{card.body}</p>
              </div>
            ))}
          </div>

          <div className="mt-24 border-t border-white/5 pt-20">
            <div className="mx-auto max-w-2xl text-center">
              <span className="text-xs font-bold uppercase tracking-widest text-[#00E5A0]">{t("landing.problem.stepsEyebrow")}</span>
              <h3 className="mt-3 font-display text-2xl font-bold text-white md:text-3xl">{t("landing.problem.stepsTitle")}</h3>
            </div>
            <div className="mt-14 grid gap-8 md:grid-cols-2">
              {steps.map((step) => (
                <div
                  key={step.n}
                  className="group relative overflow-hidden rounded-2xl border border-white/10 bg-[#111518] p-8 transition hover:border-[#00E5A0]/30"
                >
                  <span
                    className="pointer-events-none absolute right-6 top-6 font-mono text-5xl font-bold leading-none text-[#00E5A0]/10 transition group-hover:text-[#00E5A0]/20 md:right-8 md:text-6xl"
                    aria-hidden
                  >
                    {step.n}
                  </span>
                  <div className="relative z-10 max-w-[85%] pr-2">
                    <div className="mb-4 inline-flex rounded-xl bg-[#00E5A0]/15 p-3 text-[#00E5A0]">
                      <step.icon className="h-6 w-6" />
                    </div>
                    <h4 className="font-display text-xl font-bold text-white">{step.title}</h4>
                    <p className="mt-3 leading-relaxed text-white/50">{step.body}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
