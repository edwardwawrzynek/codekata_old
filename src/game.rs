use core::fmt::Display;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Add;

pub type GamePlayer = u32;

/// Some type of game. It is expected to be turn based, and eventually reach an end state.
pub trait Game: Clone {
    type Move: Serialize;
    type Score: Add + Serialize + Display;
    type State: Serialize + DeserializeOwned;

    /// Check if a game can be created with the number of players
    fn check_num_players(players: usize) -> bool;
    /// Create an instance of the game with the given number of players
    fn new_with_players(players: usize) -> Self;
    /// Create an instance of the game from the given state and number of players
    fn from_state(state: Self::State, players: usize) -> Self;

    /// Get the serializable state of the game
    fn state(&self) -> Self::State;
    /// Return the state of the game (a game starts as Running and becomes Finished)
    fn finished(&self) -> bool;
    /// Check if the game is waiting on a move by the given player
    fn waiting_on(&self, player: GamePlayer) -> bool;
    /// Make a move for the given player. If the move is legal, make it and return true. If not, return false.
    fn make_move(&mut self, player: GamePlayer, move_to_make: Self::Move) -> bool;
    /// Get the score for each player. If scores are not available at the current point in the game, return None.
    fn scores(&self) -> Option<Vec<Self::Score>>;
}
