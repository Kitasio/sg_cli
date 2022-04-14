use std::error::Error;
use clap::StructOpt;
use sqlx::postgres::PgPoolOptions;
use std::process;

pub mod configuration;
pub mod functions;
pub mod helper_funcs;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let conn_str = std::env::var("CONN_STR").expect("CONN_STR not set!");
    let pool = PgPoolOptions::new()
        .connect(conn_str.as_str()).await?;

    let args = configuration::Cli::parse();

    match &args.command {
        configuration::Commands::ChangeStage { stage } => {
            println!("changing stage to: {}", stage);
            if let Err(e) = functions::change_stage(*stage, args.verbose, pool).await {
                eprintln!("Error changing stage: {}", e);
                process::exit(1)
            }
        }
        configuration::Commands::ChangeHost { host } => {
            println!("changing host to: {}", host);
            if let Err(e) = functions::change_host_for_images(host, args.verbose, pool).await {
                eprintln!("Error changing host: {}", e);
                process::exit(1)
            }
        }
        configuration::Commands::ChangeMetadataFaded { opacity } => {
            println!(
                "Changing metadata image url to images with opacity: {}",
                opacity
            );
            if let Err(e) = functions::change_metadata_faded(*opacity, args.verbose, pool).await {
                eprintln!("Error changing metadata to faded images: {}", e);
                process::exit(1)
            }
        }
        configuration::Commands::RestoreImages => {
            println!("Restoring images based on edition and stage");
            if let Err(e) = functions::restore_images(args.verbose, args.force, pool).await {
                eprintln!("Error restoring images: {}", e);
                process::exit(1)
            }
        }
        configuration::Commands::ShowFrozen => {
            if let Err(e) = functions::show_frozen(pool).await {
                eprintln!("Error showing frozen images: {}", e);
                process::exit(1)
            }
        }
        configuration::Commands::DumpMetadata => {
            if let Err(e) = functions::generate_json_files(pool).await {
                eprintln!("Error dumping metadata: {}", e);
                process::exit(1)
            }
        }
        configuration::Commands::InitFrom { endpoint } => {
            if let Err(e) = functions::db_init_from_endpoint(endpoint, pool).await {
                eprintln!("Error initiating db from endpoint: {}", e);
                process::exit(1)
            }
        }
    }

    Ok(())
}