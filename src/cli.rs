use clap::Parser;


#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct CLI {
    /// Config file
    #[arg(value_name = "config")]
    pub config: String
}