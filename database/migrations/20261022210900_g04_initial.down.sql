-- G-04.04: DESTRUKTIVNÍ návrat, pouze v izolované testovací DEV DB.
-- Před použitím na databázi s historií je povinná záloha a schválení.
DROP TABLE IF EXISTS airradar_v2.flight_samples;
-- Schéma záměrně nemažeme: může později obsahovat jiné objekty.
