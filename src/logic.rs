use log::{info, debug};
use rand::seq::SliceRandom;
use std::time;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

pub struct OwnLogic;

pub const SIMULATIONS_PER_ROLLOUT: u32 = 100;
pub const TIME_LIMIT: u128 = 1800;
pub const EXPLORATION_CONSTANT: f64 = 1.41;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("Requested move");

        //Create root node
        let mut root = Node::new(state.clone());

        //Expand root node
        root.expand();

        let start = time::Instant::now();

        //Run MCTS algorithm for about 2 seconds
        while start.elapsed().as_millis() < TIME_LIMIT {
            root.mcts(&state.current_team());
        }

        //Return best move
        root.select_child(true).state.last_move().unwrap()

    }

    fn on_game_end(&mut self, _result: &socha_client_2023::protocol::GameResult) {}

    fn on_update_state(&mut self, state: &State) { debug!("Board:\n{}", state.board()) }
    
}

//Node struct for MCTS algorithm
struct Node {
    state: State,
    children: Vec<Node>,
    visits: u32,
    total: f64,
}

//Node methods
impl Node {

    // Constructor
    fn new(state: State) -> Self {
        Node {
            state,
            children: Vec::new(),
            visits: 0,
            total: 0.,
        }
    }

    // MCTS algorithm
    fn mcts(&mut self, team: &Team) -> f64 {
        let mut result = 0.;
        if self.visits > 0 && !self.state.is_terminal() {
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child(false);
            result = selected_child.mcts(team);
        } else {
            for _i in 0..SIMULATIONS_PER_ROLLOUT {
                result += self.rollout(team);
            }
        }
        self.visits += SIMULATIONS_PER_ROLLOUT;
        self.total += if self.state.current_team() == *team {1. - result} else {result};
        return result;
    }
    
    // Selects the best child node based on the UCB1 formula
    fn select_child(&mut self, e: bool) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut() {
            let score = if child.visits > 0 {
                child.total / child.visits as f64
                + EXPLORATION_CONSTANT * ((self.visits as f64).ln() / (child.visits as f64)).sqrt()
            } else {
                f64::MAX
            };
            if e {
                println!("Visits: {}, \t {score}", ((child.visits as f64/self.visits as f64)*100.).round());
            }
            if score >= best_score {
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
    // Returns 1 if the current team wins, 0 if the opponent wins and 0.5 if it's a draw
    fn rollout(&mut self, team: &Team) -> f64 {
        let mut state = self.state.clone();
        while !state.is_over() {
            let random_move = *state.possible_moves()
                .choose(&mut rand::thread_rng())
                .expect("No move found!");
            state.perform(random_move);
        }
        let us = state.fish(team.to_owned());
        let opponent = state.fish(team.opponent());
        if us > opponent {
            return 1.;
        } else if us < opponent {
            return 0.;
        } else {
            return 0.5;
        }
    }

}