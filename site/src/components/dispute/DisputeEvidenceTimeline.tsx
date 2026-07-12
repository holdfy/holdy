import { AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { DisputeEvidenceItem } from "@/lib/api-client";

/** Timeline somente-leitura de evidências de uma disputa — mostra ambos os lados. */
export function DisputeEvidenceTimeline({
  evidence,
  currentParty,
}: {
  evidence: DisputeEvidenceItem[];
  currentParty: "buyer" | "seller";
}) {
  const { t } = useTranslation();

  const kindLabels: Record<string, string> = {
    photo: t("order.evidenceKindPhoto", "Foto"),
    video: t("order.evidenceKindVideo", "Vídeo"),
    tracking_code: t("order.evidenceKindTracking", "Código de rastreio"),
    message: t("order.evidenceKindMessage", "Mensagem"),
    other: t("order.evidenceKindOther", "Outro"),
  };

  if (!evidence.length) {
    return (
      <p className="text-xs text-muted-foreground py-1">
        {t("order.evidenceEmpty", "Nenhuma evidência enviada ainda.")}
      </p>
    );
  }

  return (
    <div className="space-y-2">
      {evidence.map((ev) => {
        const isMine = ev.party === currentParty;
        const partyLabel = isMine
          ? t("order.evidenceYou", "Você")
          : ev.party === "buyer"
            ? t("order.evidenceBuyer", "Comprador")
            : t("order.evidenceSeller", "Vendedor");
        const kindLabel = kindLabels[ev.kind] ?? ev.kind;
        const isMedia = ev.kind === "photo" || ev.kind === "video";

        return (
          <div
            key={ev.id}
            className={`rounded-xl border p-3 text-xs ${
              ev.ai_flagged
                ? "border-destructive/40 bg-destructive/5"
                : "border-border bg-muted/20"
            }`}
          >
            <div className="flex items-center justify-between gap-2 mb-1.5">
              <span
                className={`font-semibold px-2 py-0.5 rounded-full ${
                  isMine ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground"
                }`}
              >
                {partyLabel}
              </span>
              <div className="flex items-center gap-1.5 shrink-0">
                {ev.ai_flagged && (
                  <span className="flex items-center gap-1 text-destructive font-semibold">
                    <AlertTriangle className="h-3 w-3" />
                    {t("order.evidenceFlagged", "Suspeita")}
                  </span>
                )}
                <span className="text-muted-foreground">{kindLabel}</span>
              </div>
            </div>

            {isMedia && ev.minio_url ? (
              ev.kind === "video" ? (
                <video src={ev.minio_url} controls className="max-h-48 w-full rounded-lg" />
              ) : (
                <a href={ev.minio_url} target="_blank" rel="noopener noreferrer">
                  <img
                    src={ev.minio_url}
                    alt={kindLabel}
                    className="max-h-48 rounded-lg object-contain"
                    onError={(e) => {
                      (e.currentTarget as HTMLImageElement).style.display = "none";
                    }}
                  />
                </a>
              )
            ) : isMedia && !ev.minio_url ? (
              <p className="text-muted-foreground italic">
                {t("order.evidenceUnavailable", "Arquivo indisponível")}
              </p>
            ) : (
              ev.content && <p className="text-foreground break-words">{ev.content}</p>
            )}

            <p className="text-[10px] text-muted-foreground mt-1.5">
              {new Date(ev.created_at).toLocaleString("pt-BR")}
            </p>
          </div>
        );
      })}
    </div>
  );
}
