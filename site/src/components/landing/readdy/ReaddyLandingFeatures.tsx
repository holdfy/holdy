import { useEffect, useMemo, useRef, useState } from "react";
import {
  Coins,
  Eye,
  Handshake,
  Landmark,
  Link2,
  MessageCircle,
  Percent,
  RefreshCw,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { ReaddyWhatsAppChat, type ReaddyChatMessage } from "./ReaddyWhatsAppChat";

export function ReaddyLandingFeatures() {
  const { t, i18n } = useTranslation();
  const [disputeVisible, setDisputeVisible] = useState(false);
  const disputeRef = useRef<HTMLDivElement>(null);

  const features = useMemo(
    () => [
      { icon: Landmark, title: t("landing.features.f1Title"), body: t("landing.features.f1Body") },
      { icon: Link2, title: t("landing.features.f2Title"), body: t("landing.features.f2Body") },
      { icon: Percent, title: t("landing.features.f3Title"), body: t("landing.features.f3Body") },
      { icon: ShieldCheck, title: t("landing.features.f4Title"), body: t("landing.features.f4Body") },
      { icon: MessageCircle, title: t("landing.features.f5Title"), body: t("landing.features.f5Body") },
      { icon: Eye, title: t("landing.features.f6Title"), body: t("landing.features.f6Body") },
    ],
    [t, i18n.language],
  );

  const disputeMessages: ReaddyChatMessage[] = useMemo(
    () => [
      { from: "buyer", text: t("landing.features.disputeChat1") },
      { from: "agent", text: t("landing.features.disputeChat2") },
      { from: "agent", text: t("landing.features.disputeChat3") },
      { from: "buyer", text: t("landing.features.disputeChat4") },
      { from: "agent", text: t("landing.features.disputeChat5") },
      { from: "buyer", text: t("landing.features.disputeChat6") },
      { from: "agent", text: t("landing.features.disputeChat7") },
    ],
    [t, i18n.language],
  );

  const disputeListItems = useMemo(
    () => [
      { icon: Handshake, text: t("landing.features.disputeLi1") },
      { icon: Coins, text: t("landing.features.disputeLi2") },
      { icon: Sparkles, text: t("landing.features.disputeLi3") },
    ],
    [t, i18n.language],
  );

  useEffect(() => {
    const el = disputeRef.current;
    if (!el) return;
    const io = new IntersectionObserver(
      ([e]) => {
        if (e?.isIntersecting) setDisputeVisible(true);
      },
      { threshold: 0.2 },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <section id="features" className="scroll-mt-24 border-t border-white/5 bg-[#0A0D0F] py-20 md:py-28">
      <div className="mx-auto max-w-6xl px-5 md:px-8">
        <div className="mx-auto max-w-3xl text-center">
          <span className="text-xs font-bold uppercase tracking-widest text-[#00E5A0]">{t("landing.features.eyebrow")}</span>
          <h2 className="mt-3 font-display text-3xl font-bold text-white md:text-4xl">{t("landing.features.title")}</h2>
          <p className="mt-4 text-lg text-white/50">{t("landing.features.subtitle")}</p>
        </div>

        <div className="mt-16 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {features.map((f) => (
            <div
              key={f.title}
              className="rounded-2xl border border-white/10 bg-[#111518] p-8 transition hover:border-[#00E5A0]/25"
            >
              <div className="mb-5 inline-flex rounded-xl bg-[#00E5A0]/15 p-3 text-[#00E5A0]">
                <f.icon className="h-6 w-6" />
              </div>
              <h3 className="font-display text-xl font-bold text-white">{f.title}</h3>
              <p className="mt-3 leading-relaxed text-white/50">{f.body}</p>
            </div>
          ))}
        </div>

        <div
          ref={disputeRef}
          className="mt-20 grid items-center gap-12 rounded-3xl border border-[#00E5A0]/20 bg-gradient-to-br from-[#00E5A0]/5 to-transparent p-8 md:grid-cols-2 md:p-12"
        >
          <div>
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-[#ff6b6b]/30 bg-[#ff6b6b]/10 px-3 py-1 text-xs font-semibold uppercase tracking-wider text-[#ff6b6b]">
              <RefreshCw className="h-3.5 w-3.5" />
              {t("landing.features.disputeBadge")}
            </div>
            <h3 className="font-display text-2xl font-bold text-white md:text-3xl">{t("landing.features.disputeTitle")}</h3>
            <p className="mt-4 leading-relaxed text-white/50">{t("landing.features.disputeBody")}</p>
            <ul className="mt-8 space-y-4">
              {disputeListItems.map((row) => (
                <li key={row.text} className="flex items-center gap-3 text-white/70">
                  <row.icon className="h-5 w-5 shrink-0 text-[#00E5A0]" />
                  {row.text}
                </li>
              ))}
            </ul>
          </div>
          <div className={disputeVisible ? "motion-safe:animate-[readdyMsgIn_0.5s_ease-out_forwards]" : "opacity-0"}>
            <ReaddyWhatsAppChat
              key={i18n.language}
              messages={disputeMessages}
              label={t("landing.features.disputeChatLabel")}
              accentLabel={t("landing.features.disputeChatAccent")}
              glowColor="rgba(255,107,107,0.12)"
              chatHeight={340}
              scrollbarStyle="dispute"
            />
          </div>
        </div>
      </div>
    </section>
  );
}
