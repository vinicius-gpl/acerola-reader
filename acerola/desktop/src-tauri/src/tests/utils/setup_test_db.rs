pub async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // archive
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/001_create_chapter_template.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/002_create_comic_directory.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/003_create_chapter_archive.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/004_create_volume_archive.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/005_rename_volume_id_fk_to_volume_fk.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // metadata
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/001_create_comic_metadata.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/002_create_chapter_metadata.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/source/001_create_anilist_source.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/source/003_create_mangadex_source.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/relationship/001_create_genre.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/relationship/004_create_author.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // category
    sqlx::query(include_str!("../../infra/db/migrations/models/category/001_create_category.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/category/002_create_comic_category.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // history
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/history/001_create_reading_history.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/history/002_create_chapter_read.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // views
    sqlx::query(include_str!("../../infra/db/migrations/views/001_create_comic_summary_view.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!("../../infra/db/migrations/views/002_drop_comic_summary_view.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/views/003_recreate_comic_summary_view_with_fk_names.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // sync
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/sync/001_create_sync_history_log.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    pool
}

/// Pool com seeds de produção aplicados — usado quando o teste depende dos templates padrão.
pub async fn setup_test_db_with_seeds() -> sqlx::SqlitePool {
    let pool = setup_test_db().await;

    sqlx::query(include_str!("../../infra/db/migrations/seeds/001_seed_chapter_template.sql"))
        .execute(&pool)
        .await
        .unwrap();

    pool
}

/// Zera o `last_modified` de todos os comics — usado para forçar reprocessamento no incremental.
pub async fn reset_comics_last_modified(pool: &sqlx::SqlitePool) {
    sqlx::query("UPDATE comic_directory SET last_modified = 0").execute(pool).await.unwrap();
}

/// Pool com um comic_directory já inserido — usado por testes de chapter que precisam da FK.
pub async fn setup_test_db_with_comic() -> sqlx::SqlitePool {
    let pool = setup_test_db().await;

    sqlx::query(
        "INSERT INTO comic_directory (id, name, path, last_modified, external_sync_enabled, hidden)
         VALUES (1, 'Test', '/test', 0, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

/// Insere um comic_directory mínimo (sem capa nem metadados).
pub async fn insert_comic_directory(pool: &sqlx::SqlitePool, id: i64, name: &str, path: &str) {
    sqlx::query(
        "INSERT INTO comic_directory (id, name, path, last_modified, external_sync_enabled, hidden)
         VALUES (?, ?, ?, 0, 0, 0)",
    )
    .bind(id)
    .bind(name)
    .bind(path)
    .execute(pool)
    .await
    .unwrap();
}

/// Insere um comic_directory com capa e versão de capa — usado pelos testes de browse de capa.
pub async fn insert_comic_directory_with_cover(
    pool: &sqlx::SqlitePool, id: i64, name: &str, path: &str, cover_path: &str, cover_version: i64,
) {
    sqlx::query(
        "INSERT INTO comic_directory (id, name, path, cover, last_modified, external_sync_enabled, hidden)
         VALUES (?, ?, ?, ?, ?, 0, 0)",
    )
    .bind(id)
    .bind(name)
    .bind(path)
    .bind(cover_path)
    .bind(cover_version)
    .execute(pool)
    .await
    .unwrap();
}

/// Insere metadados mínimos de um quadrinho, vinculados por `comic_directory_fk`.
pub async fn insert_comic_metadata(pool: &sqlx::SqlitePool, id: i64, comic_directory_fk: i64, title: &str) {
    sqlx::query(
        "INSERT INTO comic_metadata (id, comic_directory_fk, title, description, status) VALUES (?, ?, ?, '', '')",
    )
    .bind(id)
    .bind(comic_directory_fk)
    .bind(title)
    .execute(pool)
    .await
    .unwrap();
}

/// Insere um capítulo mínimo vinculado a um comic_directory.
pub async fn insert_chapter_archive(pool: &sqlx::SqlitePool, id: i64, comic_directory_fk: i64) {
    sqlx::query(
        "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified)
         VALUES (?, ?, 'path', '1', 0, ?, 0)",
    )
    .bind(id)
    .bind(format!("{id}"))
    .bind(comic_directory_fk)
    .execute(pool)
    .await
    .unwrap();
}

/// Insere um capítulo com checksum — usado por testes de sync de arquivos que comparam hash.
pub async fn insert_chapter_archive_with_checksum(
    pool: &sqlx::SqlitePool, id: i64, comic_directory_fk: i64, chapter: &str, path: &str, checksum: &str,
) {
    sqlx::query(
        "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified)
         VALUES (?, ?, ?, '1', 0, ?, ?, 0)",
    )
    .bind(id)
    .bind(chapter)
    .bind(path)
    .bind(checksum)
    .bind(comic_directory_fk)
    .execute(pool)
    .await
    .unwrap();
}

/// Insere um quadrinho com N capítulos e metadados — usado pelos testes de ordenação/filtro da Home.
pub async fn insert_comic_with_chapters(
    pool: &sqlx::SqlitePool, id: i64, name: &str, title: &str, chapter_count: i64,
) {
    insert_comic_directory(pool, id, name, &format!("/mangas/{}", name.to_lowercase())).await;
    insert_comic_metadata(pool, id, id, title).await;

    for i in 0..chapter_count {
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified)
             VALUES (?, ?, ?, ?, 0, ?, 0)",
        )
        .bind(format!("{id}{i}"))
        .bind(format!("Capítulo {i}"))
        .bind(format!("/mangas/{}/cap{}", name.to_lowercase(), i))
        .bind(format!("{i:03}"))
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// Pool com um comic_directory e volumes já inseridos.
pub async fn setup_test_db_with_volumes() -> sqlx::SqlitePool {
    let pool = setup_test_db_with_comic().await;

    sqlx::query(
        "INSERT INTO volume_archive (id, name, path, volume_sort, is_special, comic_directory_fk, last_modified)
         VALUES (1, 'Vol 01', '/test/v1', '1', 0, 1, 0),
                (2, 'Vol 02', '/test/v2', '2', 0, 1, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}
