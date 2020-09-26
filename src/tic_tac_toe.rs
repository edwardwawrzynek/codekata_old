use crate::{Game, GamePlayer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TicTacToe {
    board: [[i8; 3]; 3],
    turn: i8,
}

#[derive(Serialize, Deserialize)]
struct Move {
    x: i32,
    y: i32,
}

impl TicTacToe {
    fn full(&self) -> bool {
        for row in &self.board {
            for cell in row {
                if *cell != -1 {
                    return false;
                }
            }
        }

        true
    }

    fn check_win(&self, player: GamePlayer) -> bool {
        if player != 0 && player != 1 {
            false
        } else {
            true
        }
    }
}

impl Game for TicTacToe {
    type Move = Move;
    type Score = f64;
    type State = Self;

    fn check_num_players(players: usize) -> bool {
        players == 2
    }

    fn new_with_players(players: usize) -> Self {
        assert_eq!(players, 2);
        TicTacToe {
            board: [[-1; 3]; 3],
            turn: 0,
        }
    }

    fn from_state(state: Self::State, players: usize) -> Self {
        state
    }

    fn state(&self) -> Self::State {
        self.clone()
    }

    fn finished(&self) -> bool {
        unimplemented!()
    }

    fn waiting_on(&self, player: u32) -> bool {
        unimplemented!()
    }

    fn make_move(&mut self, player: u32, move_to_make: Self::Move) -> bool {
        unimplemented!()
    }

    fn scores(&self) -> Option<Vec<Self::Score>> {
        unimplemented!()
    }
}
