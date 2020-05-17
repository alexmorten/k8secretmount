#[macro_use]
extern crate log;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, ListParams, Meta, WatchEvent},
    Client,
};

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const USAGE_STRING: &str = "Usage: k8secretmount <secret name> <mount path> [<namespace>]";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secret_name = match std::env::args().nth(1) {
        Some(value) => value,
        None => {
            eprintln!("{}", USAGE_STRING);
            std::process::exit(1);
        }
    };

    let folder_path = match std::env::args().nth(2) {
        Some(value) => value,
        None => {
            eprintln!("{}", USAGE_STRING);
            std::process::exit(1);
        }
    };

    let namespace = std::env::args().nth(3).unwrap_or("default".into());

    let folder_path = Path::new(&folder_path);
    create_if_not_exists(folder_path)?;

    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let client = Client::try_default().await?;

    let secrets: Api<Secret> = Api::namespaced(client, &namespace);
    let mut lp = ListParams::default();
    lp.timeout = None;
    let mut stream = secrets.watch(&lp, "0").await?.boxed();

    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(s) => {
                if Meta::name(&s) == secret_name {
                    info!("updating mount path");
                    write_secret_data(s, folder_path)?
                }
            }
            WatchEvent::Modified(s) => {
                if Meta::name(&s) == secret_name {
                    info!("updating mount path");
                    write_secret_data(s, folder_path)?
                }
            }
            WatchEvent::Deleted(s) => println!("Deleted {}", Meta::name(&s)),
            WatchEvent::Bookmark(_s) => {}
            WatchEvent::Error(s) => println!("{}", s),
        }
    }
    Ok(())
}

fn write_secret_data(s: k8s_openapi::api::core::v1::Secret, folder: &Path) -> std::io::Result<()> {
    let data = match s.data {
        Some(d) => d,
        None => return Ok(()),
    };
    for (name, content) in &data {
        let mut file = File::create(folder.join(name))?;
        file.write_all(&content.0)?;
    }

    let files = fs::read_dir(folder)?;
    for file in files {
        let file = file?;
        let file_name = file.file_name().into_string();
        match file_name {
            Ok(name) => {
                if !data.contains_key(&name) {
                    info!("file {} is not part of the secret, removing ...", name);
                    fs::remove_file(file.path())?;
                }
            }
            Err(_) => info!("directoy contains a file with a non utf8 name, ignoring"),
        }
    }
    Ok(())
}

fn create_if_not_exists(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    fs::create_dir_all(path)
}
