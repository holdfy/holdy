//! Internacionalização (pt-BR, en, es) para o dashboard HoldFy.

mod messages;

use leptos::prelude::*;

pub use messages::MsgKey;

/// Idiomas suportados.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    PtBr,
    En,
    Es,
}

impl Locale {
    pub const ALL: [Locale; 3] = [Locale::PtBr, Locale::En, Locale::Es];

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "pt-BR" | "pt" => Some(Self::PtBr),
            "en" | "en-US" => Some(Self::En),
            "es" | "es-ES" => Some(Self::Es),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::En => "en",
            Self::Es => "es",
        }
    }

    pub fn label_key(self) -> MsgKey {
        match self {
            Self::PtBr => MsgKey::LangPt,
            Self::En => MsgKey::LangEn,
            Self::Es => MsgKey::LangEs,
        }
    }
}

/// Contexto reativo de idioma (fornecido por [`I18nProvider`]).
#[derive(Clone, Copy)]
pub struct I18nContext {
    pub locale: RwSignal<Locale>,
}

pub fn use_i18n() -> I18nContext {
    expect_context::<I18nContext>()
}

/// Traduz uma chave para o idioma indicado.
pub fn t(locale: Locale, key: MsgKey) -> &'static str {
    messages::lookup(locale, key)
}

/// Componente de texto traduzido (reativo ao idioma atual).
#[component]
pub fn T(key: MsgKey) -> impl IntoView {
    let i18n = use_i18n();
    move || t(i18n.locale.get(), key)
}
