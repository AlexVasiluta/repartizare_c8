use std::net::SocketAddr;
use std::sync::Arc;

use crate::county::County;
use crate::dbmgr::{FullSchool, DB};
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::{extract::Path, response::Json, routing::get, Extension, Router};
use serde::ser::SerializeMap;
use serde::Serialize;

const AVAILABLE_YEARS: [i32; 4] = [2020, 2021, 2022, 2023];

async fn years() -> Json<Status<Vec<i32>>> {
    Status::success(Vec::from(AVAILABLE_YEARS))
}

async fn counties(
    Extension(db): Extension<Arc<DB>>,
    Path(year): Path<i32>,
) -> Json<Status<Vec<County>>> {
    match db.get_counties(year).await {
        Ok(counties) => Status::success(counties),
        Err(err) => Status::error(err.to_string()),
    }
}

async fn schools(
    Extension(db): Extension<Arc<DB>>,
    Path((year, county)): Path<(i32, String)>,
) -> Json<Status<Vec<String>>> {
    match db.get_schools(year, county.as_str()).await {
        Ok(schools) => Status::success(schools),
        Err(err) => Status::error(err.to_string()),
    }
}

async fn school(
    Extension(db): Extension<Arc<DB>>,
    Path((year, county, school)): Path<(i32, String, String)>,
) -> Json<Status<FullSchool>> {
    match db
        .get_full_school(year, county.as_str(), school.as_str())
        .await
    {
        Ok(school) => Status::success(school),
        Err(err) => Status::error(err.to_string()),
    }
}

async fn callback() -> impl IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        Status::<()>::error("404 Not Found".to_string()),
    )
}

pub async fn run_server(db_prefix: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/adm_api/years", get(years))
        .route("/adm_api/:year/counties", get(counties))
        .route("/adm_api/:year/:county/schools", get(schools))
        .route("/adm_api/:year/:county/fullSchool/:school", get(school))
        .layer(Extension(Arc::new(DB::new(db_prefix))))
        .fallback(callback.into_service());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

impl Serialize for County {
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
