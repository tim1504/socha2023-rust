use log::{info, debug};
use rand::seq::SliceRandom;
use std::time;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

pub struct OwnLogic;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("Requested move");

        //Create root node
        let mut root = Node::new(state.clone());
        //Expand root node
        root.expand();

        //Runs for 2 seconds
        let start = time::Instant::now();

        while start.elapsed().as_millis() < 1800 {
            root.mcts();
        }

        //Return best move
        root.select_child().state.last_move().unwrap()

    }

    fn on_game_end(&mut self, _result: &socha_client_2023::protocol::GameResult) {}

    fn on_update_state(&mut self, state: &State) { debug!("Board:\n{}", state.board()) }
    
}

struct Node {
    state: State,
    children: Vec<Node>,
    visits: u32,
    total: i32,
}

impl Node {

    fn new(state: State) -> Self {
        Node {
            state,
            children: Vec::new(),
            visits: 0,
            total: 0,
        }
    }

    fn mcts(&mut self) -> i32 {
        let mut result = 0;
        if self.visits > 0 && !self.state.is_over(){
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child();
            result = selected_child.mcts();
        } else {
            for _i in 0..100 {
                result += self.rollout(self.state.current_team());
            }
        }
        self.visits += 1;
        self.total += result;
        return result;
    }
    
    fn select_child(&mut self) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut() {
            let mut score = (child.total as f64 / child.visits as f64) + (2.0*(self.visits as f64).ln() / child.visits as f64).sqrt();
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

    fn expand(&mut self) {
        for m in self.state.possible_moves() {
            let mut next_state = self.state.clone();
            next_state.perform(m);
            self.children.push(Node::new(next_state));
        }
    }

    fn rollout(&mut self, team: Team) -> i32 {
        let mut state = self.state.clone();
        while !state.is_over() {
            let random_move = *state.possible_moves()
                .choose(&mut rand::thread_rng())
                .expect("No move found!");
            state.perform(random_move);
        }
        (state.fish(team) - state.fish(team.opponent())) as i32
    }

}