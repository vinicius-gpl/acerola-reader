use crate::protocol::files::SyncDirection;

/// Direção explícita de um `P2PNode::sync_comic`, exposta pra fronteira FFI (Kotlin) — mesmo
/// padrão de `FfiNetworkMode`/`NetworkMode` em `lib/mode.rs`. `SyncDirection` (o tipo interno do
/// protocolo, em `protocol::files::model`) não é exportável via UniFFI diretamente porque vive
/// num módulo `pub(crate)`, então esse enum espelhado é o que o Kotlin realmente vê.
#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiSyncDirection {
    Push,
    Pull,
}

impl From<FfiSyncDirection> for SyncDirection {
    fn from(value: FfiSyncDirection) -> Self {
        match value {
            FfiSyncDirection::Push => SyncDirection::Push,
            FfiSyncDirection::Pull => SyncDirection::Pull,
        }
    }
}
