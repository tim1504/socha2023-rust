use log::{info, debug};
use rand::seq::SliceRandom;

use socha_client_2023::{client::GameClientDelegate, game::{Move, Team, State}};

/// An empty game logic structure that implements the client delegate trait
/// and thus is responsible e.g. for picking a move when requested.
pub struct OwnLogic;

impl GameClientDelegate for OwnLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {

        info!("Requested move");
        let simulations = 10000/state.possible_moves().len();

        let mut best_move: Option<Move> = None;
        let mut best_score = i32::MIN;

        for possible_move in state.possible_moves() {

            let score = {
                let mut sum = 0;
                for _i in 0..simulations {
                    sum += random_simulation(state.clone(), &state.current_team())
                }
                sum
            };

            if score > best_score {
                best_move = Some(possible_move);
                best_score = score;
            }

        }

        if best_move.is_none() {
            panic!("No best move could be found!");
        }

        fn random_simulation( mut gs: State, team: &Team) -> i32 {
            while !gs.is_over() {
                let random_move = *gs.possible_moves()
                    .choose(&mut rand::thread_rng())
                    .expect("No move found!");

                gs.perform(random_move);
            }
            if gs.winner().is_none() {
                return 0
            }
            if gs.winner().unwrap().eq(team) {
                return 1
            }
            -1
        }

        best_move.unwrap()

    }

    fn on_game_end(&mut self, _result: &socha_client_2023::protocol::GameResult) {
        //do sth
    }

    fn on_update_state(&mut self, state: &State) {
        debug!("Board:\n{}", state.board());
    }
}
