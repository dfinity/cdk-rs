use ic_cdk::export::candid::CandidType;
use ic_cdk_macros::*;
use pleco::tools::Searcher;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;

type GameStore = BTreeMap<String, GameInternal>;

#[derive(Clone, Debug, Default, CandidType, Serialize)]
pub struct Game {
    pub fen: String,
}

pub struct GameInternal {
    pub board: pleco::Board,
}

thread_local! {
    static STORE: RefCell<GameStore> = RefCell::default();
}

#[update]
fn new(name: String, white: bool) -> () {
    STORE.with(|game_store| {
        game_store.borrow_mut().insert(
            name.clone(),
            GameInternal {
                board: pleco::Board::start_pos(),
            },
        );
    });

    // If the user is playing black;
    if !white {
        ai_move(name);
    }
}

#[update(name = "move")]
fn uci_move(name: String, m: String) -> bool {
    let should_move = STORE.with(|game_store| {
        let mut game_store = game_store.borrow_mut();
        let game = game_store
            .get_mut(&name)
            .unwrap_or_else(|| panic!("Game {} does not exist.", name));

        // If the move is valid, also apply the next move using AI.
        game.board.apply_uci_move(&m)
    });
    if should_move {
        ai_move(name);
    }
    should_move
}

#[update(name = "aiMove")]
fn ai_move(name: String) -> () {
    STORE.with(|game_store| {
        let mut game_store = game_store.borrow_mut();
        let game = game_store
            .get_mut(&name)
            .unwrap_or_else(|| panic!("Game {} does not exist.", name));

        let b = game.board.shallow_clone();
        let m = pleco::bots::MiniMaxSearcher::best_move(b, 3);

        game.board.apply_move(m);
    });
}

#[query(name = "getFen")]
fn get_fen(name: String) -> Option<String> {
    STORE.with(|game_store| {
        game_store
            .borrow()
            .get(&name)
            .and_then(|game| Some(game.board.fen()))
    })
}
