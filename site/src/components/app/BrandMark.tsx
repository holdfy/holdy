import { cn } from "@/lib/utils";

type BrandMarkSize = "sidebar" | "mobileHeader" | "nav" | "hero";

const presets: Record<BrandMarkSize, { logo: string }> = {
  /** Compact contexts */
  sidebar: {
    logo: "h-10 w-auto",
  },
  /** Mobile header */
  mobileHeader: {
    logo: "h-10 w-auto",
  },
  /** Landing header/footer */
  nav: {
    logo: "h-9 w-auto md:h-10",
  },
  /** Landing hero emphasis */
  hero: {
    logo: "h-16 w-auto md:h-20",
  },
};

export function BrandMark({
  size,
  className,
  textClassName,
}: {
  size: BrandMarkSize;
  className?: string;
  textClassName?: string;
}) {
  const p = presets[size];
  return (
    <div className={cn("inline-flex items-center", className)}>
      <img
        src="/logo-holdfy-transparent.png"
        alt="Holdfy"
        className={cn("select-none object-contain", p.logo, textClassName)}
        draggable={false}
      />
    </div>
  );
}
