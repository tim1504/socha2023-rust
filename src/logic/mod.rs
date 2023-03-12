mod node;
use node::Node;

use log::{info, debug};
use std::{time, fs::OpenOptions};
use std::io::prelude::*;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

pub struct OwnLogic {
    pub time: u32,
    pub exploration_constant: f64, 
    pub n_simulations: u32,
    pub our_team: Option<Team>,
    pub n_iterations: Option<u32>,
    pub test: bool,
}

impl GameClientDelegate for OwnLogic {

    fn on_welcome(&mut self, team: Team) {
        self.our_team = Some(team);
    }

    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("Requested move");

        //Create root node
        let mut root = Node::new(state.clone());

        //Expand root node
        root.expand();

        let start = time::Instant::now();

        let mut iterations: u32 = 0;

        //Run MCTS algorithm for given time or default 1 second
        while start.elapsed().as_millis() < self.time as u128 {
            iterations+=1;
            if self.n_iterations.is_some() && self.n_iterations.unwrap() <= iterations {
                break;
            }
            root.mcts(&state.current_team(), self.exploration_constant, self.n_simulations);
        }

        //Return best move
        root.select_child(self.exploration_constant).state.last_move().unwrap()

    }

    fn on_game_end(&mut self, result: &socha_client_2023::protocol::GameResult) {

        if !self.test {return}
    
        let result_string = if result.winner().is_none() {
            "\ndraw"
        } else if result.winner().as_ref().unwrap().team().eq(&self.our_team.unwrap()) {
            "\nwin"
        } else {
            "\nloss"
        };

        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("result.txt")
            .expect("Couldn't find result.txt!");
        file.write_all(result_string.as_bytes()).expect("Couldn't append to result.txt!");
    }

    fn on_update_state(&mut self, state: &State) { debug!("Board:\n{}", state.board()) }
    
}
