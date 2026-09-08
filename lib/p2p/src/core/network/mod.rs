//! Módulo de orquestração de rede e gerenciamento de estado.

/// Gerenciador assíncrono do event loop de rede.
pub mod manager;
/// Reconexão proativa com peers conhecidos depois que o node é reconstruído (restart).
pub mod reconnect;
/// Tabela e gerenciador de estado nominal de conexões.
pub mod state;
