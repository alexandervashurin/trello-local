use serde::Serialize;
use crate::models::{User, Board, List, Card};

#[derive(Serialize, Default)]
pub struct BoardView {
    pub id: i64,
    pub title: String,
    pub owner_id: i64,
    pub is_shared: bool,
    pub members: Vec<User>,
    pub lists: Vec<ListView>,
}

impl BoardView {
    pub fn from_board(board: Board) -> Self {
        BoardView {
            id: board.id,
            title: board.title,
            owner_id: board.owner_id,
            is_shared: board.is_shared,
            members: Vec::new(),
            lists: Vec::new(),
        }
    }

    pub fn with_members(mut self, members: Vec<User>) -> Self {
        self.members = members;
        self
    }

    pub fn with_lists(mut self, lists: Vec<ListView>) -> Self {
        self.lists = lists;
        self
    }
}

#[derive(Serialize, Default)]
pub struct ListView {
    pub id: i64,
    pub title: String,
    pub cards: Vec<Card>,
}

impl ListView {
    pub fn from_list(list: List) -> Self {
        ListView {
            id: list.id,
            title: list.title,
            cards: Vec::new(),
        }
    }

    pub fn with_cards(mut self, cards: Vec<Card>) -> Self {
        self.cards = cards;
        self
    }
}
