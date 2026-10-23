-- G-04.04: destruktivní rollback historie je záměrně zakázán.
-- Obnova schématu probíhá pouze na novém izolovaném cíli z pg_dump/pg_restore.
DO $$
BEGIN
    RAISE EXCEPTION 'G-04 down migrace je zakázána: historie se nesmí destruktivně mazat';
END
$$;
