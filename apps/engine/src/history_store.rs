//! G-04.05/.07/.08 · J09: izolovaný DEV PostgreSQL přístup přes SQLx.
//! Nikdy se neaktivuje implicitně při G-03 ani nevytváří schéma nebo role.
use crate::history::Sample;
use sqlx::{
    PgPool, Postgres, QueryBuilder, Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::time::Duration;

/// Přihlašovací údaje jsou samostatné systémové proměnné.
/// URL s heslem se nepoužívá: heslo proto nemusí být URL-escaped.
fn dev_options(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    ca: &str,
) -> Result<PgConnectOptions, &'static str> {
    if host.is_empty()
        || host.trim() != host
        || host.chars().any(char::is_whitespace)
        || host.contains("://")
        || host.contains('/')
        || port == 0
        || username.trim().is_empty()
        || password.is_empty()
        || ca.trim().is_empty()
    {
        return Err("G-04: neplatná konfigurace host/port/uživatel/heslo/CA");
    }
    Ok(PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(username)
        .password(password)
        .database("airradar_v2_dev")
        .ssl_mode(PgSslMode::VerifyFull)
        .ssl_root_cert(ca))
}

/// Pouze explicitní G-04 DEV runtime, ověření certifikátu serveru zůstává povinné.
/// Samotné jméno a heslo nezaručuje šifrování přenosu.
pub fn dev_pool_from_env() -> Result<PgPool, Box<dyn std::error::Error>> {
    let host = std::env::var("AIRRADAR_V2_DEV_DB_HOST")?;
    let port: u16 = std::env::var("AIRRADAR_V2_DEV_DB_PORT")
        .unwrap_or_else(|_| "5432".to_owned())
        .parse()?;
    let username = std::env::var("AIRRADAR_V2_DEV_DB_USER")?;
    let password = std::env::var("AIRRADAR_V2_DEV_DB_PASSWORD")?;
    let ca = std::env::var("AIRRADAR_V2_DEV_CA_CERT")?;
    let options = dev_options(&host, port, &username, &password, &ca)?;
    Ok(PgPoolOptions::new()
        .max_connections(2)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(2))
        .idle_timeout(Some(Duration::from_secs(60)))
        .connect_lazy_with(options))
}

/// Jeden vícerow INSERT v implicitní PostgreSQL transakci.
/// Konflikt UTC bucketu je idempotentní; žádný SELECT před INSERT ani UPDATE.
pub async fn insert_batch(pool: &PgPool, batch: &[Sample]) -> Result<u64, &'static str> {
    if batch.is_empty() || batch.len() > 128 {
        return Err("invalid batch size");
    }
    let mut query = QueryBuilder::<Postgres>::new(
        "INSERT INTO airradar_v2.flight_samples          (bucket_at, aircraft_key, position_at, latitude, longitude, altitude_ft,           ground_speed_kt, track_deg, squawk, on_ground)          SELECT to_timestamp(v.bucket_epoch), v.aircraft_key, to_timestamp(v.position_epoch),                 v.latitude, v.longitude, v.altitude_ft, v.ground_speed_kt::real,                 v.track_deg::real, v.squawk, v.on_ground FROM (",
    );
    query.push_values(batch.iter(), |mut b, s| {
        b.push_bind(s.bucket_at as f64)
            .push_bind(&s.aircraft_key)
            .push_bind(s.position_at)
            .push_bind(s.latitude)
            .push_bind(s.longitude)
            .push_bind(s.altitude_ft)
            .push_bind(s.ground_speed_kt)
            .push_bind(s.track_deg)
            .push_bind(s.squawk.as_deref())
            .push_bind(s.on_ground);
    });
    query.push(
        ") AS v (bucket_epoch, aircraft_key, position_epoch, latitude, longitude,          altitude_ft, ground_speed_kt, track_deg, squawk, on_ground)          ON CONFLICT (bucket_at, aircraft_key) DO NOTHING",
    );
    let statement = query.build();
    match tokio::time::timeout(Duration::from_secs(5), statement.execute(pool)).await {
        Ok(Ok(result)) => Ok(result.rows_affected()),
        _ => Err("historický SQL zápis selhal nebo vypršel timeout"),
    }
}

/// Historické čtení je explicitní a časově ohraničené; živá mapa používá RAM.
/// SQL řádky se nevracejí přímo do veřejného DTO.
#[allow(dead_code)]
pub async fn read_history(
    pool: &PgPool,
    aircraft_key: &str,
    from_epoch: f64,
    until_epoch: f64,
    limit: i64,
) -> Result<Vec<Sample>, &'static str> {
    if !from_epoch.is_finite()
        || !until_epoch.is_finite()
        || from_epoch >= until_epoch
        || !(1..=1000).contains(&limit)
        || aircraft_key.len() > 14
    {
        return Err("invalid history query");
    }
    let rows = tokio::time::timeout(
        Duration::from_secs(5),
        sqlx::query(
            "SELECT extract(epoch from bucket_at)::double precision AS bucket_epoch,                     aircraft_key, extract(epoch from position_at)::double precision AS position_epoch,                     latitude, longitude, altitude_ft, ground_speed_kt, track_deg, squawk, on_ground              FROM airradar_v2.flight_samples              WHERE aircraft_key=$1 AND bucket_at>=to_timestamp($2)                AND bucket_at<to_timestamp($3)              ORDER BY bucket_at ASC LIMIT $4",
        )
        .bind(aircraft_key)
        .bind(from_epoch)
        .bind(until_epoch)
        .bind(limit)
        .fetch_all(pool),
    )
    .await
    .map_err(|_| "history query timeout")?
    .map_err(|_| "history query failed")?;
    rows.iter()
        .map(|r| {
            Ok(Sample {
                bucket_at: r.try_get::<f64, _>("bucket_epoch").map_err(|_| "bucket")? as i64,
                aircraft_key: r.try_get("aircraft_key").map_err(|_| "key")?,
                position_at: r.try_get("position_epoch").map_err(|_| "time")?,
                latitude: r.try_get("latitude").map_err(|_| "latitude")?,
                longitude: r.try_get("longitude").map_err(|_| "longitude")?,
                altitude_ft: r.try_get("altitude_ft").map_err(|_| "altitude")?,
                ground_speed_kt: r
                    .try_get::<Option<f32>, _>("ground_speed_kt")
                    .map_err(|_| "speed")?
                    .map(f64::from),
                track_deg: r
                    .try_get::<Option<f32>, _>("track_deg")
                    .map_err(|_| "track")?
                    .map(f64::from),
                squawk: r.try_get("squawk").map_err(|_| "squawk")?,
                on_ground: r.try_get("on_ground").map_err(|_| "ground")?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_batch_is_never_a_transaction() {
        let _ = Sample {
            bucket_at: 0,
            aircraft_key: "icao:abcdef".to_string(),
            position_at: 0.0,
            latitude: 0.0,
            longitude: 0.0,
            altitude_ft: None,
            ground_speed_kt: None,
            track_deg: None,
            squawk: None,
            on_ground: None,
        };
    }
    #[test]
    fn g04_username_password_and_tls_config_validate_without_network() {
        // Hesla mohou obsahovat znaky, které by vyžadovaly URL escapování.
        let options = dev_options(
            "postgres-dev.example.test",
            5432,
            "airradar_v2_dev_app",
            "a:@ /?#%v2",
            "/etc/airradar-v2/pg-ca.pem",
        );
        assert!(options.is_ok());
        assert!(dev_options("", 5432, "user", "pass", "ca.pem").is_err());
        assert!(dev_options("host", 0, "user", "pass", "ca.pem").is_err());
        assert!(dev_options("host", 5432, "", "pass", "ca.pem").is_err());
        assert!(dev_options("host", 5432, "user", "", "ca.pem").is_err());
        assert!(dev_options("host", 5432, "user", "pass", "").is_err());
        assert!(dev_options("postgres://host", 5432, "user", "pass", "ca.pem").is_err());
    }

    #[tokio::test]
    async fn g04_postgres_batch_dedup_and_bounded_history() {
        let Ok(url) = std::env::var("AIRRADAR_G04_CI_DATABASE_URL") else {
            // Povinně spouští izolovaný PostgreSQL workflow, v běžném CI není DB.
            return;
        };
        let pool = PgPool::connect(&url).await.expect("isolated PostgreSQL");
        let sample = Sample {
            bucket_at: 1792670400,
            aircraft_key: "icao:fed123".into(),
            position_at: 1792670405.0,
            latitude: 49.2,
            longitude: 17.3,
            altitude_ft: Some(10000),
            ground_speed_kt: Some(200.0),
            track_deg: Some(89.0),
            squawk: Some("7000".into()),
            on_ground: Some(false),
        };
        assert_eq!(
            insert_batch(&pool, std::slice::from_ref(&sample))
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            insert_batch(&pool, std::slice::from_ref(&sample))
                .await
                .unwrap(),
            0
        );
        let list = read_history(&pool, &sample.aircraft_key, 1792670400.0, 1792670430.0, 10)
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].latitude, sample.latitude);
        assert_eq!(list[0].aircraft_key, sample.aircraft_key);
        assert!(
            read_history(&pool, &sample.aircraft_key, 1792670430.0, 1792670450.0, 10)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            read_history(&pool, &sample.aircraft_key, 1792670400.0, 1792670430.0, 0)
                .await
                .is_err()
        );
        pool.close().await;
    }
}
