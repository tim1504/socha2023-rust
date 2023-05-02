use log::{info, debug};
use rand::seq::SliceRandom;
use std::time;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

pub struct OwnLogic {
    pub game_tree: Option<Node>,
}

pub const SIMULATIONS_PER_ROLLOUT: u32 = 100;
pub const TIME_LIMIT: u128 = 1900;
pub const EXPLORATION_CONSTANT: f64 = 1.41;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("Requested move");

        let start = time::Instant::now();

        // Check if the game tree contains the current state
        let mut alpha_root = None;
        if self.game_tree.is_some() {
            let game_tree = self.game_tree.take().unwrap();
            for node in game_tree.children.iter().flat_map(|n| n.children.iter().chain(Some(n))) {
                if node.state == *state {
                    alpha_root = Some(node.to_owned());
                    break;
                }
            }
        }
        // Else create a new game tree
        let mut alpha_root: Node = alpha_root.unwrap_or(Node::new(state.clone()));

        let root = &mut alpha_root;
        if root.children.is_empty() {
            root.expand();
        }
        
        // Run MCTS algorithm for about 2 seconds
        while start.elapsed().as_millis() < TIME_LIMIT {
            root.mcts(&state.current_team(), 1);
        }

        // Select move with highest visits
        let best_node: Node = root.children.iter().max_by_key(|c| c.visits).unwrap().clone();
        println!("{}", best_node.total / best_node.visits as f64);
        let best_move = best_node.state.last_move().unwrap().clone();
        // Save the game tree for the next move
        self.game_tree = Some(alpha_root);
        best_move

    }

    fn on_game_end(&mut self, _result: &socha_client_2023::protocol::GameResult) {}

    fn on_update_state(&mut self, state: &State) { debug!("Board:\n{}", state.board()) }
    
}

// Node struct for MCTS algorithm
#[derive(Clone)]
pub struct Node {
    state: State,
    children: Vec<Node>,
    visits: u32,
    total: f64,
}

// Node methods
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
    fn mcts(&mut self, team: &Team, depth: i32) -> f64 {
        //println!("{}", self.state.current_team().index());
        //println!("{}", depth);
        let mut result = 0.;
        if self.visits > 0 && !self.state.is_terminal() {
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child(team);
            result = selected_child.mcts(team, depth + 1);
        } else {
            for _i in 0..SIMULATIONS_PER_ROLLOUT {
                result += self.rollout(team);
            }
            result /= SIMULATIONS_PER_ROLLOUT as f64;
        }
        self.visits += 1;
        self.total += result;
        return result;
    }
    
    // Selects the best child node based on the UCB1 formula
    fn select_child(&mut self, team: &Team) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut() {
            let score = if child.visits > 0 {
                if self.state.current_team().eq(team){
                    child.total / child.visits as f64
                    + EXPLORATION_CONSTANT * ((self.visits as f64).ln() / (child.visits as f64)).sqrt()
                }else{
                    1.0 - (child.total / child.visits as f64)
                    + EXPLORATION_CONSTANT * ((self.visits as f64).ln() / (child.visits as f64)).sqrt()
                }
                
            } else {
                f64::MAX
            };
            if self.state.current_team().eq(team){
                //println!("we {}", score);
            }else{
                //println!("enemy {}", score);
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