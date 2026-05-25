import { useQuery } from "@tanstack/react-query";
import { Shield, ShieldCheck, ShieldAlert } from "lucide-react";
import { api, type ReputationSeal } from "@/lib/api-client";
import { useTranslation } from "react-i18next";

interface Props {
  userId: string;
  completed?: number;
  disputeCount?: number;
  kycApproved?: boolean;
  className?: string;
}

const SEAL_ICON = {
  verified: ShieldCheck,
  premium: ShieldCheck,
  authenticated: ShieldAlert,
} satisfies Record<ReputationSeal["name"], React.ElementType>;

const SEAL_COLOR: Record<ReputationSeal["badge_color"], string> = {
  blue: "text-blue-500 bg-blue-50 dark:bg-blue-950/40 border-blue-200 dark:border-blue-800",
  gold: "text-amber-500 bg-amber-50 dark:bg-amber-950/40 border-amber-200 dark:border-amber-800",
  green: "text-emerald-500 bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-800",
};

export function ReputationBadge({ userId, completed = 0, disputeCount = 0, kycApproved = false, className }: Props) {
  const { t } = useTranslation();
  const { data, isLoading } = useQuery({
    queryKey: ["reputation", userId, completed, disputeCount, kycApproved],
    queryFn: () => api.getReputation(userId, { completed, dispute_count: disputeCount, kyc_approved: kycApproved }),
    staleTime: 5 * 60 * 1000,
  });

  if (isLoading) {
    return <div className={`h-6 w-24 animate-pulse rounded-full bg-muted ${className ?? ""}`} />;
  }

  if (!data) return null;

  const { score, seal, completed_transactions } = data;

  if (!seal) {
    return (
      <div className={`inline-flex items-center gap-1.5 text-xs text-muted-foreground ${className ?? ""}`}>
        <Shield className="h-3.5 w-3.5" />
        <span>{t("reputation.noSeal", "Score {{score}}", { score })}</span>
      </div>
    );
  }

  const Icon = SEAL_ICON[seal.name];
  const colorClass = SEAL_COLOR[seal.badge_color];

  return (
    <div
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-xs font-medium ${colorClass} ${className ?? ""}`}
      title={t("reputation.sealTitle", "{{label}} · Score {{score}} · {{txns}} transações", {
        label: seal.label,
        score,
        txns: completed_transactions,
      })}
    >
      <Icon className="h-3.5 w-3.5" />
      <span>{seal.label}</span>
    </div>
  );
}
