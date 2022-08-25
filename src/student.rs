use crate::county::County;
use regex::Regex;
use sqlx::Executor;

#[derive(Debug, serde::Deserialize)]
struct RawStudent {
    #[serde(alias = "ja")]
    judet_id: String,
    #[serde(alias = "n")]
    id: String,
    #[serde(alias = "jp")]
    _judet_name: String,
    #[serde(alias = "s")]
    scoala_provenienta: String,
    #[serde(alias = "sc")]
    _id_scoala_proevenienta: String,

    #[serde(alias = "madm")]
    medie_admitere: String,
    #[serde(alias = "mev")]
    medie_evaluare: String,
    #[serde(alias = "mabs")]
    medie_absolvire: String,

    #[serde(alias = "nro")]
    nota_ro: String,
    #[serde(alias = "nmate")]
    nota_mate: String,

    #[serde(alias = "lm")]
    _limba_materna: String,
    #[serde(alias = "nlm")]
    _nota_lma: String,

    #[serde(alias = "h")]
    liceu: String,
    #[serde(alias = "sp")]
    specializare: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Student {
    pub id: String,
    pub provenienta: String,
    pub judet: String,

    #[serde(rename = "medie_adm")]
    #[sqlx(rename = "medie_adm")]
    pub medie_admitere: f64,
    #[serde(rename = "medie_en")]
    #[sqlx(rename = "medie_en")]
    pub medie_evaluare: f64,
    #[serde(rename = "medie_abs")]
    #[sqlx(rename = "medie_abs")]
    pub medie_absolvire: f64,

    #[serde(rename = "nota_ro")]
    #[sqlx(rename = "nota_ro")]
    pub nota_romana: f64,
    #[serde(rename = "nota_mate")]
    #[sqlx(rename = "nota_mate")]
    pub nota_mate: f64,

    pub liceu: String,
    pub id_specializare: i32,
    #[serde(rename = "specializare_display")]
    #[sqlx(rename = "specializare_display")]
    pub specializare: String,
}

impl Student {
    fn from_raw(st: &RawStudent, county_id: i32) -> Student {
        let finder_regex = Regex::new("([0-9]+)").unwrap();
        Student {
            id: st.id.clone(),
            provenienta: st.scoala_provenienta.clone(),
            judet: st.judet_id.clone(),

            medie_admitere: st.medie_admitere.parse().unwrap_or(-1.0),
            medie_evaluare: st.medie_evaluare.parse().unwrap_or(-1.0),
            medie_absolvire: st.medie_absolvire.parse().unwrap_or(-1.0),

            nota_romana: st.nota_ro.parse().unwrap(),
            nota_mate: st.nota_mate.parse().unwrap(),

            liceu: st.liceu.clone(),
            id_specializare: if st.specializare == "Nerepartizat" {
                -county_id
            } else {
                finder_regex.captures(&st.specializare).unwrap()[0]
                    .parse()
                    .unwrap()
            },
            specializare: st.specializare.clone(),
        }
    }
}

async fn get_all(year: i32, county: &County) -> Result<Vec<Student>, reqwest::Error> {
    let body = reqwest::get(
        format!(
            "http://static.admitere.edu.ro/{year}/repartizare/{}/data/candidate.json",
            county.code
        )
        .as_str(),
    )
    .await?
    .json::<Vec<RawStudent>>()
    .await?;

    Ok(body
        .iter()
        .map(|x| Student::from_raw(x, county.id))
        .collect::<Vec<Student>>())
}

pub async fn insert_students(
    year: i32,
    county: &County,
    db: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = db.begin().await?;
    for st in get_all(year, &county).await? {
        tx.execute(sqlx::query("
INSERT INTO students 
    (id, provenienta, medie_adm, medie_en, medie_abs, nota_ro, nota_mate, liceu, id_specializare, specializare_display, judet) 
VALUES 
    (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
)
        .bind(&st.id)
        .bind(&st.provenienta)
        .bind(&st.medie_admitere)
        .bind(&st.medie_evaluare)
        .bind(&st.medie_absolvire)
        .bind(&st.nota_romana)
        .bind(&st.nota_mate)
        .bind(&st.liceu)
        .bind(&st.id_specializare)
        .bind(&st.specializare)
        .bind(&st.judet)
    )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
