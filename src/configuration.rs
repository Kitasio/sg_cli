use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub edition: u32,
    pub name: String,
    pub description: String,
    pub image: String,
    pub dna: String,
    pub stage: u8,
    pub frozen: bool,
    pub seller_fee_basis_points: u32,
    pub fee_recipient: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Parser)]
#[clap(name = "Soul genesis CLI")]
#[clap(about = "CLI to manage the SoulGenesis Project (env variable CONN_STR required, connection string for postgres)", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(long, short)]
    pub verbose: bool,

    #[clap(long, short)]
    pub force: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Changes the stage to (1..5)
    #[clap(arg_required_else_help = true)]
    ChangeStage { stage: u8 },

    /// Changes the host for every image (pass only host, example: media.images.example.com)
    #[clap(arg_required_else_help = true)]
    ChangeHost { host: String },

    /// Change metadata image url to (35, 45 and 100 accepted)
    #[clap(arg_required_else_help = true)]
    ChangeMetadataFaded { opacity: u8 },

    /// Restores the images back to normal
    RestoreImages,

    /// Show all frozen images
    ShowFrozen,

    /// Dumps all metadata entries from the database to json files
    DumpMetadata,

    /// Resets metadata in db from files in json folder
    InitFrom { endpoint: String },
}