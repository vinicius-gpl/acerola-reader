use sqlx::SqlitePool;

use crate::{
    data::{
        models::category::{category::Category, comic_category::ComicCategory},
        repositories::category::CategoryRepository,
    },
    infra::error::DbError,
};

/// Serviço responsável pelas regras de negócio de categorias (marcadores).
pub struct CategoryService {
    repo: CategoryRepository,
}

impl CategoryService {
    /// Inicializa o serviço de categorias.
    pub fn new(pool: SqlitePool) -> Self {
        Self { repo: CategoryRepository::new(pool) }
    }

    /// Cria uma nova categoria (marcador) com nome e cor.
    pub async fn create_category(&self, name: String, color: i64) -> Result<Category, DbError> {
        let category = Category { id: None, name, color };
        self.repo.base.insert(&category).await
    }

    /// Retorna todas as categorias (marcadores) cadastradas.
    pub async fn get_categories(&self) -> Result<Vec<Category>, DbError> {
        self.repo.base.find_all().await
    }

    /// Deleta uma categoria pelo seu ID.
    pub async fn delete_category(&self, id: i64) -> Result<(), DbError> {
        self.repo.base.delete(id).await
    }

    /// Atribui uma categoria a um quadrinho, garantindo que seja única por quadrinho.
    pub async fn assign_category_to_comic(
        &self, comic_id: i64, category_id: i64,
    ) -> Result<ComicCategory, DbError> {
        // Garante que qualquer associação anterior seja removida
        self.repo.remove_category_from_comic(comic_id).await?;

        let assignment = ComicCategory { id: None, comic_directory_fk: comic_id, category_id };
        self.repo.comic_category_base.insert(&assignment).await
    }

    /// Remove o marcador de um quadrinho.
    pub async fn remove_category_from_comic(&self, comic_id: i64) -> Result<(), DbError> {
        self.repo.remove_category_from_comic(comic_id).await
    }

    /// Busca a categoria associada a um quadrinho específico.
    pub async fn get_comic_category(&self, comic_id: i64) -> Result<Option<Category>, DbError> {
        self.repo.get_comic_category(comic_id).await
    }

    pub async fn get_all_comic_categories(&self) -> Result<Vec<ComicCategory>, DbError> {
        self.repo.comic_category_base.find_all().await
    }
}

#[cfg(test)]
mod tests {
    use super::CategoryService;
    use crate::tests::utils::setup_test_db::setup_test_db_with_comic;

    async fn setup() -> (sqlx::SqlitePool, CategoryService) {
        let pool = setup_test_db_with_comic().await;
        let service = CategoryService::new(pool.clone());
        (pool, service)
    }

    #[tokio::test]
    async fn test_creates_and_lists_categories() {
        let (_pool, service) = setup().await;

        let created = service.create_category("Favoritos".to_string(), 0xFF0000).await.unwrap();
        assert_eq!(created.name, "Favoritos");
        assert_eq!(created.color, 0xFF0000);

        let categories = service.get_categories().await.unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].name, "Favoritos");
    }

    #[tokio::test]
    async fn test_deletes_category() {
        let (_pool, service) = setup().await;

        let created = service.create_category("Lendo".to_string(), 0x00FF00).await.unwrap();
        service.delete_category(created.id.unwrap()).await.unwrap();

        assert!(service.get_categories().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_assign_category_to_comic_replaces_previous_assignment() {
        let (_pool, service) = setup().await;

        let first = service.create_category("Lendo".to_string(), 0x00FF00).await.unwrap();
        let second = service.create_category("Concluido".to_string(), 0x0000FF).await.unwrap();

        service.assign_category_to_comic(1, first.id.unwrap()).await.unwrap();
        service.assign_category_to_comic(1, second.id.unwrap()).await.unwrap();

        let assigned = service.get_comic_category(1).await.unwrap().unwrap();
        assert_eq!(assigned.id, second.id);

        let all = service.get_all_comic_categories().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_category_from_comic() {
        let (_pool, service) = setup().await;

        let category = service.create_category("Favoritos".to_string(), 0xFF0000).await.unwrap();
        service.assign_category_to_comic(1, category.id.unwrap()).await.unwrap();

        service.remove_category_from_comic(1).await.unwrap();

        assert!(service.get_comic_category(1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_comic_category_returns_none_when_unset() {
        let (_pool, service) = setup().await;

        assert!(service.get_comic_category(1).await.unwrap().is_none());
    }
}
