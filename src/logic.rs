use log::{info, debug};
use std::{time, collections::HashSet};

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State, Vec2, Doubled}};

pub struct OwnLogic {
    pub game_tree: Option<Node>,
}

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
        while start.elapsed().as_millis() < TIME_LIMIT && !root.fully_expanded {
            root.mcts(&state.current_team());
        }

        // Select move with highest visits
        let best_move = root.children.iter().max_by_key(|c| ((c.total/c.visits as f64)*1000000.) as i32).unwrap().state.last_move().unwrap().clone();
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
    fully_expanded: bool,
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
            fully_expanded: false,
        }
    }

    // MCTS algorithm
    fn mcts(&mut self, team: &Team) -> (f64,bool) {
        let result;
        if self.visits > 0 && !self.state.is_terminal() {
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child(team);
            let fully_expanded;
            (result, fully_expanded) = selected_child.mcts(team);
            if fully_expanded {self.fully_expanded = self.children.iter().all(|c| c.fully_expanded);}
        } else {
            result = self.rollout(team);
            self.fully_expanded = self.state.is_terminal();
        }
        self.visits += 1;
        self.total += result;
        return (result, self.fully_expanded);
    }
    
    // Selects the best child node based on the UCB1 formula
    fn select_child(&mut self, my_team: &Team) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut().filter(|c| !c.fully_expanded) {
            let score = if child.visits > 0 {
                let winrate = if self.state.current_team() == *my_team {child.total / child.visits as f64} else {1. - child.total / child.visits as f64};
                winrate + EXPLORATION_CONSTANT * ((self.visits as f64).ln() / (child.visits as f64)).sqrt()
            } else {
                f64::MAX
            };
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
    fn rollout(&mut self, my_team: &Team) -> f64 {
        let s = self.state;
        let fishes = |team: Team| -> [u8; 64] { 
            let mut steps: [u8; 64] = [u8::MAX; 64];
            let mut queue: Vec<Vec2<Doubled>> = s.board().penguins().filter(|p| p.1 == team).map(|p| p.0).collect();
            let mut visited: HashSet<Vec2<Doubled>> = HashSet::new();
            let mut step = 2;
            while !queue.is_empty() {
                let mut temp: Vec<Vec2<Doubled>> = Vec::new();
                for f in queue {
                    for m in s.board().possible_moves_from(f) {
                        if !visited.contains(&m.to()) {
                            visited.insert(m.to());
                            steps[(m.to().y*8+m.to().x/2) as usize] = step;
                            temp.push(m.to());
                        }
                    }
                }
                queue = temp;
                step += 1;
            }
            steps
        };
        let steps_us = fishes(*my_team);
        let steps_opponent = fishes(my_team.opponent());
        let mut fish_us = s.fish(*my_team) as f64;
        let mut fish_opponent = s.fish(my_team.opponent()) as f64;
        for (c,f) in s.board().fields() {
            if f.is_empty() {continue;}
            let i = (c.y*8+c.x/2) as usize;
            let fish = f.fish() as f64;
            if steps_us[i] > steps_opponent[i] {fish_opponent += fish;} else if steps_us[i] < steps_opponent[i] {fish_us += fish;}
        }
        ((fish_us)/(fish_us+fish_opponent)-fish_opponent/(fish_us+fish_opponent))/2.+0.5
    }

}