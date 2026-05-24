import { useEffect, useMemo, useRef } from "react";
import { cn } from "@/lib/utils";
import { Link } from "react-router-dom";
import { ArrowRight } from "lucide-react";
import { type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";

interface VerticalMarqueeProps {
  children: ReactNode;
  pauseOnHover?: boolean;
  reverse?: boolean;
  className?: string;
  speed?: number;
}

function VerticalMarquee({
  children,
  pauseOnHover = false,
  reverse = false,
  className,
  speed = 30,
}: VerticalMarqueeProps) {
  return (
    <div
      className={cn("group flex flex-col overflow-hidden", className)}
      style={
        {
          "--duration": `${speed}s`,
        } as React.CSSProperties
      }
    >
      <div
        className={cn(
          "flex shrink-0 flex-col animate-marquee-vertical motion-reduce:animate-none",
          reverse && "[animation-direction:reverse]",
          pauseOnHover && "group-hover:[animation-play-state:paused]",
        )}
      >
        {children}
      </div>
      <div
        className={cn(
          "flex shrink-0 flex-col animate-marquee-vertical motion-reduce:animate-none",
          reverse && "[animation-direction:reverse]",
          pauseOnHover && "group-hover:[animation-play-state:paused]",
        )}
        aria-hidden="true"
      >
        {children}
      </div>
    </div>
  );
}

export function CTAWithVerticalMarquee({
  className,
  eyebrow,
  title,
  description,
  footnote,
  marqueeItems,
}: {
  className?: string;
  eyebrow?: string;
  title?: string;
  description?: string;
  footnote?: string;
  marqueeItems?: string[];
}) {
  const { t } = useTranslation();
  const marqueeRef = useRef<HTMLDivElement>(null);

  const resolvedEyebrow = eyebrow ?? t("landing.cta.eyebrow");
  const resolvedTitle = title ?? t("landing.cta.title");
  const resolvedDescription = description ?? t("landing.cta.description");
  const resolvedFootnote = footnote ?? t("landing.cta.footnote");
  const resolvedMarqueeItems = useMemo(
    () =>
      marqueeItems ?? [
        t("landing.cta.marquee1"),
        t("landing.cta.marquee2"),
        t("landing.cta.marquee3"),
        t("landing.cta.marquee4"),
        t("landing.cta.marquee5"),
      ],
    [marqueeItems, t],
  );

  useEffect(() => {
    const marqueeContainer = marqueeRef.current;
    if (!marqueeContainer) return;

    const prefersReduced =
      typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (prefersReduced) return;

    let rafId = 0;

    const updateOpacity = () => {
      const items = marqueeContainer.querySelectorAll(".marquee-item");
      const containerRect = marqueeContainer.getBoundingClientRect();
      const centerY = containerRect.top + containerRect.height / 2;

      items.forEach((item) => {
        const itemRect = item.getBoundingClientRect();
        const itemCenterY = itemRect.top + itemRect.height / 2;
        const distance = Math.abs(centerY - itemCenterY);
        const maxDistance = containerRect.height / 2;
        const normalizedDistance = Math.min(distance / maxDistance, 1);
        const opacity = 1 - normalizedDistance * 0.75;
        (item as HTMLElement).style.opacity = opacity.toString();
      });
    };

    const loop = () => {
      updateOpacity();
      rafId = requestAnimationFrame(loop);
    };

    rafId = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafId);
  }, []);

  return (
    <div className={cn("overflow-hidden text-white", className)}>
      <div className="w-full max-w-7xl animate-fade-in-up motion-reduce:animate-none">
        <div className="grid grid-cols-1 items-center gap-12 lg:grid-cols-2 lg:gap-16">
          <div className="max-w-xl space-y-8">
            <span className="text-xs font-bold uppercase tracking-widest text-[#00E5A0] animate-fade-in-up motion-reduce:animate-none [animation-delay:100ms]">
              {resolvedEyebrow}
            </span>
            <h2 className="font-display text-4xl font-bold leading-tight tracking-tight text-white animate-fade-in-up motion-reduce:animate-none [animation-delay:200ms] md:text-5xl lg:text-6xl">
              {resolvedTitle}
            </h2>
            <p className="text-lg leading-relaxed text-white/50 animate-fade-in-up motion-reduce:animate-none [animation-delay:300ms] md:text-xl">
              {resolvedDescription}
            </p>
            <div className="flex flex-wrap gap-4 animate-fade-in-up motion-reduce:animate-none [animation-delay:400ms]">
              <Button
                size="lg"
                className="h-12 rounded-full border-0 bg-[#00E5A0] px-8 font-semibold text-[#0A0D0F] hover:bg-[#00c98a]"
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
                className="h-12 rounded-full border-white/15 bg-transparent px-8 font-semibold text-white hover:bg-white/10 hover:text-white"
                asChild
              >
                <Link to="/login">{t("common.signIn")}</Link>
              </Button>
            </div>
            <p className="text-sm text-white/45 animate-fade-in-up motion-reduce:animate-none [animation-delay:500ms]">
              {resolvedFootnote}
            </p>
          </div>

          <div
            ref={marqueeRef}
            className="relative flex h-[min(420px,55vh)] items-center justify-center animate-fade-in-up motion-reduce:animate-none [animation-delay:300ms] md:h-[min(480px,50vh)] lg:h-[560px]"
          >
            <div className="relative h-full w-full">
              <VerticalMarquee speed={22} className="h-full">
                {resolvedMarqueeItems.map((item, idx) => (
                  <div
                    key={`${item}-${idx}`}
                    className="marquee-item py-6 font-display text-3xl font-light tracking-tight text-white/90 md:text-4xl lg:text-5xl xl:text-6xl"
                  >
                    {item}
                  </div>
                ))}
              </VerticalMarquee>

              <div className="pointer-events-none absolute left-0 right-0 top-0 z-10 h-32 bg-gradient-to-b from-[#0A0D0F] via-[#0A0D0F]/60 to-transparent md:h-40" />
              <div className="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-32 bg-gradient-to-t from-[#0A0D0F] via-[#0A0D0F]/60 to-transparent md:h-40" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default CTAWithVerticalMarquee;
