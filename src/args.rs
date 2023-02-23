use clap::Parser;

#[derive(Parser, Debug)]
pub struct ClientArgs {

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