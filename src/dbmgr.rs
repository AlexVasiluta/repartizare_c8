use crate::{county::County, specializare::Specializare, student::Student};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DB {
    prefix: std::path::PathBuf,
    pools: Arc<RwLock<std::collections::HashMap<i32, sqlx::SqlitePool>>>,
}

impl DB {
    pub fn new(prefix: String) -> DB {
        DB {
            prefix: std::path::Path::new(prefix.as_str()).to_owned(),
            pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get_year_pool(
        &self,
        year: i32,
    ) -> Result<sqlx::SqlitePool, Box<dyn std::error::Error>> {
        let pools = self.pools.read().await;

        if let Some(pool) = pools.get(&year) {
            return Ok(pool.clone());
        }

        drop(pools);

        let mut pools = self.pools.write().await;

        let pool = crate::db::create_pool(
            format!(
                "sqlite://{}",
                self.prefix
                    .join(format!("{year}.db"))
                    .to_str()
                    .ok_or("Failed to create db path")?
            )
            .as_str(),
            false,
        )
        .await?;
        if let Some(old_pool) = pools.insert(year, pool) {
            old_pool.close().await
        }

        Ok(pools.get(&year).unwrap().clone())
    }

    pub async fn get_counties(&self, year: i32) -> Result<Vec<County>, Box<dyn std::error::Error>> {
        let pool = self.get_year_pool(year).await?;
        let counties = sqlx::query_as::<_, County>("SELECT * FROM counties ORDER BY code ASC;")
            .fetch_all(&pool)
            .await?;

        Ok(counties)
    }

    pub async fn get_schools(
        &self,
        year: i32,
        county: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let pool = self.get_year_pool(year).await?;

        #[derive(sqlx::FromRow)]
        struct Result {
            liceu: String,
        }

        let schools = sqlx::query_as::<_, Result>(
            "SELECT liceu FROM specializari WHERE judet = ? GROUP BY liceu ORDER BY liceu ASC",
        )
        .bind(county)
        .fetch_all(&pool)
        .await?;

        Ok(schools.iter().map(|x| x.liceu.clone()).collect())
    }

    pub async fn get_full_school(
        &self,
        year: i32,
        county: &str,
        school: &str,
    ) -> Result<FullSchool, Box<dyn std::error::Error>> {
        let pool = self.get_year_pool(year).await?;

        let specs = sqlx::query_as::<_, SpecShort>(
            "SELECT id, name FROM specializari WHERE judet = ? AND liceu = ? ORDER BY id ASC",
        )
        .bind(county)
        .bind(school)
        .fetch_all(&pool)
        .await?;

        let mut school = FullSchool {
            specializari: std::collections::HashMap::new(),
            specializari_short: specs,
        };

        for spec in &school.specializari_short {
            school.specializari.insert(spec.id, FullSpec {
                elevi: sqlx::query_as::<_, Student>(
                    "SELECT * FROM students WHERE judet = ? AND id_specializare = ? ORDER BY medie_adm DESC",
                ).bind(county).bind(spec.id).fetch_all(&pool).await?,

                spec: sqlx::query_as::<_, Specializare>(
                    "SELECT * FROM specializari WHERE judet = ? AND id = ?",
                ).bind(county).bind(spec.id).fetch_one(&pool).await?,
            });
        }

        Ok(school)
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SpecShort {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct FullSpec {
    pub elevi: Vec<Student>,
    #[serde(rename = "sp")]
    pub spec: Specializare,
}

#[derive(Serialize)]
pub struct FullSchool {
    #[serde(rename = "specs")]
    pub specializari_short: Vec<SpecShort>,
    #[serde(rename = "spec_data")]
    pub specializari: std::collections::HashMap<i32, FullSpec>,
}
