mod node;
use node::Node;

use log::{info, debug};
use std::time;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

pub struct OwnLogic {
    pub time: u32,
    pub exploration_constant: f64, 
    pub n_simulations: u32,
}

impl GameClientDelegate for OwnLogic {

    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("Requested move");

        //Create root node
        let mut root = Node::new(state.clone());

        //Expand root node
        root.expand();

        let start = time::Instant::now();

        //Run MCTS algorithm for given time or default 1 second
        while start.elapsed().as_millis() < self.time as u128 {
            root.mcts(&state.current_team(), self.exploration_constant, self.n_simulations);
        }

        //Return best move
        root.select_child(self.exploration_constant).state.last_move().unwrap()

    }

    fn on_game_end(&mut self, _result: &socha_client_2023::protocol::GameResult) {}

    fn on_update_state(&mut self, state: &State) { debug!("Board:\n{}", state.board()) }
    
}
