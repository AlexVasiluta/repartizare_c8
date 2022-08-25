use sqlx::Executor;

use crate::county::County;

#[derive(Debug, serde::Deserialize)]
struct RawSpecializare {
    #[serde(alias = "j")]
    judet: String,
    #[serde(alias = "c")]
    cod: String,

    #[serde(alias = "l")]
    liceu: String,
    #[serde(alias = "lc")]
    _liceu_id: String,

    #[serde(alias = "m")]
    mediu: String,
    #[serde(alias = "sp")]
    specializare: String,
    #[serde(alias = "lp")]
    _limba_predare: String,
    #[serde(alias = "lb")]
    limba_bilingv: String,

    #[serde(alias = "nlt")]
    nr_locuri_total: String,
    #[serde(alias = "nlo")]
    nr_locuri_ocupate: String,

    #[serde(alias = "fi")]
    _forma_invatamant: String,
    #[serde(alias = "p")]
    profil: String,
    #[serde(alias = "f")]
    filiera: String,
    #[serde(alias = "n")]
    _nivel: String,

    #[serde(alias = "um")]
    ultima_medie: Option<String>,
    #[serde(alias = "uma")]
    ultima_medie_anterior: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Specializare {
    pub id: i32,

    pub name: String,
    pub judet: String,

    pub liceu: String,
    pub mediu: String,

    pub specializare: String,
    pub bilingv: bool,

    pub locuri: i32,
    pub ocupate: i32,

    pub profil: String,
    pub filiera: String,

    pub ultima_medie: f64,
    #[serde(rename = "ultima_medie_ant")]
    #[sqlx(rename = "ultima_medie_ant")]
    pub ultima_medie_anterior: f64,
}

impl Specializare {
    fn from_raw(st: &RawSpecializare) -> Specializare {
        let mut name = format!("{}: {}", st.cod, st.specializare);
        if st.limba_bilingv != "-" {
            name = format!("{} (Bilingv {})", name, st.limba_bilingv);
        }

        Specializare {
            id: st.cod.parse().unwrap(),
            name: name,
            judet: st.judet.clone(),
            liceu: st.liceu.clone(),
            mediu: st.mediu.clone(),
            specializare: st.specializare.clone(),
            bilingv: st.limba_bilingv != "-",
            locuri: st.nr_locuri_total.parse().unwrap(),
            ocupate: st.nr_locuri_ocupate.parse().unwrap(),
            profil: st.profil.clone(),
            filiera: st.filiera.clone(),
            ultima_medie: match st.ultima_medie.clone() {
                Some(um) => um.parse().unwrap_or(-1.0),
                None => -1.0,
            },
            ultima_medie_anterior: st.ultima_medie_anterior.parse().unwrap_or(-1.0),
        }
    }

    pub fn nerepartizat(county: &County) -> Specializare {
        let nerep = format!("Nerepartizat {}", county.code);
        Specializare {
            id: -county.id,
            name: nerep.clone(),
            judet: county.code.clone(),
            liceu: "-".to_string(),
            mediu: "-".to_string(),
            specializare: nerep.clone(),
            bilingv: false,
            locuri: -1,
            ocupate: -1,
            profil: nerep.clone(),
            filiera: "-".to_string(),
            ultima_medie: -1.0,
            ultima_medie_anterior: -1.0,
        }
    }
}

async fn get_all(year: i32, county: &County) -> Result<Vec<Specializare>, reqwest::Error> {
    let body = reqwest::get(
        format!(
            "http://static.admitere.edu.ro/{year}/repartizare/{}/data/specialization.json",
            county.code
        )
        .as_str(),
    )
    .await?
    .json::<Vec<RawSpecializare>>()
    .await?;

    Ok(body
        .iter()
        .map(|x| Specializare::from_raw(x))
        .collect::<Vec<Specializare>>())
}

pub async fn insert_specializari(
    year: i32,
    county: &County,
    db: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = db.begin().await?;
    insert_specializare(&Specializare::nerepartizat(county), &mut tx).await?;
    for sp in get_all(year, county).await? {
        insert_specializare(&sp, &mut tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn insert_specializare(
    sp: &Specializare,
    db: &mut sqlx::SqliteConnection,
) -> Result<(), sqlx::Error> {
    match db.execute(sqlx::query(
        "
INSERT INTO specializari
(id, name, liceu, mediu, judet, specializare, bilingv, locuri, ocupate, profil, filiera, ultima_medie, ultima_medie_ant)
VALUES
(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
)
    .bind(&sp.id)
    .bind(&sp.name)
    .bind(&sp.liceu)
    .bind(&sp.mediu)
    .bind(&sp.judet)
    .bind(&sp.specializare)
    .bind(&sp.bilingv)
    .bind(&sp.locuri)
    .bind(&sp.ocupate)
    .bind(&sp.profil)
    .bind(&sp.filiera)
    .bind(&sp.ultima_medie)
    .bind(&sp.ultima_medie_anterior)
)
    .await {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("{:#?}", sp);
            Ok(())
        }
    }
}
