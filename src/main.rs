mod logic;
mod args;

use std::str::FromStr;
use args::ClientArgs;
use clap::Parser;
use simplelog::{SimpleLogger, Config};
use log::LevelFilter;
use socha_client_2023::{client::{GameClient, DebugMode}};

use logic::OwnLogic;

/// Software Challenge 2023 client.

fn main() {
    // Parse command line arguments
    let args = ClientArgs::parse();

    // Set up logging
    SimpleLogger::init(LevelFilter::from_str(&args.level).expect("Invalid log level."), Config::default()).expect("Could not initialize logger.");
    
    // Setup the client and the delegate
    let debug_mode = DebugMode {
        debug_reader: args.debug_reader,
        debug_writer: args.debug_writer,
    };

    let logic = OwnLogic{
        time: args.time,
        exploration_constant: args.exploration_constant,
        n_simulations: args.n_simulations,
        our_team: None,
        test: args.test,
    }; 

    let client = GameClient::new(logic, debug_mode, args.reservation);
    let _result = client.connect(&args.host, args.port).expect("Error while running client.");
}
