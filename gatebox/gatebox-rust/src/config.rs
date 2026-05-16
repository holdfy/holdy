// Converted from gateboxgo/config/init.go
use std::path::Path;

/// Carrega variáveis do arquivo .env.
/// Busca em: diretório atual → diretório do executável → diretório pai do executável.
pub fn init() {
    let loaded = dotenvy::dotenv().is_ok()
        || try_load_from_exe_dir()
        || try_load_from_exe_parent_dir();

    if loaded {
        tracing::info!("Variáveis carregadas do .env");
    } else {
        tracing::warn!("Arquivo .env não encontrado - usando variáveis de ambiente");
    }
}

fn try_load_from_exe_dir() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .and_then(|dir| dotenvy::from_path(dir.join(".env")).ok())
        .is_some()
}

fn try_load_from_exe_parent_dir() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().and_then(|p| p.parent()).map(Path::to_path_buf))
        .and_then(|dir| dotenvy::from_path(dir.join(".env")).ok())
        .is_some()
}
