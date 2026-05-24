import { useMemo } from "react";
import { Check, Heart, ShoppingBag, Store } from "lucide-react";
import { Trans, useTranslation } from "react-i18next";

export function ReaddyLandingBenefits() {
  const { t } = useTranslation();

  const benefits = useMemo(
    () => [t("landing.benefits.b1"), t("landing.benefits.b2"), t("landing.benefits.b3"), t("landing.benefits.b4")],
    [t],
  );

  const useCases = useMemo(
    () => [
      {
        title: t("landing.benefits.uc1Title"),
        body: t("landing.benefits.uc1Body"),
        icon: Store,
        img: "/use-cases/whatsapp.png",
      },
      {
        title: t("landing.benefits.uc2Title"),
        body: t("landing.benefits.uc2Body"),
        icon: Heart,
        img: "/use-cases/social-media.png",
      },
      {
        title: t("landing.benefits.uc3Title"),
        body: t("landing.benefits.uc3Body"),
        icon: ShoppingBag,
        img: "/use-cases/olx.png",
      },
    ],
    [t],
  );

  const defiSplit = useMemo(
    () => [
      { pct: "70%", l: t("landing.benefits.defiSeller") },
      { pct: "10%", l: t("landing.benefits.defiBuyer") },
      { pct: "20%", l: t("landing.benefits.defiPlatform") },
    ],
    [t],
  );

  return (
    <section id="benefits" className="scroll-mt-24 border-t border-white/5 bg-[#0D1114] py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-5 md:px-8">
        <div className="grid gap-8 lg:grid-cols-2 lg:items-start">
          <div>
            <h3 className="font-display text-2xl font-bold text-white md:text-3xl">{t("landing.benefits.title")}</h3>
            <p className="mt-4 leading-relaxed text-white/50">{t("landing.benefits.subtitle")}</p>
            <ul className="mt-8 space-y-4">
              {benefits.map((item) => (
                <li key={item} className="flex gap-3 text-white/70">
                  <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#00E5A0]/20 text-[#00E5A0]">
                    <Check className="h-3.5 w-3.5" />
                  </span>
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="rounded-3xl border border-white/10 bg-[#111518] p-8 md:p-10">
            <div className="mb-2 text-xs font-bold uppercase tracking-widest text-[#00E5A0]">{t("landing.benefits.defiEyebrow")}</div>
            <p className="font-display text-xl font-bold text-white">{t("landing.benefits.defiTitle")}</p>
            <p className="mt-3 text-sm leading-relaxed text-white/50">
              <Trans
                i18nKey="landing.benefits.defiBody"
                components={{ strong: <strong className="text-white" /> }}
              />
            </p>
            <div className="mt-8 grid grid-cols-3 gap-3 text-center">
              {defiSplit.map((row) => (
                <div key={row.l} className="rounded-xl border border-white/10 bg-[#0A0D0F] px-2 py-4">
                  <p className="font-display text-2xl font-bold text-[#00E5A0]">{row.pct}</p>
                  <p className="mt-1 text-xs text-white/45">{row.l}</p>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="mt-24">
          <div className="mx-auto max-w-2xl text-center">
            <span className="text-xs font-bold uppercase tracking-widest text-[#00E5A0]">{t("landing.benefits.useCasesEyebrow")}</span>
            <h3 className="mt-3 font-display text-2xl font-bold text-white md:text-3xl">{t("landing.benefits.useCasesTitle")}</h3>
          </div>
          <div className="mt-12 grid gap-8 md:grid-cols-3">
            {useCases.map((uc) => (
              <div
                key={uc.title}
                className="group overflow-hidden rounded-2xl border border-white/10 bg-[#111518] transition hover:border-[#00E5A0]/30"
              >
                <div className="relative h-44 overflow-hidden">
                  <img
                    src={uc.img}
                    alt=""
                    className="h-full w-full object-cover transition duration-500 group-hover:scale-105"
                    loading="lazy"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-[#111518] to-transparent" />
                  <div className="absolute bottom-4 left-4 flex h-10 w-10 items-center justify-center rounded-xl bg-[#00E5A0]/20 text-[#00E5A0]">
                    <uc.icon className="h-5 w-5" />
                  </div>
                </div>
                <div className="p-6">
                  <h4 className="font-display text-lg font-bold text-white">{uc.title}</h4>
                  <p className="mt-2 text-sm leading-relaxed text-white/50">{uc.body}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
