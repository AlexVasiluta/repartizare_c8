use crate::*;
use rocket::serde::{json::Json, Serialize};
use serde::ser::SerializeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const AVAILABLE_YEARS: [i32; 3] = [2020, 2021, 2022];
const PORT: i32 = 8095;

struct DB {
    pools: Arc<RwLock<std::collections::HashMap<i32, sqlx::SqlitePool>>>,
}

impl DB {
    fn new() -> DB {
        DB {
            pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    async fn get_year_pool(&self, year: i32) -> Result<sqlx::SqlitePool, sqlx::Error> {
        let pools = self.pools.read().await;

        if let Some(pool) = pools.get(&year) {
            return Ok(pool.clone());
        }

        drop(pools);

        let mut pools = self.pools.write().await;

        let pool = db::create_pool(format!("sqlite://{year}.db").as_str(), false).await?;
        if let Some(old_pool) = pools.insert(year, pool) {
            old_pool.close().await
        }

        Ok(pools.get(&year).unwrap().clone())
    }

    async fn get_counties(&self, year: i32) -> Result<Vec<county::County>, sqlx::Error> {
        let pool = self.get_year_pool(year).await?;
        let counties =
            sqlx::query_as::<_, county::County>("SELECT * FROM counties ORDER BY code ASC;")
                .fetch_all(&pool)
                .await?;

        Ok(counties)
    }

    async fn get_schools(&self, year: i32, county: &str) -> Result<Vec<String>, sqlx::Error> {
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

    async fn get_full_school(
        &self,
        year: i32,
        county: &str,
        school: &str,
    ) -> Result<FullSchool, sqlx::Error> {
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
                elevi: sqlx::query_as::<_, student::Student>(
                    "SELECT * FROM students WHERE judet = ? AND id_specializare = ? ORDER BY medie_adm DESC",
                ).bind(county).bind(spec.id).fetch_all(&pool).await?,

                spec: sqlx::query_as::<_, specializare::Specializare>(
                    "SELECT * FROM specializari WHERE judet = ? AND id = ?",
                ).bind(county).bind(spec.id).fetch_one(&pool).await?,
            });
        }

        Ok(school)
    }
}

impl<T> Status<T> {
    fn success(data: T) -> Json<Status<T>> {
        Json(Status {
            result_type: "success".to_string(),
            data: StatusData::Success(data),
        })
    }

    fn error(data: String) -> Json<Status<T>> {
        Json(Status {
            result_type: "error".to_string(),
            data: StatusData::Error(data),
        })
    }
}

#[get("/years")]
fn years() -> Json<Status<Vec<i32>>> {
    Status::success(Vec::from(AVAILABLE_YEARS))
}

#[get("/<year>/counties")]
async fn counties(db: &rocket::State<DB>, year: i32) -> Json<Status<Vec<county::County>>> {
    match db.get_counties(year).await {
        Ok(counties) => Status::success(counties),
        Err(err) => Status::error(err.to_string()),
    }
}

#[get("/<year>/<county>/schools")]
async fn schools(db: &rocket::State<DB>, year: i32, county: &str) -> Json<Status<Vec<String>>> {
    match db.get_schools(year, county).await {
        Ok(schools) => Status::success(schools),
        Err(err) => Status::error(err.to_string()),
    }
}

#[get("/<year>/<county>/fullSchool/<school>")]
async fn school(
    db: &rocket::State<DB>,
    year: i32,
    county: &str,
    school: &str,
) -> Json<Status<FullSchool>> {
    match db.get_full_school(year, county, school).await {
        Ok(school) => Status::success(school),
        Err(err) => Status::error(err.to_string()),
    }
}

#[rocket::main]
pub async fn run_server() -> Result<(), rocket::Error> {
    let _rocket = rocket::custom(rocket::Config::figment().merge(("port", PORT)))
        .manage(DB::new())
        .mount("/adm_api", routes![years, counties, schools, school])
        .launch()
        .await?;

    Ok(())
}

impl Serialize for county::County {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("code", self.code.as_str())?;
        map.serialize_entry("name", self.name.as_str())?;
        map.end()
    }
}

#[derive(Serialize, sqlx::FromRow)]
struct SpecShort {
    id: i32,
    name: String,
}

#[derive(Serialize)]
struct FullSpec {
    elevi: Vec<student::Student>,
    #[serde(rename = "sp")]
    spec: specializare::Specializare,
}

#[derive(Serialize)]
struct FullSchool {
    #[serde(rename = "specs")]
    specializari_short: Vec<SpecShort>,
    #[serde(rename = "spec_data")]
    specializari: std::collections::HashMap<i32, FullSpec>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum StatusData<T> {
    Success(T),
    Error(String),
}

#[derive(Serialize)]
struct Status<T> {
    #[serde(rename = "type")]
    result_type: String,
    data: StatusData<T>,
}
