use clap::{Clap, AppSettings};

/// Running a server with arguments for the coins
#[derive(Clap, Clone)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// A list of probabilities for each coin
    pub coin_probs: Vec<f64>,
	/// Verbose output
	#[clap(short, long)]
	pub verbose: bool,
}