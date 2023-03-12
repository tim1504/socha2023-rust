use clap::Parser;

#[derive(Parser, Debug)]
pub struct ClientArgs {

    /// The amount of time used to process the result
    #[clap(short = 't', long = "time", default_value_t = 1000)]
    pub time: u32,
    /// The exploration constant of the UCB1 formular
    #[clap(short = 'c', long = "exploration_constant", default_value_t = 2.0)]
    pub exploration_constant: f64,
    /// The number of random simulatins per rollout
    #[clap(short = 's', long = "n_simulations", default_value_t = 100)]
    pub n_simulations: u32,
    /// The number of monte carlo simulations per move request
    #[clap(short = 'i', long = "n_iterations")]
    pub n_iterations: Option<u32>,
    #[clap(short, long = "test", default_value_t = false)]
    pub test: bool,

    /// The game server's host address.
    #[clap(short, long, default_value = "localhost")]
    pub host: String,
    /// The game server's port.
    #[clap(short, long, default_value_t = 13050)]
    pub port: u16,
    /// A game reservation.
    #[clap(short, long)]
    pub reservation: Option<String>,
    /// The level to log at.
    #[clap(short, long, default_value = "Info")]
    pub level: String,
    /// Reads incoming XML messages from the console for debugging.
    #[clap(short = 'd', long)]
    pub debug_reader: bool,
    /// Prints outgoing XML messages to the console for debugging.
    #[clap(short = 'D', long)]
    pub debug_writer: bool,

}