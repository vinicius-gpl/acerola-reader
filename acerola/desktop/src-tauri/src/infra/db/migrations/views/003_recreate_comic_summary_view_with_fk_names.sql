CREATE VIEW IF NOT EXISTS comic_summary_view AS
SELECT
  md.id AS directory_fk,
  md.name AS folder_name,
  md.cover AS folder_cover,
  md.banner AS folder_banner,
  md.external_sync_enabled AS external_sync,
  mm.title AS metadata_title,
  mm.sync_source AS active_source,
  mm.id AS metadata_fk
FROM comic_directory md
LEFT JOIN comic_metadata mm ON md.id = mm.comic_directory_fk;
