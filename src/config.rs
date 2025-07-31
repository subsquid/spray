use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::Uri;


#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub sources: HashMap<String, GeyserConfig>
}


#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeyserConfig {
    #[serde_as(as = "DisplayFromStr")]
    pub url: Uri,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub x_token: Option<AsciiMetadataValue>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub x_access_token: Option<AsciiMetadataValue>
}


impl Config {
    pub fn read(file: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(file)?;
        let cfg = serde_yaml::from_reader(BufReader::new(file))?;
        Ok(cfg)
    }
}