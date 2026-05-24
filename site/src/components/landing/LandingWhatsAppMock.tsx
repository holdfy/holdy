import { useEffect, useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

/** Preset conversations — same UI flow (typing → buyer → typing → agent → ticks). */
export const WHATSAPP_MOCK_SCENARIOS = {
  payment: {
    customer: "Can I pay safely?",
    agent:
      "Yes! I generated your Holdfy link — pay with PIX; we hold the funds until delivery is confirmed.",
    loopMs: 8200,
  },
  dispute: {
    customer: "The seller sent the wrong item. I need to open a dispute.",
    agent:
      "Dispute #2841 is open. Upload photos + your tracking ID in the secure link I sent — we review within 48h and funds stay held until it’s resolved.",
    loopMs: 9600,
  },
} as const;

export type WhatsAppMockScenario = keyof typeof WHATSAPP_MOCK_SCENARIOS;

type Theme = "light" | "dark";

const themes: Record<
  Theme,
  {
    shell: string;
    header: string;
    headerSub: string;
    chat: string;
    incoming: string;
    incomingText: string;
    outgoing: string;
    outgoingText: string;
    typingBg: string;
    typingDot: string;
    onlineDot: string;
    onlineRing: string;
    metaMuted: string;
  }
> = {
  light: {
    shell: "border-border bg-card shadow-xl shadow-primary/10",
    header: "bg-[#075E54] text-white",
    headerSub: "text-white/70",
    chat: "wa-chat-bg",
    incoming: "bg-white text-[#303030] shadow-sm",
    incomingText: "text-[#303030]",
    outgoing: "bg-[#DCF8C6] text-[#303030] shadow-sm",
    outgoingText: "text-[#303030]",
    typingBg: "bg-white text-[#54656f]",
    typingDot: "bg-[#8696a0]",
    onlineDot: "bg-[#25D366]",
    onlineRing: "border-[#075E54]",
    metaMuted: "text-[#667781]",
  },
  dark: {
    shell: "border-white/10 bg-[#0b141a] shadow-2xl shadow-black/40",
    header: "bg-[#1f2c33] text-[#e9edef]",
    headerSub: "text-[#8696a0]",
    chat: "bg-[#0b141a]",
    incoming: "bg-[#1f2c33] text-[#e9edef]",
    incomingText: "text-[#e9edef]",
    outgoing: "bg-[#005c4b] text-[#e9edef]",
    outgoingText: "text-[#e9edef]",
    typingBg: "bg-[#1f2c33] text-[#8696a0]",
    typingDot: "bg-[#8696a0]",
    onlineDot: "bg-[#25D366]",
    onlineRing: "border-[#1f2c33]",
    metaMuted: "text-emerald-100/75",
  },
};

function TypingDots({ dotClass }: { dotClass: string }) {
  return (
    <div className="flex items-center gap-1.5 px-1 py-0.5" aria-hidden>
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className={cn("h-2 w-2 rounded-full wa-typing-dot", dotClass)}
          style={{ animationDelay: `${i * 0.12}s` }}
        />
      ))}
    </div>
  );
}

export function LandingWhatsAppMock({
  className,
  theme = "light",
  compact = false,
  scenario = "payment",
}: {
  className?: string;
  theme?: Theme;
  /** Tighter padding + smaller text for embedded blocks */
  compact?: boolean;
  /** `payment` = hero-style trust/PIX; `dispute` = example dispute flow for agent section */
  scenario?: WhatsAppMockScenario;
}) {
  const reduceMotion = useReducedMotion();
  const [stage, setStage] = useState(0);
  const [loopKey, setLoopKey] = useState(0);
  const t = themes[theme];
  const copy = WHATSAPP_MOCK_SCENARIOS[scenario];
  const loopMs = copy.loopMs;

  useEffect(() => {
    if (reduceMotion) {
      setStage(4);
      return undefined;
    }
    setStage(0);
    const timers = [
      window.setTimeout(() => setStage(1), 500),
      window.setTimeout(() => setStage(2), 1400),
      window.setTimeout(() => setStage(3), 2200),
      window.setTimeout(() => setStage(4), 3200),
      window.setTimeout(() => setLoopKey((k) => k + 1), loopMs),
    ];
    return () => timers.forEach(clearTimeout);
  }, [reduceMotion, loopKey, loopMs, scenario]);

  const showLeftTyping = stage === 0;
  const showCustomer = stage >= 1;
  const showRightTyping = stage === 2;
  const showAgent = stage >= 3;
  const showTicks = stage >= 4;

  /** Minimum chat body height ≈ final bubbles (avoids layout jump without stretching between messages) */
  const chatMinHeight = cn(
    compact
      ? scenario === "dispute"
        ? "min-h-[11.5rem]"
        : "min-h-[10.5rem]"
      : scenario === "dispute"
        ? "min-h-[12.5rem]"
        : "min-h-[11rem]",
  );

  /** Outgoing column: reserve roughly one agent bubble so height does not pop when it appears */
  const outgoingSlotMin = compact
    ? scenario === "dispute"
      ? "min-h-[7.75rem]"
      : "min-h-[6.5rem]"
    : scenario === "dispute"
      ? "min-h-[8.5rem]"
      : "min-h-[7rem]";

  return (
    <div
      className={cn(
        "overflow-hidden rounded-[2rem] border",
        t.shell,
        compact ? "rounded-2xl" : "rounded-[2rem]",
        className,
      )}
    >
      {/* Phone status */}
      <div className={cn("flex items-center justify-between px-4 py-2 text-[10px] font-medium", t.header)}>
        <span className="w-8" />
        <span className="tabular-nums text-white/90">9:41</span>
        <span className="w-8 text-right text-white/80">LTE</span>
      </div>

      {/* WA header */}
      <div className={cn("flex items-center gap-3 px-3 py-2.5", t.header)}>
        <div className="relative">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-white/15 text-xs font-bold text-white">
            AC
          </div>
          <span
            className={cn(
              "absolute bottom-0 right-0 h-2.5 w-2.5 rounded-full border-2",
              t.onlineRing,
              t.onlineDot,
              !reduceMotion && "wa-online-pulse",
            )}
          />
        </div>
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium text-white">Holdfy</p>
          <p className={cn("text-xs", t.headerSub)}>online</p>
        </div>
      </div>

      {/* Chat — min-height + no layout animations so parent section height stays stable */}
      <div className={cn("flex flex-col gap-3 px-3 py-4", t.chat, compact ? "py-3" : "py-4", chatMinHeight)}>
        {/* Left slot: typing or customer */}
        <div className="flex min-h-[3.25rem] justify-start">
          <div className="max-w-[92%]">
            {showLeftTyping && (
              <motion.div
                initial={false}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.96 }}
                className={cn(
                  "inline-flex rounded-2xl rounded-tl-sm px-3 py-2",
                  t.typingBg,
                  !reduceMotion && "wa-bubble-pop",
                )}
              >
                <TypingDots dotClass={t.typingDot} />
              </motion.div>
            )}
            {showCustomer && (
              <motion.div
                initial={reduceMotion ? false : { opacity: 0, x: -12, scale: 0.98 }}
                animate={{ opacity: 1, x: 0, scale: 1 }}
                transition={{ type: "spring", stiffness: 420, damping: 28 }}
                className={cn(
                  "rounded-2xl rounded-tl-sm px-3 py-2.5 text-sm leading-snug",
                  t.incoming,
                  compact && "text-[13px]",
                  t.incomingText,
                )}
              >
                {copy.customer}
              </motion.div>
            )}
          </div>
        </div>

        {/* Right slot: reserve height of outgoing bubble only; align content to top-right (no flex-1 gap) */}
        <div className="flex justify-end">
          <div className={cn("relative flex max-w-[92%] flex-col items-end justify-start", outgoingSlotMin)}>
            {showRightTyping && (
              <motion.div
                initial={false}
                animate={{ opacity: 1, scale: 1 }}
                className={cn(
                  "ml-auto inline-flex rounded-2xl rounded-tr-sm px-3 py-2",
                  t.typingBg,
                  !reduceMotion && "wa-bubble-pop",
                )}
              >
                <TypingDots dotClass={t.typingDot} />
              </motion.div>
            )}
            {showAgent && (
              <motion.div
                initial={reduceMotion ? false : { opacity: 0, x: 14, scale: 0.98 }}
                animate={{ opacity: 1, x: 0, scale: 1 }}
                transition={{ type: "spring", stiffness: 400, damping: 26 }}
                className={cn(
                  "rounded-2xl rounded-tr-sm px-3 py-2.5 text-sm leading-snug",
                  t.outgoing,
                  compact && "text-[13px]",
                  t.outgoingText,
                )}
              >
                <p>{copy.agent}</p>
                <div className="mt-1 flex items-center justify-end gap-0.5 pr-0.5">
                  <span className={cn("text-[10px]", t.metaMuted)}>
                    {scenario === "dispute" ? "11:08" : "10:42"}
                  </span>
                  <span className="flex translate-y-px">
                    <Check
                      className={cn(
                        "h-3.5 w-3.5 -mr-1.5 text-[#53BDEB]",
                        showTicks && !reduceMotion && "wa-tick-pop",
                      )}
                      strokeWidth={3}
                    />
                    <Check
                      className={cn("h-3.5 w-3.5 text-[#53BDEB]", showTicks && !reduceMotion && "wa-tick-pop-delay")}
                      strokeWidth={3}
                    />
                  </span>
                </div>
              </motion.div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
