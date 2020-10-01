use crate::game::{Game, GameOutcome, GamePlayer};
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 19;
const WIN_LEN: usize = 6;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gomoku {
    board: [[i8; BOARD_SIZE]; BOARD_SIZE],
    turn: i8,
}

#[derive(FromForm)]
pub struct Move {
    x: i32,
    y: i32,
}

impl Gomoku {
    fn full(&self) -> bool {
        for row in &self.board {
            for cell in row {
                if *cell == -1 {
                    return false;
                }
            }
        }

        true
    }

    fn check_row(&self, player: GamePlayer, row_i: usize) -> bool {
        let mut count = 0;
        let row = &self.board[row_i];
        for cell in row {
            if *cell == player as i8 {
                count += 1;
            } else {
                count = 0;
            }
            if count >= WIN_LEN {
                return true;
            }
        }

        false
    }

    fn check_col(&self, player: GamePlayer, col_i: usize) -> bool {
        let mut count = 0;
        for row in &self.board {
            let cell = row[col_i];
            if cell == player as i8 {
                count += 1;
            } else {
                count = 0;
            }
            if count >= WIN_LEN {
                return true;
            }
        }

        false
    }

    fn check_diags(&self, player: GamePlayer) -> bool {
        // up left diagonal
        for i in 0..(BOARD_SIZE*2 -1) {
            let mut count = 0;
            let j = if i < BOARD_SIZE { 0 } else { i - BOARD_SIZE + 1 };
            let z = if i < BOARD_SIZE { i + 1} else { BOARD_SIZE };
            for y in j..z {
                let x = i - y;

                if self.board[x][y] == player as i8 {
                    count += 1;
                } else {
                    count = 0;
                }

                if count >= WIN_LEN {
                    return true;
                }
            }
        };
        // down right diagonal
        for i in 0..(BOARD_SIZE*2 - 1) {
            let mut count = 0;
            let j = if i < BOARD_SIZE { BOARD_SIZE - i - 1 } else { 0 };
            let z = if i < BOARD_SIZE { BOARD_SIZE } else { BOARD_SIZE*2 - i - 1 };
            for y in j..z {
                let x = i + y + 1 - BOARD_SIZE;

                if self.board[x][y] == player as i8 {
                    count += 1;
                } else {
                    count = 0;
                }

                if count >= WIN_LEN {
                    return true;
                }
            }
        }
        false
    }

    fn check_win(&self, player: GamePlayer) -> bool {
        if player != 0 && player != 1 {
            false
        } else {
            for i in 0..BOARD_SIZE {
                if self.check_col(player, i) || self.check_row(player, i) {
                    return true;
                }
            }

            self.check_diags(player)
        }
    }
}

impl Game for Gomoku {
    type Move = Move;
    type Score = f64;
    type State = Self;

    fn check_num_players(players: usize) -> bool {
        players == 2
    }

    fn new_with_players(players: usize) -> Self {
        assert_eq!(players, 2);
        Gomoku {
            board: [[-1; BOARD_SIZE]; BOARD_SIZE],
            turn: 0,
        }
    }

    fn from_state(state: Self::State, _players: usize) -> Self {
        state
    }

    fn state(&self) -> Self::State {
        self.clone()
    }

    fn finished(&self) -> bool {
        self.full() || self.check_win(0) || self.check_win(1)
    }

    fn waiting_on(&self, player: u32) -> bool {
        player as i8 == self.turn
    }

    fn make_move(&mut self, player: u32, move_to_make: &Self::Move) -> bool {
        if player as i8 != self.turn {
            return false;
        } else if move_to_make.x >= BOARD_SIZE as i32
            || move_to_make.y >= BOARD_SIZE as i32
            || move_to_make.x < 0
            || move_to_make.y < 0
        {
            return false;
        } else {
            if self.board[move_to_make.x as usize][move_to_make.y as usize] != -1 {
                return false;
            }
            self.board[move_to_make.x as usize][move_to_make.y as usize] = player as i8;
        }

        self.turn = if self.turn == 0 {
            1
        } else if self.turn == 1 {
            0
        } else {
            -1
        };

        true
    }

    fn scores(&self) -> Option<Vec<Self::Score>> {
        if self.check_win(0) {
            Some(vec![1.0, 0.0])
        } else if self.check_win(1) {
            Some(vec![0.0, 1.0])
        } else if self.full() {
            Some(vec![0.5, 0.5])
        } else {
            None
        }
    }

    fn outcome(&self) -> GameOutcome {
        if self.check_win(0) {
            GameOutcome::Win(0)
        } else if self.check_win(1) {
            GameOutcome::Win(1)
        } else if self.full() {
            GameOutcome::Tie
        } else {
            GameOutcome::None
        }
    }
}
