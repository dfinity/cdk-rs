use candid_derive::CandidType;
use ic_cdk::storage;
use ic_cdk_macros::*;
use pleco::tools::Searcher;
use serde::Serialize;
use std::collections::BTreeMap;

type GameStore = BTreeMap<String, GameInternal>;

#[repr(u8)]
#[derive(Clone, Debug, CandidType, Serialize)]
pub enum Piece {
    None,
    BlackPawn,
    BlackRook,
    BlackKnight,
    BlackBishop,
    BlackQueen,
    BlackKing,
    WhitePawn,
    WhiteRook,
    WhiteKnight,
    WhiteBishop,
    WhiteQueen,
    WhiteKing,
}

impl From<pleco::Piece> for Piece {
    fn from(p: pleco::Piece) -> Self {
        match p {
            pleco::Piece::None => Piece::None,
            pleco::Piece::BlackPawn => Piece::BlackPawn,
            pleco::Piece::BlackRook => Piece::BlackRook,
            pleco::Piece::BlackKnight => Piece::BlackKnight,
            pleco::Piece::BlackBishop => Piece::BlackBishop,
            pleco::Piece::BlackQueen => Piece::BlackQueen,
            pleco::Piece::BlackKing => Piece::BlackKing,
            pleco::Piece::WhitePawn => Piece::WhitePawn,
            pleco::Piece::WhiteRook => Piece::WhiteRook,
            pleco::Piece::WhiteKnight => Piece::WhiteKnight,
            pleco::Piece::WhiteBishop => Piece::WhiteBishop,
            pleco::Piece::WhiteQueen => Piece::WhiteQueen,
            pleco::Piece::WhiteKing => Piece::WhiteKing,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, CandidType, Serialize)]
pub enum Player {
    Black,
    White,
}

#[derive(Clone, Debug, Default, CandidType, Serialize)]
pub struct Game {
    pub name: String,
    pub fen: String,
}

pub struct GameInternal {
    pub board: pleco::Board,
}

#[update]
fn new(name: String) -> () {
    let game_store = storage::get::<GameStore>();
    game_store.insert(
        name,
        GameInternal {
            board: pleco::Board::start_pos(),
        },
    );
}

#[update(name = "newFromFen")]
fn new_from_fen(name: String, fen: String) -> () {
    let game_store = storage::get::<GameStore>();
    game_store.insert(
        name,
        GameInternal {
            board: pleco::Board::from_fen(&fen).unwrap(),
        },
    );
}

#[update(name = "uci")]
fn uci_move(name: String, m: String) -> bool {
    let game_store = storage::get::<GameStore>();

    let game = game_store
        .get_mut(&name)
        .expect(&format!("No game named {}", name));

    game.board.apply_uci_move(&m)
}

#[update(name = "generateMove")]
fn generate_move(name: String) -> () {
    let game_store = storage::get::<GameStore>();

    let game = game_store
        .get_mut(&name)
        .expect(&format!("No game named {}", name));

    let b = game.board.shallow_clone();
    let m = pleco::bots::MiniMaxSearcher::best_move(b, 3);

    game.board.apply_move(m);
}

#[query(name = "getBoard")]
fn get_board(name: String) -> Option<Vec<Vec<Piece>>> {
    let game_store = storage::get::<GameStore>();

    game_store.get(&name).and_then(|game| {
        let mut board = Vec::new();
        for _ in 0..8 {
            let mut file = Vec::new();
            for _ in 0..8 {
                file.push(Piece::None);
            }

            board.push(file);
        }

        for (sq, piece) in game.board.get_piece_locations() {
            board[sq.file() as usize][sq.rank() as usize] = piece.into();
        }

        Some(board)
    })
}

#[query(name = "getState")]
fn get_state(name: String) -> Option<Game> {
    let game_store = storage::get::<GameStore>();

    game_store.get(&name).and_then(|game| {
        Some(Game {
            name,
            fen: game.board.fen(),
        })
    })
}
