use log::{info, debug};
use rand::seq::SliceRandom;
use std::{time::{self}, collections::{HashMap, BinaryHeap}, cmp::Ordering};
use colored::*;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State, Board}};

pub struct OwnLogic {
    pub game_tree: Option<Node>,
}

//pub const SIMULATIONS_PER_ROLLOUT: u32 = 100;
pub const TIME_LIMIT: u128 = 1000;
pub const EXPLORATION_CONSTANT: f64 = 1.41;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("{}", "Requested Move".white());

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
        info!("{}{}", "Score of choosen move: ".green() ,format!("{:.2}",best_node.total));
        info!("{}", "Children:".blue());
        for n in &root.children {
            print!("{}{}{}", "[".blue(), format!("{:.2}",n.total), "]".blue());
        }
        println!("");

        
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


struct Edge {
    to: usize,
    cost: i64,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct GNode {
    index: usize,
    distance: i64,
}



impl Ord for GNode {
    fn cmp(&self, other: &GNode) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for GNode {
    fn partial_cmp(&self, other: &GNode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Graph {
    adj_list: HashMap<usize, Vec<Edge>>,
}

impl Graph {
    fn new() -> Self {
        Self {
            adj_list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, u: usize, v: usize, w: i64) {
        self.adj_list.entry(u).or_insert(vec![]).push(Edge {
            to: v,
            cost: w,
        });
 
    }

    fn dijkstra(&self, start: usize) -> Vec<i64> {
        let mut dist = vec![i64::MAX; self.adj_list.len()];
        let mut heap = BinaryHeap::new();

        dist[start] = 0;
        heap.push(GNode {
            index: start,
            distance: 0,
        });

        while let Some(GNode { index: u, distance: _ }) = heap.pop() {
            for Edge { to, cost } in self.adj_list.get(&u).unwrap_or(&vec![]) {
                let alt = dist[u] + cost;
                let to_dist = dist.get_mut(*to);
                if let Some(to_dist) = to_dist {
                    if alt < *to_dist {
                        *to_dist = alt;
                        heap.push(GNode {
                            index: *to,
                            distance: -alt,
                        });
                    }
                }
            }
        }

        dist
    }
    

}



fn heuristic(state: &State, my_team: &Team) -> f64{
    let mut graph = Graph::new();
    let mut starting_points_own = Vec::<usize>::new();
    let mut starting_points_enemy = Vec::<usize>::new();
    let start = std::time::Instant::now();
    
    //Creating the Graph
    for field in state.board().fields() {
        let index = Board::index_for(field.0);
        if field.1.is_occupied() == true {
            if field.1.penguin().as_ref().unwrap().eq(&my_team){
                starting_points_own.push(index);
            }else{
                starting_points_enemy.push(index);
            }
        }

        let mut moves = state.board().possible_moves_from(field.0);
        if moves.count() as i32 == 0 {
            graph.add_edge(index, index, 0);
        }
        moves = state.board().possible_moves_from(field.0);
        for neighbour in moves{
            let neighbour_coords = neighbour.to();
            let neighbour_index = Board::index_for(neighbour_coords);

            graph.add_edge(index, neighbour_index, 1);
        }
    }
    //println!("Graph has been created.");
    //println!("{}", graph.adj_list.len());
    //print_adj_list(&graph);

    let mut own_1 = vec![std::i64::MAX; 64];
    let mut own_2 = vec![std::i64::MAX; 64];
    let mut own_3 = vec![std::i64::MAX; 64];
    let mut own_4 = vec![std::i64::MAX; 64];
    if starting_points_own.len() > 0{
        own_1 = graph.dijkstra(starting_points_own[0]);
        if starting_points_own.len() > 1{
            own_2 = graph.dijkstra(starting_points_own[1]);
            if starting_points_own.len() > 2{
                own_3 = graph.dijkstra(starting_points_own[2]);
                if starting_points_own.len() > 3{
                    own_4 = graph.dijkstra(starting_points_own[3]);
                }
            }
        }
    }
    let own = find_min_values(&own_1, &own_2, &own_3, &own_4);
    
    let mut enemy_1 = vec![std::i64::MAX; 64];
    let mut enemy_2 = vec![std::i64::MAX; 64];
    let mut enemy_3 = vec![std::i64::MAX; 64];
    let mut enemy_4 = vec![std::i64::MAX; 64];
    if starting_points_enemy.len() > 0{
        enemy_1 = graph.dijkstra(starting_points_enemy[0]);
        if starting_points_enemy.len() > 1{
            enemy_2 = graph.dijkstra(starting_points_enemy[1]);
            if starting_points_enemy.len() > 2{
                enemy_3 = graph.dijkstra(starting_points_enemy[2]);
                if starting_points_enemy.len() > 3{
                    enemy_4 = graph.dijkstra(starting_points_enemy[3]);
                }
            }
        }
    }
    //println!("Dijkstra succesfull");
    //println!("{:?}", starting_points_enemy);

    let enemy =find_min_values(&enemy_1, &enemy_2, &enemy_3, &enemy_4);

    let lists = compare_lists(&own, &enemy);
    
    let mut value_own = 0.0;
    for n in lists.0{
        if own[n] != 0{
            value_own += state.board().get(Board::coords_for(n)).expect("REASON").fish() as f64;
        }
        
    }

    let mut value_enemy = 0.0;
    for n in lists.1{
        if enemy[n] != 0{
            value_enemy += state.board().get(Board::coords_for(n)).expect("REASON").fish() as f64;
        }
        
    }
    

    let mut value = value_own - value_enemy;
    //print!("{:?}", enemy);
    //let elapsed = start.elapsed();
    let us = state.fish(my_team.to_owned());
    let opponent = state.fish(my_team.opponent());
    value += us as f64 - opponent as f64;
    /*/
    println!("Shortest distances: {:?}", own);
    println!("{}", value);
    println!("Elapsed time: {} ms", elapsed.as_micros());
    */
    return value;
}

fn qubic(x: f64, n: i32) -> f64{
    let mut result = x;
    for _i in 1..n{
        result *= x;
    }
    return result; 
}

fn find_min_values(values_1: &[i64], values_2: &[i64], values_3: &[i64], values_4: &[i64]) -> Vec<i64> {
    let mut min_values = Vec::with_capacity(values_1.len());
    for i in 0..values_1.len() {
        let min_value = std::cmp::min(
            std::cmp::min(values_1[i], values_2[i]),
            std::cmp::min(values_3[i], values_4[i]),
        );
        min_values.push(min_value);
    }
    min_values
}

fn compare_lists(list_a: &[i64], list_b: &[i64]) -> (Vec<usize>, Vec<usize>) {
    let mut a_indices = Vec::new();
    let mut b_indices = Vec::new();
    for i in 0..list_a.len().min(list_b.len()) {
        if list_a[i] < list_b[i] {
            a_indices.push(i);
        } else if list_a[i] > list_b[i] {
            b_indices.push(i);
        }
    }
    (a_indices, b_indices)
}
