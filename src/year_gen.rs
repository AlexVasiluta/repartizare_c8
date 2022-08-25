use crate::*;

pub async fn do_year(year: i32) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::create_pool(format!("sqlite://{year}.db").as_str(), true).await?;

    // insert counties
    let counties = county::get_all(year).await?;
    futures::future::join_all(counties.iter().map(|county| async {
        sqlx::query("INSERT INTO counties (code, name) VALUES (?, ?) ON CONFLICT DO NOTHING")
            .bind(county.code.clone())
            .bind(county.name.clone())
            .fetch_all(&db)
            .await
    }))
    .await;

    let mut handles = Vec::new();
    for county in counties {
        let county1 = county.clone();
        let db1 = db.clone();
        handles.push(tokio::spawn(async move {
            // insert specializari
            specializare::insert_specializari(year, &county1, &db1)
                .await
                .unwrap();
            // insert students
            student::insert_students(year, &county1, &db1)
                .await
                .unwrap();
            println!("Finished {} for year {}", county1.name, year);
        }));
    }
    futures::future::join_all(handles)
        .await
        .iter()
        .for_each(|x| match x {
            Err(err) => println!("{:#?}", err),
            _ => {}
        });

    db.close().await;

    Ok(())
}
