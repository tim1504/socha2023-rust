use log::{info, debug};
use rand::seq::SliceRandom;
use std::time;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State, Board, Doubled, Vec2}};

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
            root.mcts(&state.current_team());
        }

        println!("{}", root.depth());
        //root.print_tree(0);
        
        // Select move with highest visits
        let best_move = root.children.iter().max_by_key(|c| c.visits).unwrap().state.last_move().unwrap().clone();
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

    fn depth(&self) -> usize {
        if self.children.is_empty() {
            // If the node has no children, its depth is 1.
            1
        } else {
            // Otherwise, the depth of the node is the maximum depth of its children plus 1.
            self.children.iter().map(|child| child.depth()).max().unwrap() + 1
        }
    }

    fn print_tree(&self, indent: usize) {
        println!("{}Node", " ".repeat(indent * 4));
        for child in self.children.iter() {
            child.print_tree(indent + 1);
        }
    }

    // MCTS algorithm
    fn mcts(&mut self, team: &Team) -> f64 {
        let mut result = 0.;
        if self.visits > 0 && !self.state.is_terminal() {
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child();
            result = selected_child.mcts(team);
        } else {
            result += heuristic(&self.state, team);
        }
        self.visits += 1;
        self.total += if self.state.current_team() == *team {1. - result} else {result};
        return result;
    }
    
    // Selects the best child node based on the UCB1 formula
    fn select_child(&mut self) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut() {
            let score = if child.visits > 0 {
                child.total / child.visits as f64
                + EXPLORATION_CONSTANT * ((self.visits as f64).ln() / (child.visits as f64)).sqrt()
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

fn heuristic(state: &State, team: &Team) -> f64 {
    let us = state.fish(*team) as f64;
    let opponent = state.fish(team.opponent()) as f64;
    if state.is_over() {
        if us > opponent {
            return f64::MAX;
        } else if us < opponent {
            return f64::MIN;
        } else {
            return 0.;
        }
    }
    let mut freedom = 0.;
    let mut starting_points_own = Vec::<usize>::new();
    let mut starting_points_enemy = Vec::<usize>::new();

    for field in state.board().fields() {
        let index = Board::index_for(field.0);
        if field.1.is_occupied() == true {
            if field.1.penguin().as_ref().unwrap().eq(&team){
                starting_points_own.push(index);
            }else{
                starting_points_enemy.push(index);
            }
        }
    }
    let mut starting_points_own_ = Vec::<Vec2<Doubled>>::new();
    let mut starting_points_enemy_ = Vec::<Vec2<Doubled>>::new();

    for field in starting_points_own {
        let coords = Board::coords_for(field);
        let moves = state.board().possible_moves_from(coords);

        for m in moves {
            starting_points_own_.push(m.to());
        }

        freedom = freedom + starting_points_own_.len() as f64;
    }

    for field in starting_points_enemy {
        let coords = Board::coords_for(field);
        let moves = state.board().possible_moves_from(coords);


        for m in moves{
            starting_points_enemy_.push(m.to());
        }

        freedom = freedom - starting_points_enemy_.len() as f64;
    }



    for field in starting_points_own_ {
        let moves = state.board().possible_moves_from(field);
        freedom = freedom + moves.count()as f64;
    }

    for field in starting_points_enemy_ {
        let moves = state.board().possible_moves_from(field);
        freedom = freedom - moves.count()as f64;
    }



    let score_diff = us - opponent;
    

    return score_diff as f64;
}