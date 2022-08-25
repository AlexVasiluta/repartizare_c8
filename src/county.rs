use scraper::{Html, Selector};
use titlecase::titlecase;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct County {
    #[sqlx(default)]
    pub id: i32,
    pub code: String,
    pub name: String,
}

pub async fn get_all(year: i32) -> Result<Vec<County>, reqwest::Error> {
    let mut result = Vec::new();

    let sel = Selector::parse(".county .card-body").unwrap();

    let resp = reqwest::get(format!(
        "http://static.admitere.edu.ro/{year}/repartizare/index.html"
    ))
    .await?;

    if resp.status().as_u16() != 200 {
        panic!("Year was probably deleted.")
    }

    let body = resp.text().await?;

    let doc = Html::parse_document(&body);

    let mut i = 0;
    for element in doc.select(&sel) {
        match element.value().attr("href") {
            Some(val) => {
                let inner_val: String = element.text().collect();
                let inner_val = match inner_val.strip_prefix(" ") {
                    Some(val) => String::from(val),
                    None => inner_val,
                };

                result.push(County {
                    id: i,
                    code: String::from(val.trim_end_matches("/index.html")),
                    name: String::from(titlecase(inner_val.as_str())),
                });

                i += 1;
            }
            None => continue,
        }
    }

    Ok(result)
}
