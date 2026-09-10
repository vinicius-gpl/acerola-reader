use super::{ComicError, DbError, PathError};

impl From<DbError> for ComicError {
    fn from(db_err: DbError) -> Self {
        match db_err {
            DbError::UniqueViolation => {
                log::debug!("[ComicError] Duplicate comic blocked by UNIQUE constraint.");
                ComicError::AlreadyExists
            },
            DbError::NotFound => {
                log::warn!(
                    "[ComicError] Record not found — possible race condition or invalid id."
                );
                ComicError::NotFound
            },
            DbError::CheckViolation => {
                log::warn!("[ComicError] Invalid data rejected by CHECK constraint.");
                ComicError::InvalidRequest(DbError::CheckViolation.to_string())
            },
            DbError::ForeignKeyViolation => {
                log::error!(
                    "[ComicError] Referential integrity violation — parent record missing or removed."
                );
                ComicError::IntegrityViolation
            },
            DbError::Internal(ref err) => {
                log::error!("[ComicError] Unmapped internal database error: {:?}", err);
                ComicError::SystemFailure(db_err.to_string())
            },
        }
    }
}

impl From<sqlx::Error> for ComicError {
    fn from(err: sqlx::Error) -> Self {
        ComicError::from(DbError::from(err))
    }
}

impl From<std::io::Error> for ComicError {
    fn from(io_err: std::io::Error) -> Self {
        ComicError::Io(io_err)
    }
}

impl From<PathError> for ComicError {
    fn from(err: PathError) -> Self {
        match err {
            PathError::AccessDenied => {
                log::warn!("[ComicError] Access denied by PathGuard — path outside allowed root.");
                ComicError::InvalidRequest(err.to_string())
            },
            PathError::NotFound(_) => {
                log::warn!("[ComicError] Path not found: {}", err);
                ComicError::NotFound
            },
            PathError::ActionFailed(_) => {
                log::error!("[ComicError] Filesystem operation failed: {}", err);
                ComicError::SystemFailure(err.to_string())
            },
        }
    }
}

impl From<reqwest::Error> for ComicError {
    fn from(err: reqwest::Error) -> Self {
        ComicError::SystemFailure(format!("HTTP request failed: {}", err))
    }
}

impl From<quick_xml::DeError> for ComicError {
    fn from(err: quick_xml::DeError) -> Self {
        ComicError::InvalidRequest(format!("XML parse error: {}", err))
    }
}

impl From<String> for ComicError {
    fn from(err: String) -> Self {
        ComicError::SystemFailure(err)
    }
}
