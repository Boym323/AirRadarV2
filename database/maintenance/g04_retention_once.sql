-- G-04.09 · J09. SPOUŠTĚT RUČNĚ AŽ PO SCHVÁLENÍ SPRÁVCEM DEV DB.
-- Bez globálních změn PG, bez automatického cron jobu nebo TRUNCATE.
BEGIN;
DO $$
BEGIN
    IF current_database() <> 'airradar_v2_dev' THEN
        RAISE EXCEPTION 'G-04 retence povolena pouze ve vyhrazené airradar_v2_dev';
    END IF;
    IF current_user <> 'airradar_v2_dev_maintenance' THEN
        RAISE EXCEPTION 'G-04 retence vyžaduje maintenance roli';
    END IF;
    IF to_regclass('airradar_v2.flight_samples') IS NULL THEN
        RAISE EXCEPTION 'G-04 schéma není vytvořené';
    END IF;
END
$$;
WITH old_rows AS (
    SELECT ctid FROM airradar_v2.flight_samples
    WHERE bucket_at < now() - interval '30 days'
    ORDER BY bucket_at
    LIMIT 1000
)
DELETE FROM airradar_v2.flight_samples
WHERE ctid IN (SELECT ctid FROM old_rows);
COMMIT;
-- Větší počet historických řádků se čistí po dávkách MIMO Rust hot path.
-- Bez naměřeného profilu retence se tento skript nespouští pravidelně.
