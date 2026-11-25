//! Standalone loopback-first F4-d web shop host.
#![forbid(unsafe_code)]

use kr_service::{
    account_worker::{EconomySettings, Worker},
    accounts::{EconomyPolicy, ShopCatalog},
    web_shop,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    env, fs, io,
    path::{Component, Path, PathBuf},
    time::Duration,
};

const MAX_CONFIG_BYTES: u64 = 64 * 1024;
const MAX_PROFILE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    database: PathBuf,
    #[serde(default)]
    economy_profile_path: Option<PathBuf>,
    #[serde(default)]
    economy_policy: Option<EconomyPolicy>,
    #[serde(default)]
    shop_catalog: Option<ShopCatalog>,
    #[serde(default)]
    summary: Option<Value>,
    web_shop: web_shop::Config,
}

fn usage() -> ! {
    eprintln!("usage: kr-webshop --config <path>");
    std::process::exit(2);
}

fn read_bounded(path: &Path, limit: u64, label: &str) -> io::Result<Vec<u8>> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{label} must not contain '..'"),
        ));
    }
    let meta = fs::metadata(path)?;
    if !meta.is_file() || meta.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{label} size out of bounds"),
        ));
    }
    fs::read(path)
}

fn load(path: &Path) -> io::Result<ConfigFile> {
    let bytes = read_bounded(path, MAX_CONFIG_BYTES, "config")?;
    serde_json::from_slice(&bytes).map_err(|e| io::Error::other(format!("{e:?}")))
}

fn load_economy(config: &ConfigFile) -> io::Result<EconomySettings> {
    let profile = if let Some(path) = &config.economy_profile_path {
        let bytes = read_bounded(path, MAX_PROFILE_BYTES, "economy_profile_path")?;
        serde_json::from_slice::<Value>(&bytes).map_err(|e| io::Error::other(format!("{e:?}")))?
    } else {
        serde_json::json!({
            "economy_policy": config.economy_policy.clone().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "economy_policy is required"))?,
            "shop_catalog": config.shop_catalog.clone().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "shop_catalog is required"))?
        })
    };
    let policy: EconomyPolicy =
        serde_json::from_value(profile.get("economy_policy").cloned().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "economy_policy is missing")
        })?)
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
    let catalog: ShopCatalog =
        serde_json::from_value(profile.get("shop_catalog").cloned().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "shop_catalog is missing")
        })?)
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
    EconomySettings::new(policy, catalog).map_err(|e| io::Error::other(format!("{e:?}")))
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut args = env::args_os();
    if args.next().is_none() {
        usage();
    }
    let flag = args.next().unwrap_or_default();
    if flag != "--config" {
        usage();
    }
    let path = args.next().map(PathBuf::from).unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }
    let config = load(&path)?;
    config.web_shop.validate()?;
    let _ = &config.summary;
    let economy = load_economy(&config)?;
    let worker = Worker::open_with_economy(config.database, 64, Duration::from_secs(5), economy)
        .await
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
    let service = worker.service();
    let auth = web_shop::WebShopAuth::new(config.web_shop.session_ttl_seconds)?;
    let server = tokio::spawn(web_shop::serve(config.web_shop, service, auth));
    tokio::select! {
        result = server => result.map_err(|e| io::Error::other(format!("{e:?}")))??,
        _ = tokio::signal::ctrl_c() => {}
    }
    worker
        .shutdown()
        .await
        .map_err(|e| io::Error::other(format!("{e:?}")))
}
