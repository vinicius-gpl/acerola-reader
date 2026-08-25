use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    let mut migrations = vec![];
    migrations.extend(archive_migrations());
    migrations.extend(metadata_migrations());
    migrations.extend(category_migrations());
    migrations.extend(history_migrations());
    migrations.extend(view_migrations());
    migrations.extend(seed_migrations());
    migrations.extend(sync_migrations());
    migrations
}

fn archive_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_archive_template",
            sql: include_str!("./migrations/models/archive/001_create_chapter_template.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_comic_directory",
            sql: include_str!("./migrations/models/archive/002_create_comic_directory.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_chapter_archive",
            sql: include_str!("./migrations/models/archive/003_create_chapter_archive.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_volume_archive",
            sql: include_str!("./migrations/models/archive/004_create_volume_archive.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 19,
            description: "rename_volume_id_fk_to_volume_fk",
            sql: include_str!("./migrations/models/archive/005_rename_volume_id_fk_to_volume_fk.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

fn metadata_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 5,
            description: "create_comic_metadata",
            sql: include_str!("./migrations/models/metadata/001_create_comic_metadata.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "create_chapter_metadata",
            sql: include_str!("./migrations/models/metadata/002_create_chapter_metadata.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "create_anilist_source",
            sql: include_str!("./migrations/models/metadata/source/001_create_anilist_source.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 8,
            description: "create_mangadex_source",
            sql: include_str!("./migrations/models/metadata/source/003_create_mangadex_source.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9,
            description: "create_genre",
            sql: include_str!("./migrations/models/metadata/relationship/001_create_genre.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 10,
            description: "create_author",
            sql: include_str!("./migrations/models/metadata/relationship/004_create_author.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 17,
            description: "drop_chapter_metadata",
            sql: "DROP TABLE IF EXISTS chapter_metadata;",
            kind: MigrationKind::Up,
        },
    ]
}

fn category_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 11,
            description: "create_category",
            sql: include_str!("./migrations/models/category/001_create_category.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 12,
            description: "create_comic_category",
            sql: include_str!("./migrations/models/category/002_create_comic_category.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

fn history_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 13,
            description: "create_reading_history",
            sql: include_str!("./migrations/models/history/001_create_reading_history.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 14,
            description: "create_chapter_read",
            sql: include_str!("./migrations/models/history/002_create_chapter_read.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

fn view_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 15,
            description: "create_comic_summary_view",
            sql: include_str!("./migrations/views/001_create_comic_summary_view.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 20,
            description: "drop_comic_summary_view",
            sql: include_str!("./migrations/views/002_drop_comic_summary_view.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 21,
            description: "recreate_comic_summary_view_with_fk_names",
            sql: include_str!("./migrations/views/003_recreate_comic_summary_view_with_fk_names.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

fn seed_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 16,
        description: "seed_archive_template",
        sql: include_str!("./migrations/seeds/001_seed_chapter_template.sql"),
        kind: MigrationKind::Up,
    }]
}

fn sync_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 18,
        description: "create_sync_history_log",
        sql: include_str!("./migrations/models/sync/001_create_sync_history_log.sql"),
        kind: MigrationKind::Up,
    }]
}
