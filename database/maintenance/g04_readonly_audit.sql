-- G-04.01 / J09 · pouze read-only audit; nespouštět v transakci s DDL.
-- Výstup předá správce DB spolu s externím hostitelským auditem.
SELECT current_database() AS database_name,
       current_user AS role_name,
       current_setting('server_version') AS server_version,
       pg_size_pretty(pg_database_size(current_database())) AS database_size;

SELECT name, setting, unit
FROM pg_settings
WHERE name IN ('max_connections', 'shared_buffers', 'wal_level',
               'archive_mode', 'archive_command', 'fsync',
               'full_page_writes', 'synchronous_commit')
ORDER BY name;

SELECT wal_records, wal_fpi, wal_bytes, stats_reset
FROM pg_stat_wal;

-- Může vyžadovat oprávnění dle verze PostgreSQL. Při odmítnutí je výsledek
-- NEOVĚŘENO, nikoli PASS; serverová konfigurace se tímto skriptem nemění.
SELECT name, size
FROM pg_ls_waldir()
ORDER BY name DESC
LIMIT 1;
