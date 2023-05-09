use log::{info, debug};
use rand::seq::SliceRandom;
use std::{time, collections::HashSet};

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State, Vec2, Doubled}};

pub struct OwnLogic {
    pub game_tree: Option<Node>,
}

pub const TIME_LIMIT: u128 = 1800;
pub const EXPLORATION_CONSTANT: f64 = 1.41;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("{}", "Requested Move");

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
        let best_node: Node = root.children.iter().max_by_key(|c| (c.total * 100.0) as i32).unwrap().clone();
        info!("{}{}{}{}", "Current Score: ", format!("{:.2}",heuristic(state,&_my_team)), " -> Score of choosen move: " ,format!("{:.2}",best_node.total));
        info!("Maximum depth of the Game Tree: {}", max_depth(&root));

        
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
        let mut result;
        if self.visits > 0 && !self.state.is_terminal() {
            if self.children.is_empty() {
                self.expand();
            }
            let selected_child = self.select_child(team);
            result = selected_child.mcts(team, depth + 1);
            //println!("mcts performed at depth: {}", depth);
        } else {
            result = heuristic(&self.state, team);
            //println!("rollout performed at depth: {}{}{}", depth, " with the result: ", result);
        }
        self.visits += 1;
        self.total = result;
        if self.state.current_team().eq(team){
            for n in &self.children{
                if n.total > result && result <= 100.0 && n.visits > 0{
                    result = n.total;
                }
            }
            /*/
            println!("we");
            println!("{}", result);
            println!(" {}", self.children.len());
            for n in &self.children {
                print!("[{}{}", n.total, "]");
            }
            println!("");
            */
        }else{
            for n in &self.children{
                if n.total < result && result <= 100.0 && n.visits > 0{
                    result = n.total;
                }
            }
            /*
            println!("enemy");
            println!("{}", result);
            println!(" {}", self.children.len());
            for n in &self.children {
                print!("[{}{}", n.total, "]");
            }
            println!("");
            */
        }
       
        return result;
    }
    
    // Selects the best child node based on the UCB1 formula
    fn select_child(&mut self, team: &Team) -> &mut Node {
        let mut best_score = f64::MIN;
        let mut best_child = None;
        for child in self.children.iter_mut() {
            let score = if child.visits > 0 {
                if self.state.current_team().eq(team){
                    child.total as f64
                    + EXPLORATION_CONSTANT * ((self.visits as f64).ln() / (child.visits as f64)).sqrt()
                }else{
                    -1.0 * (child.total as f64)
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

fn max_depth(root: &Node) -> u32 {
    if root.children.is_empty() {
        // Base case: the root has no children, so its depth is 1.
        return 1;
    } else {
        // Recursive case: the depth of the root is the maximum depth of its children plus one.
        let child_depths = root.children.iter().map(|child| max_depth(child));
        return child_depths.max().unwrap() + 1;
    }
}

fn heuristic(s: &State, my_team: &Team) -> f64 {
    let fishes = |team: Team| -> [u8; 64] { 
        let mut steps: [u8; 64] = [u8::MAX; 64];
        let mut queue: Vec<Vec2<Doubled>> = s.board().penguins().filter(|p| p.1 == team).map(|p| p.0).collect();
        let mut visited: HashSet<Vec2<Doubled>> = HashSet::new();
        let mut step = 1;
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
    let fishes_us = fishes(*my_team);
    let fishes_opponent = fishes(my_team.opponent());
    let mut score = (s.fish(*my_team) as i32) - (s.fish(my_team.opponent()) as i32);
    for (c,f) in s.board().fields() {
        if f.is_empty() {continue;}
        let i = (c.y*8+c.x/2) as usize;
        let fish = f.fish() as i32;
        score += if fishes_us[i] > fishes_opponent[i] {-fish} else if fishes_us[i] < fishes_opponent[i] {fish} else {0};
    }
    score as f64
}