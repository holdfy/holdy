import { CheckCircle2, Package, Truck, MapPin, Clock, AlertTriangle, RotateCcw } from "lucide-react";
import { type TrackingInfo } from "@/lib/api-client";

interface Props {
  info: TrackingInfo;
}

// ─── Status config ────────────────────────────────────────────────────────────

type StatusKey = TrackingInfo["current_status"];

const STATUS_CONFIG: Record<string, {
  label: string;
  badgeClass: string;
  icon: React.ElementType;
}> = {
  delivered: {
    label: "Entregue",
    badgeClass: "bg-emerald-100 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300",
    icon: CheckCircle2,
  },
  in_transit: {
    label: "Em trânsito",
    badgeClass: "bg-blue-100 text-blue-700 dark:bg-blue-950/60 dark:text-blue-300",
    icon: Truck,
  },
  out_for_delivery: {
    label: "Saiu para entrega",
    badgeClass: "bg-orange-100 text-orange-700 dark:bg-orange-950/60 dark:text-orange-300",
    icon: Package,
  },
  posted: {
    label: "Postado",
    badgeClass: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300",
    icon: Package,
  },
  exception: {
    label: "Problema na entrega",
    badgeClass: "bg-red-100 text-red-700 dark:bg-red-950/60 dark:text-red-300",
    icon: AlertTriangle,
  },
  return_in_progress: {
    label: "Retorno em andamento",
    badgeClass: "bg-amber-100 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300",
    icon: RotateCcw,
  },
  returned: {
    label: "Devolvido",
    badgeClass: "bg-amber-100 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300",
    icon: RotateCcw,
  },
};

const FALLBACK_STATUS = {
  label: "Status desconhecido",
  badgeClass: "bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400",
  icon: Package,
};

function getStatusConfig(status: StatusKey) {
  return STATUS_CONFIG[status] ?? FALLBACK_STATUS;
}

// ─── Date formatting ──────────────────────────────────────────────────────────

function formatEventDate(iso: string): string {
  try {
    return new Intl.DateTimeFormat("pt-BR", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

// ─── Component ────────────────────────────────────────────────────────────────

export function TrackingCard({ info }: Props) {
  const cfg = getStatusConfig(info.current_status);
  const StatusIcon = cfg.icon;

  return (
    <div className="rounded-2xl border border-border bg-card p-5 space-y-4">
      {/* Header */}
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase mb-1">
            Código de rastreio
          </p>
          <p className="font-mono font-bold text-base">{info.tracking_code}</p>
          {info.carrier && (
            <p className="text-xs text-muted-foreground mt-0.5">{info.carrier}</p>
          )}
        </div>
        <span
          className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold shrink-0 ${cfg.badgeClass}`}
        >
          <StatusIcon className="h-3.5 w-3.5" />
          {cfg.label}
        </span>
      </div>

      {/* Estimated delivery */}
      {info.estimated_delivery && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Clock className="h-4 w-4 shrink-0" />
          <span>
            Previsão:{" "}
            <span className="font-medium text-foreground">
              {formatEventDate(info.estimated_delivery)}
            </span>
          </span>
        </div>
      )}

      {/* Events timeline */}
      {info.events.length > 0 && (
        <div>
          <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase mb-3">
            Histórico
          </p>
          <ol className="relative border-l border-border/60 space-y-4 pl-4">
            {info.events.map((ev, idx) => (
              <li key={idx} className="relative">
                {/* Dot */}
                <span
                  className={`absolute -left-[1.375rem] top-0.5 h-3 w-3 rounded-full border-2 border-background ${
                    idx === 0 ? "bg-primary" : "bg-muted-foreground/30"
                  }`}
                />
                <p
                  className={`text-sm font-medium leading-snug ${
                    idx === 0 ? "text-foreground" : "text-muted-foreground"
                  }`}
                >
                  {ev.description}
                </p>
                <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5 mt-0.5">
                  <span className="text-[11px] text-muted-foreground">
                    {formatEventDate(ev.occurred_at)}
                  </span>
                  {ev.location && (
                    <span className="inline-flex items-center gap-0.5 text-[11px] text-muted-foreground">
                      <MapPin className="h-3 w-3" />
                      {ev.location}
                    </span>
                  )}
                </div>
              </li>
            ))}
          </ol>
        </div>
      )}

      {info.events.length === 0 && (
        <p className="text-sm text-muted-foreground text-center py-2">
          Nenhum evento registrado ainda.
        </p>
      )}

      {/* Provider tag */}
      {info.provider_used && (
        <p className="text-[11px] text-muted-foreground text-right">
          via {info.provider_used}
        </p>
      )}
    </div>
  );
}
