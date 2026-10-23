-- G-04.04 / J09: pouze nová samostatná V2 databáze, spouští migrační role.
-- Bez automatického nasazení nebo vytváření rolí na sdíleném DB LXC.
CREATE SCHEMA IF NOT EXISTS airradar_v2;
REVOKE ALL ON SCHEMA airradar_v2 FROM PUBLIC;

CREATE TABLE IF NOT EXISTS airradar_v2.flight_samples (
    bucket_at timestamptz NOT NULL,
    aircraft_key varchar(14) NOT NULL
        CONSTRAINT flight_samples_aircraft_key_ck
        CHECK (aircraft_key ~ '^(icao|nonicao):[0-9a-f]{6}$'),
    position_at timestamptz NOT NULL,
    latitude double precision NOT NULL
        CONSTRAINT flight_samples_latitude_ck CHECK (latitude BETWEEN -90 AND 90),
    longitude double precision NOT NULL
        CONSTRAINT flight_samples_longitude_ck CHECK (longitude BETWEEN -180 AND 180),
    altitude_ft integer,
    ground_speed_kt real
        CONSTRAINT flight_samples_speed_ck CHECK (ground_speed_kt BETWEEN 0 AND 1500),
    track_deg real
        CONSTRAINT flight_samples_track_ck CHECK (track_deg >= 0 AND track_deg < 360),
    squawk varchar(4)
        CONSTRAINT flight_samples_squawk_ck CHECK (squawk ~ '^[0-7]{4}$'),
    on_ground boolean,
    CONSTRAINT flight_samples_bucket_ck CHECK (
        bucket_at = date_bin('30 seconds', position_at, '1970-01-01 00:00:00+00'::timestamptz)
    ),
    CONSTRAINT flight_samples_pk PRIMARY KEY (bucket_at, aircraft_key)
);

-- Runtime role nemá DDL. Její explicitní GRANT dělá až správce cílové DEV DB.
REVOKE ALL ON TABLE airradar_v2.flight_samples FROM PUBLIC;
