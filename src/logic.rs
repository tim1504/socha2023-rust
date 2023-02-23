use crate::args;
use args::ClientArgs;

use log::{info, debug};
use rand::seq::SliceRandom;
use std::time;
use clap::Parser;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

pub struct OwnLogic;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        //Parse arguments
        let args = ClientArgs::parse();
        let time = args.time;
        let exploration_constant = args.exploration_constant;
        let n_simulations = args.n_simulations;

        info!("Requested move");

        //Create root node
        let mut root = Node::new(state.clone());

        //Expand root node
        root.expand();

        let start = time::Instant::now();

        //Run MCTS algorithm for given time or default 1 second
        while start.elapsed().as_millis() < time as u128 {
            root.mcts(&state.current_team(), exploration_constant, n_simulations);
        }

        //Return best move
        root.select_child(exploration_constant).state.last_move().unwrap()

    }

    fn on_game_end(&mut self, _result: &socha_client_2023::protocol::GameResult) {}

    fn on_update_state(&mut self, state: &State) { debug!("Board:\n{}", state.board()) }
    
}

//Node struct for MCTS algorithm
struct Node {
    state: State,
    children: Vec<Node>,
    visits: u32,
    total: i32,
}

//Node methods
impl Node {

    // Constructor
    fn new(state: State) -> Self {
        Node {
            state,
            children: Vec::new(),
            visits: 0,
            total: 0,
        }
    }

    // MCTS algorithm
    fn mcts(&mut self, team: &Team, c: f64, n: u32) -> i32 {
        let mut result = 0;
        if self.visits > 0 && !self.state.is_over(){
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child(c);
            result = selected_child.mcts(team, c, n);
        } else {
            for _i in 0..n {
                result += self.rollout(team);
            }
        }
        self.visits += 1;
        self.total += result;
        return result;
    }
    
    // Selects the best child node based on the UCB1 formula
    fn select_child(&mut self, c: f64) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut() {
            let mut score = (child.total as f64 / child.visits as f64) + c * ((self.visits as f64).ln() / child.visits as f64).sqrt();
            if child.visits == 0 {
                score = f64::MAX;
            }
            if score > best_score {
                best_child = Some(child);
                best_score = score;
            }
        }
        best_child.unwrap()
    }

    // Expands the node by creating a child node for each possible move
    fn expand(&mut self) {
        for m in self.state.possible_moves() {
            let mut next_state = self.state.clone();
            next_state.perform(m);
            self.children.push(Node::new(next_state));
        }
    }

    // Performs a random rollout from the current state
    // Returns the difference in fish between the current team and the opponent
    fn rollout(&mut self, team: &Team) -> i32 {
        let mut state = self.state.clone();
        while !state.is_over() {
            let random_move = *state.possible_moves()
                .choose(&mut rand::thread_rng())
                .expect("No move found!");
            state.perform(random_move);
        }
        (state.fish(team.to_owned()) - state.fish(team.opponent())) as i32
    }

}