use serde::Serialize;

use crate::infra::error::ComicError;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub error_type: String,
    pub message: String,
}

impl From<&ComicError> for ErrorPayload {
    fn from(err: &ComicError) -> Self {
        // Nome puro do variante (sem os dados de `Debug`), pra bater exato com a chave
        // usada em `COMIC_ERROR_MESSAGES` (errors.i18n.ts) — `format!("{:?}", err)` incluía
        // os dados internos (ex.: `InvalidRequest("msg")`), o que nunca batia com a chave.
        let error_type = match err {
            ComicError::AlreadyExists => "AlreadyExists",
            ComicError::SyncInProgress => "SyncInProgress",
            ComicError::NotFound => "NotFound",
            ComicError::ComicInfoNotFound => "ComicInfoNotFound",
            ComicError::InvalidRequest(_) => "InvalidRequest",
            ComicError::IntegrityViolation => "IntegrityViolation",
            ComicError::SystemFailure(_) => "SystemFailure",
            ComicError::Io(_) => "Io",
        };
        ErrorPayload { error_type: error_type.to_string(), message: err.to_string() }
    }
}
