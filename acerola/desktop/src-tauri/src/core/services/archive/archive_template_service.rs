use sqlx::SqlitePool;

use crate::{
    data::{
        models::archive::archive_template::{ArchiveTemplate, SortType},
        repositories::archive::archive_template_repo::ArchiveTemplateRepository,
    },
    infra::{
        error::ComicError,
        pattern::template_validator::{validate_chapter_template, validate_volume_template},
    },
};

/// Serviço responsável pelo CRUD de templates de nomenclatura definidos pelo usuário.
/// Toda a validação e o parsing de padrões continuam na camada `infra::pattern` — este
/// serviço apenas orquestra persistência e as regras de negócio em torno dela (unicidade
/// de label e proteção dos templates padrão contra remoção).
pub struct ArchiveTemplateService {
    repo: ArchiveTemplateRepository,
}

impl ArchiveTemplateService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { repo: ArchiveTemplateRepository::new(pool) }
    }

    /// Lista todos os templates (padrão do sistema + criados pelo usuário), com os
    /// padrão sempre no topo para deixar claro que são a referência inalterável.
    pub async fn list_templates(&self) -> Result<Vec<ArchiveTemplate>, ComicError> {
        let mut templates = self.repo.base.find_all().await?;
        templates
            .sort_by(|a, b| b.is_default.cmp(&a.is_default).then_with(|| a.label.cmp(&b.label)));
        Ok(templates)
    }

    /// Cria um novo template do usuário, validando o padrão com as mesmas regras usadas
    /// pelo scanner (`validate_chapter_template`/`validate_volume_template`) antes de
    /// persistir, para nunca deixar um padrão quebrado entrar no banco.
    pub async fn create_template(
        &self, label: String, pattern: String, sort_type: SortType,
    ) -> Result<ArchiveTemplate, ComicError> {
        let label = label.trim().to_string();
        let pattern = pattern.trim().to_string();

        if label.is_empty() {
            return Err(ComicError::InvalidRequest("Template name cannot be empty.".into()));
        }

        match sort_type {
            SortType::Chapter => validate_chapter_template(&pattern),
            SortType::Volume => validate_volume_template(&pattern),
        }
        .map_err(|err| ComicError::InvalidRequest(err.to_string()))?;

        let id = self.repo.base.get_next_id().await?;
        let template =
            ArchiveTemplate { id, label, pattern, sort_type, is_default: false, priority: 0 };

        Ok(self.repo.base.insert(&template).await?)
    }

    /// Remove um template criado pelo usuário. Templates padrão (`is_default`) nunca podem
    /// ser removidos por aqui — é essa garantia que mantém o parser sempre com um fallback
    /// conhecido, independentemente do que o usuário crie ou apague.
    pub async fn delete_template(&self, id: i64) -> Result<(), ComicError> {
        let templates = self.repo.base.find_all().await?;
        let target =
            templates.iter().find(|template| template.id == id).ok_or(ComicError::NotFound)?;

        if target.is_default {
            return Err(ComicError::InvalidRequest("Default templates cannot be deleted.".into()));
        }

        Ok(self.repo.base.delete(id).await?)
    }
}
