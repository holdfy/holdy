import { useEffect, useRef, useState } from "react";
import { CheckCheck, MoreHorizontal, Phone, Send, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";

export type ReaddyChatMessage = { from: "buyer" | "agent"; text: string };

function TypingDots() {
  return (
    <div className="flex items-center gap-1 px-3 py-2.5">
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className="h-1.5 w-1.5 rounded-full bg-[#8696a0] motion-safe:animate-[readdyTyping_1.2s_ease-in-out_infinite]"
          style={{ animationDelay: `${i * 0.2}s` }}
        />
      ))}
    </div>
  );
}

export function ReaddyWhatsAppChat({
  messages,
  label,
  accentLabel,
  glowColor = "rgba(0,229,160,0.12)",
  chatHeight = 280,
  scrollbarStyle = "default",
}: {
  messages: ReaddyChatMessage[];
  label: string;
  accentLabel: string;
  glowColor?: string;
  chatHeight?: number;
  scrollbarStyle?: "default" | "dispute";
}) {
  const { t } = useTranslation();
  const [visibleMessages, setVisibleMessages] = useState<ReaddyChatMessage[]>([]);
  const [isTyping, setIsTyping] = useState(false);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [started, setStarted] = useState(false);
  const chatBodyRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const timer = window.setTimeout(() => setStarted(true), 800);
    return () => window.clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (!started) return;
    if (currentIndex >= messages.length) {
      const restart = window.setTimeout(() => {
        setVisibleMessages([]);
        setCurrentIndex(0);
      }, 4000);
      return () => window.clearTimeout(restart);
    }

    const msg = messages[currentIndex];
    const typingDelay = msg.from === "agent" ? 1200 : 600;

    setIsTyping(true);
    const typingTimer = window.setTimeout(() => {
      setIsTyping(false);
      setVisibleMessages((prev) => [...prev, msg]);
      setCurrentIndex((prev) => prev + 1);
    }, typingDelay);

    return () => window.clearTimeout(typingTimer);
  }, [started, currentIndex, messages]);

  useEffect(() => {
    const body = chatBodyRef.current;
    if (!body) return;
    body.scrollTop = body.scrollHeight;
  }, [visibleMessages, isTyping]);

  return (
    <div
      className="relative overflow-hidden rounded-3xl border border-[#00E5A0]/20 motion-safe:animate-[readdyFloat_4s_ease-in-out_infinite]"
      style={{
        boxShadow: `0 0 60px ${glowColor}, 0 0 120px rgba(0,229,160,0.04)`,
      }}
    >
      <div className="flex items-center justify-between bg-[#075E54] px-4 py-1.5 text-[10px] font-medium text-white">
        <span className="w-8" />
        <span className="tabular-nums text-white/90">9:41</span>
        <span className="w-8 text-right text-white/80">LTE</span>
      </div>

      <div className="flex items-center gap-3 bg-[#075E54] px-3 py-2.5 text-white">
        <div className="relative shrink-0">
          <div className="flex h-10 w-10 items-center justify-center rounded-full border border-[#00E5A0]/50 bg-[#00E5A0]/20">
            <span className="text-xs font-bold text-[#00E5A0]">A</span>
          </div>
          <span className="absolute bottom-0 right-0 h-2.5 w-2.5 rounded-full border-2 border-[#075E54] bg-[#25D366] motion-safe:animate-[readdyOnline_2s_ease-in-out_infinite]" />
        </div>
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold leading-none text-white">{t("common.holdfy")}</p>
          <p className="mt-0.5 text-xs text-white/70">{isTyping ? t("common.typing") : t("common.online")}</p>
        </div>
        <div className="flex items-center gap-3 text-white/80">
          <Phone className="h-4 w-4" aria-hidden />
          <MoreHorizontal className="h-4 w-4" aria-hidden />
        </div>
      </div>

      <div
        ref={chatBodyRef}
        className={`flex flex-col gap-2 overflow-y-auto px-3 py-4 ${
          scrollbarStyle === "dispute" ? "readdy-chat-scrollbar-dispute" : ""
        }`}
        style={{
          height: `${chatHeight}px`,
          overscrollBehavior: "contain",
          background: "linear-gradient(180deg, #0b1a14 0%, #0d1f17 100%)",
        }}
      >
        {visibleMessages.map((msg, i) => (
          <div
            key={`${i}-${msg.text.slice(0, 12)}`}
            className={`flex ${msg.from === "buyer" ? "justify-start" : "justify-end"} motion-safe:animate-[readdyMsgIn_0.3s_ease-out_forwards]`}
          >
            <div
              className={`max-w-[82%] rounded-2xl px-3 py-2 text-[13px] leading-snug ${
                msg.from === "buyer"
                  ? "rounded-tl-sm bg-white text-[#303030]"
                  : "rounded-tr-sm bg-[#005c4b] text-[#e9edef]"
              }`}
            >
              <p>{msg.text}</p>
              <div className="mt-1 flex items-center justify-end gap-1">
                <span className="text-[10px] opacity-60">
                  {String(9 + Math.floor(i / 2)).padStart(2, "0")}:{String(41 + i * 3).padStart(2, "0")}
                </span>
                {msg.from === "agent" && <CheckCheck className="h-3 w-3 text-[#00E5A0]" aria-hidden />}
              </div>
            </div>
          </div>
        ))}

        {isTyping && (
          <div className="flex justify-start motion-safe:animate-[readdyMsgIn_0.2s_ease-out_forwards]">
            <div className="rounded-2xl rounded-tl-sm border border-white/10 bg-white/10">
              <TypingDots />
            </div>
          </div>
        )}
      </div>

      <div className="flex items-center gap-2 bg-[#1f2c33] px-3 py-2.5">
        <div className="flex-1 rounded-full bg-[#2a3942] px-4 py-2 text-xs text-[#8696a0]">{t("common.message")}</div>
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#00E5A0]">
          <Send className="h-3.5 w-3.5 text-[#0A0D0F]" aria-hidden />
        </div>
      </div>

      <div className="flex items-center justify-between border-t border-[#00E5A0]/10 bg-[#0b1a14] px-4 py-3">
        <div className="flex items-center gap-2">
          <ShieldCheck className="h-4 w-4 text-[#00E5A0]" aria-hidden />
          <span className="text-xs font-semibold text-[#00E5A0]">{label}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-[#00E5A0]" />
          <span className="text-xs text-white/40">{accentLabel}</span>
        </div>
      </div>
    </div>
  );
}
