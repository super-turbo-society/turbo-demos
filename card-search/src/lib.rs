use turbo::*;

const BOARD_SIZE: u8 = 16;
const CARD_SIZE: (u8, u8) = (16, 24);
const ROW_SPACING: u8 = 12;
const CARD_COLOR: u32 = 0x1E3A8Aff;
const CARD_HIGHLIGHT: u32 = 0x2563EBff;
const CARD_FLIPPED_COLOR: u32 = 0xF0F0F0ff;

#[turbo::game]
struct GameState {
    board: Option<card_search::Board>,
}

impl GameState {
    fn new() -> Self {
        Self { board: None }
    }

    fn update(&mut self) {
        draw_checkerboard();

        let pointer = pointer::screen();
        let (x, y) = pointer.xy();

        if gamepad::get(0).a.just_pressed() {
            camera::reset();
        }

        self.board = card_search::Board::watch("board").parse();

        if let Some(b) = &mut self.board {
            let crown_found = is_crown_found(b);
            for card in &mut b.cards {
                card.draw((x, y));
                if pointer.just_pressed() && !crown_found {
                    card.on_click((x, y));
                }
            }
        }

        if let Some(event) = os::client::watch_events("card_search", Some("alert")).data {
            let duration = 10_000;
            let millis_since = time::now() - event.created_at as u64 * 1000;
            if millis_since < duration {
                if let Ok(msg) = std::str::from_utf8(&event.data) {
                    let txt = format!("User {}", msg);
                    centered_text(&txt, 200, CARD_FLIPPED_COLOR);
                    centered_text("Found the crown", 210, CARD_FLIPPED_COLOR);
                }
            }
        }

        if self.board.is_none() || self.board.as_ref().map_or(false, is_crown_found) {
            centered_text("Press Z", 10, CARD_FLIPPED_COLOR);
            centered_text("To Start New Game", 20, CARD_FLIPPED_COLOR);
            if gamepad::get(0).a.just_pressed() {
                card_search::GenerateBoard.exec();
            }
        } else {
            centered_text("Find the Crown!", 10, CARD_FLIPPED_COLOR);
        }
    }
}

fn is_crown_found(board: &card_search::Board) -> bool {
    board.cards.iter().any(|c| c.is_crown && c.is_flipped)
}

fn generate_board() -> card_search::Board {
    let mut cards = vec![];
    for i in 0..BOARD_SIZE {
        cards.push(card_search::Card::new(i));
    }
    card_search::Board { cards }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[..max_len].to_string()
    } else {
        s.to_string()
    }
}

fn draw_checkerboard() {
    let width = 132;
    let height = 224;
    let cols = 8;
    let rows = 14;
    let tile_width = (width + cols - 1) / cols;
    let tile_height = (height + rows - 1) / rows;
    let dark = 0x1A1A1Aff;
    let light = 0x202020ff;

    for row in 0..rows {
        for col in 0..cols {
            let x = col * tile_width;
            let y = row * tile_height;
            let color = if (row + col) % 2 == 0 { dark } else { light };
            rect!(
                x = x,
                y = y,
                w = tile_width as u8,
                h = tile_height as u8,
                color = color
            );
        }
    }
}

fn centered_text(text: &str, y: i32, color: u32) {
    let x = centered_pos(text, 5, 132);
    text!(text, x = x, y = y, color = color);
}

fn centered_pos(text: &str, char_width: i32, full_width: i32) -> i32 {
    (full_width - (text.len() as i32 * char_width)) / 2
}

pub mod card_search {
    use super::*;

    #[turbo::serialize]
    pub struct Card {
        pub is_crown: bool,
        pub id: u8,
        pub is_flipped: bool,
    }

    impl Card {
        pub fn new(id: u8) -> Self {
            Self {
                is_crown: false,
                id,
                is_flipped: false,
            }
        }

        pub fn get_position_from_id(&self) -> (u32, u32) {
            let margin_x = 14;
            let margin_y = 48;
            let row = self.id / 4;
            let col = self.id % 4;
            let x = margin_x + (col as u32 * (CARD_SIZE.0 as u32 + ROW_SPACING as u32));
            let y = margin_y + (row as u32 * (CARD_SIZE.1 as u32 + ROW_SPACING as u32));
            (x, y)
        }

        pub fn is_hovered(&self, pos: (i32, i32)) -> bool {
            let (x, y) = self.get_position_from_id();
            let (px, py) = pos;
            px >= x as i32
                && px <= x as i32 + CARD_SIZE.0 as i32
                && py >= y as i32
                && py <= y as i32 + CARD_SIZE.1 as i32
        }

        pub fn draw(&self, pointer: (i32, i32)) {
            let (x, y) = self.get_position_from_id();
            let mut color = CARD_COLOR;
            if self.is_hovered(pointer) {
                color = CARD_HIGHLIGHT;
            }
            if self.is_flipped {
                color = CARD_FLIPPED_COLOR;
            }
            rect!(
                x = x,
                y = y,
                w = CARD_SIZE.0,
                h = CARD_SIZE.1,
                color = color,
                border_radius = 2
            );
            if self.is_flipped && self.is_crown {
                sprite!("crown", x = x, y = y);
            }
        }

        pub fn on_click(&mut self, pos: (i32, i32)) {
            if self.is_hovered(pos) && !self.is_flipped {
                CardClick(self.id).exec();
            }
        }
    }

    #[turbo::os::document(program = "card_search")]
    pub struct Board {
        pub cards: Vec<Card>,
    }

    #[turbo::os::command(program = "card_search", name = "card_click")]
    pub struct CardClick(pub u8);
    impl CommandHandler for CardClick {
        fn run(&mut self, user_id: &str) -> Result<(), std::io::Error> {
            let mut board = os::server::fs::read("board").unwrap_or(Board { cards: vec![] });
            for card in &mut board.cards {
                if card.id == self.0 && !card.is_flipped {
                    card.is_flipped = true;
                    if card.is_crown {
                        let short = super::truncate_string(user_id, 8);
                        os::server::alert!("{}", short);
                    }
                }
            }
            os::server::fs::write("board", &board)?;
            Ok(())
        }
    }

    #[turbo::os::command(program = "card_search", name = "generate_board")]
    pub struct GenerateBoard;
    impl CommandHandler for GenerateBoard {
        fn run(&mut self, _user_id: &str) -> Result<(), std::io::Error> {
            let mut board = super::generate_board();
            let crown_id = random::between(0, BOARD_SIZE);
            for card in &mut board.cards {
                if card.id == crown_id {
                    card.is_crown = true;
                }
            }
            os::server::fs::write("board", &board)?;
            log!("Crown: {}", crown_id);
            Ok(())
        }
    }
}
