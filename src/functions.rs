use crate::configuration::Metadata;
use crate::helper_funcs::increment_image_stage;
use sqlx::PgPool;
use std::error::Error;
use std::path::Path;
use std::{fs, process};
use url::Url;

pub async fn db_init_from_endpoint(endpoint: &str, pool: PgPool) -> Result<(), Box<dyn Error>> {
    let data = reqwest::get(endpoint).await?.text().await?;
    let data: Vec<Metadata> = serde_json::from_str(data.as_str())?;

    for i in data {
        println!("Inserting metadata: {}", i.edition);
        let _ = sqlx::query!(
            r#"
        INSERT INTO metadata (edition, data)
        VALUES ($1, $2)
        RETURNING edition
        "#,
            i.edition as i32,
            sqlx::types::Json(i) as _
        )
        .fetch_one(&pool)
        .await?;
    }

    Ok(())
}

pub async fn change_host_for_images(
    host: &str,
    verbose: bool,
    pool: PgPool,
) -> Result<(), Box<dyn Error>> {
    let records =
        sqlx::query!("SELECT edition, data FROM metadata")
            .fetch_all(&pool)
            .await?;

    for r in records {
        let mut data: Metadata =
            serde_json::from_value(r.data).expect("Cannot parse data from value");

        let mut url = Url::parse(&data.image.as_str()).expect("Failed to parse image Url");
        url.set_host(Some(host))
            .expect(format!("Could not set a new host for {}", data.edition).as_str());
        data.image = url.to_string();

        let d = serde_json::to_value(&data).expect("Failed to convert to serde value");

        let update_record = sqlx::query!(
            "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
            d,
            r.edition
        )
        .fetch_one(&pool)
        .await?;

        if verbose {
            println!("Updated host in edition: {}", update_record.edition)
        }
    }
    Ok(())
}

pub async fn change_stage(stage: u8, verbose: bool, pool: PgPool) -> Result<(), Box<dyn Error>> {
    let rec = sqlx::query!(
        "UPDATE current_stage SET stage = $1 RETURNING stage",
        stage as i16
    )
    .fetch_one(&pool)
    .await?;

    if verbose {
        println!("Updated current stage to: {}", rec.stage);
    }

    let records =
        sqlx::query!("SELECT edition, data FROM metadata WHERE data -> 'frozen' = 'false'")
            .fetch_all(&pool)
            .await?;

    for r in records {
        let mut data: Metadata =
            serde_json::from_value(r.data).expect("Cannot parse data from value");

        data.image = increment_image_stage(&data.image.as_str(), stage);
        data.stage = stage;

        let d = serde_json::to_value(&data).expect("Failed to convert to serde value");

        let update_record = sqlx::query!(
            "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
            d,
            r.edition
        )
        .fetch_one(&pool)
        .await?;

        if verbose {
            println!(
                "Updated edition {} to stage {}",
                update_record.edition, stage
            )
        }
    }

    Ok(())
}

pub async fn show_frozen(pool: PgPool) -> Result<(), Box<dyn Error>> {
    let records = sqlx::query!("SELECT data -> 'image' as image, data -> 'stage' as stage FROM metadata WHERE data -> 'frozen' = 'true'")
        .fetch_all(&pool)
        .await?;

    for r in records {
        let image: String = serde_json::from_value(r.image.unwrap()).unwrap();
        let stage: i16 = serde_json::from_value(r.stage.unwrap()).unwrap();
        println!("IMAGE STAGE\n{} {}", image, stage);
    }

    Ok(())
}

// creates a json file from every entry in the database and saves it to directory from argument
pub async fn generate_json_files(pool: PgPool) -> Result<(), Box<dyn Error>> {
    let records = sqlx::query!("SELECT edition, data FROM metadata ORDER BY edition")
        .fetch_all(&pool)
        .await?;

    let mut all_metadata: Vec<Metadata> = vec![];

    for r in records {
        let data: Metadata = serde_json::from_value(r.data).expect("Cannot parse data from value");
        all_metadata.push(data);
    }

    let now = chrono::Utc::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let path = Path::new("json-dumps");
    fs::create_dir_all(path).expect("Failed to create json folder");
    let path = path.to_path_buf();
    let path = path.join(format!("{}.json", timestamp));
    let mut file = fs::File::create(path).expect("Failed to create file");
    serde_json::to_writer_pretty(&mut file, &all_metadata).expect("Failed to write to file");

    Ok(())
}

pub async fn restore_images(verbose: bool, force: bool, pool: PgPool) -> Result<(), Box<dyn Error>> {
    match force {
        true => {
            let records = sqlx::query!("SELECT edition, data FROM metadata")
            .fetch_all(&pool)
            .await?;

            for r in records {
                let mut data: Metadata =
                    serde_json::from_value(r.data).expect("Failed to parse data from value");
        
                let mut url = Url::parse(&data.image.as_str()).expect("Failed to parse image Url");
                let path = format!("images/{}/{}.jpg", data.stage, data.edition);
                url.set_path(path.as_str());
        
                data.image = url.to_string();
        
                let d = serde_json::to_value(&data).expect("Failed to convert to serde value");
        
                let update_record = sqlx::query!(
                    "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
                    d,
                    r.edition
                )
                .fetch_one(&pool)
                .await?;
                if verbose {
                    println!(
                        "Updated edition {} -> {}",
                        update_record.edition, data.image
                    );
                }
            }
        
        },
        false => {
            let records = sqlx::query!("SELECT edition, data FROM metadata WHERE data -> 'frozen' = 'false'")
            .fetch_all(&pool)
            .await?;

            for r in records {
                let mut data: Metadata =
                    serde_json::from_value(r.data).expect("Failed to parse data from value");
        
                let mut url = Url::parse(&data.image.as_str()).expect("Failed to parse image Url");
                let path = format!("images/{}/{}.jpg", data.stage, data.edition);
                url.set_path(path.as_str());
        
                data.image = url.to_string();
        
                let d = serde_json::to_value(&data).expect("Failed to convert to serde value");
        
                let update_record = sqlx::query!(
                    "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
                    d,
                    r.edition
                )
                .fetch_one(&pool)
                .await?;
                if verbose {
                    println!(
                        "Updated edition {} -> {}",
                        update_record.edition, data.image
                    );
                }
            }
        
        }
    }

    Ok(())
}

pub async fn change_metadata_faded(
    opacity: u8,
    verbose: bool,
    pool: PgPool,
) -> Result<(), Box<dyn Error>> {
    let records =
        sqlx::query!("SELECT edition, data FROM metadata WHERE data -> 'frozen' = 'false'")
            .fetch_all(&pool)
            .await?;

    for r in records {
        let mut data: Metadata =
            serde_json::from_value(r.data).expect("Failed to parse data from value");
        match opacity {
            35 => {
                let mut url = Url::parse(&data.image.as_str()).expect("Failed to parse image Url");
                let path = format!("faded35/4/{}.jpg", data.edition);
                url.set_path(path.as_str());
        
                data.image = url.to_string();

                let d = serde_json::to_value(&data).expect("Failed to convert to serde value");
                let update_record = sqlx::query!(
                    "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
                    d,
                    r.edition
                )
                .fetch_one(&pool)
                .await?;
                if verbose {
                    println!(
                        "Updated edition {} -> {}",
                        update_record.edition, data.image
                    );
                }
            }
            45 => {
                let mut url = Url::parse(&data.image.as_str()).expect("Failed to parse image Url");
                let path = format!("faded45/4/{}.jpg", data.edition);
                url.set_path(path.as_str());
        
                data.image = url.to_string();

                let d = serde_json::to_value(&data).expect("Failed to convert to serde value");
                let update_record = sqlx::query!(
                    "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
                    d,
                    r.edition
                )
                .fetch_one(&pool)
                .await?;
                if verbose {
                    println!(
                        "Updated edition {} -> {}",
                        update_record.edition, data.image
                    );
                }
            }
            100 => {
                data.image = "https://sg-data.fra1.cdn.digitaloceanspaces.com/black.jpg".to_string();
                let d = serde_json::to_value(&data).expect("Failed to convert to serde value");
                let update_record = sqlx::query!(
                    "UPDATE metadata SET data = $1 WHERE edition = $2 RETURNING edition",
                    d,
                    r.edition
                )
                .fetch_one(&pool)
                .await?;
                if verbose {
                    println!(
                        "Updated edition {} -> {}",
                        update_record.edition, data.image
                    );
                }
            }
            _ => {
                eprintln!("opacity can be 35, 45 and 100 only");
                process::exit(1)
            }
        }
    }

    Ok(())
}
