use ic_cdk::export::candid::CandidType;
use ic_cdk::storage;
use ic_cdk_macros::*;
use pleco::tools::Searcher;
use serde::Serialize;
use std::collections::BTreeMap;

type GameStore = BTreeMap<String, GameInternal>;

#[derive(Clone, Debug, Default, CandidType, Serialize)]
pub struct Game {
    pub fen: String,
}

pub struct GameInternal {
    pub board: pleco::Board,
}

#[update]
fn new(name: String, white: bool) -> () {
    let game_store = storage::get_mut::<GameStore>();
    game_store.insert(
        name.clone(),
        GameInternal {
            board: pleco::Board::start_pos(),
        },
    );

    // If the user is playing black;
    if !white {
        ai_move(name);
    }
}

#[update(name = "move")]
fn uci_move(name: String, m: String) -> bool {
    let game_store = storage::get_mut::<GameStore>();

    if !game_store.contains_key(&name) {
        panic!("Game {} does not exist.", name);
    }

    let game = game_store
        .get_mut(&name)
        .expect(&format!("No game named {}", name));

    // If the move is valid, also apply the next move using AI.
    if game.board.apply_uci_move(&m) {
        ai_move(name);
        true
    } else {
        false
    }
}

#[update(name = "aiMove")]
fn ai_move(name: String) -> () {
    let game_store = storage::get_mut::<GameStore>();

    if !game_store.contains_key(&name) {
        panic!("Game {} does not exist.", name);
    }

    let game = game_store
        .get_mut(&name)
        .expect(&format!("No game named {}", name));

    let b = game.board.shallow_clone();
    let m = pleco::bots::MiniMaxSearcher::best_move(b, 3);

    game.board.apply_move(m);
}

#[query(name = "getFen")]
fn get_fen(name: String) -> Option<String> {
    let game_store = storage::get_mut::<GameStore>();

    game_store
        .get(&name)
        .and_then(|game| Some(game.board.fen()))
}
