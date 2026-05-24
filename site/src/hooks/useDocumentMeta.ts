import { useEffect } from "react";
import { useTranslation } from "react-i18next";

export function useDocumentMeta() {
  const { t, i18n } = useTranslation();

  useEffect(() => {
    document.title = t("meta.title");
    const desc = document.querySelector('meta[name="description"]');
    if (desc) desc.setAttribute("content", t("meta.description"));
    const ogTitle = document.querySelector('meta[property="og:title"]');
    if (ogTitle) ogTitle.setAttribute("content", t("meta.ogTitle"));
    const ogDesc = document.querySelector('meta[property="og:description"]');
    if (ogDesc) ogDesc.setAttribute("content", t("meta.ogDescription"));
  }, [t, i18n.language]);
}
