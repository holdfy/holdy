import type { MouseEvent } from "react";
import { ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";

type Props = {
  /** Element id to scroll into view (smooth). */
  targetId: string;
};

export function LandingScrollCue({ targetId }: Props) {
  const { t } = useTranslation();

  function onActivate(e: MouseEvent<HTMLAnchorElement>) {
    e.preventDefault();
    document.getElementById(targetId)?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  return (
    <a
      href={`#${targetId}`}
      onClick={onActivate}
      className="group flex flex-col items-center gap-2 rounded-2xl py-0.5 text-[#00E5A0]/80 transition hover:text-[#00E5A0] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-[#00E5A0]"
      aria-label={t("common.scrollNext")}
    >
      <span className="text-[10px] font-semibold uppercase tracking-[0.35em] text-white/35 group-hover:text-white/55">
        {t("common.scroll")}
      </span>
      <span className="flex h-11 w-11 items-center justify-center rounded-full border border-[#00E5A0]/25 bg-[#00E5A0]/5 shadow-[0_0_24px_rgba(0,229,160,0.12)] motion-reduce:animate-none motion-safe:animate-[heroScrollCue_2.2s_ease-in-out_infinite]">
        <ChevronDown className="h-5 w-5" strokeWidth={2.5} aria-hidden />
      </span>
    </a>
  );
}
