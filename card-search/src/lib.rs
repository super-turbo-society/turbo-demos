use os::client::watch_file;

turbo::cfg! {r#"
    name = "Card Search"
    version = "1.0.0"
    author = "Turbo"
    description = "Set Up a Counter in Turbo OS"
    [settings]
    resolution = [132, 224]
    [turbo-os]
    api-url = "http://localhost:8000"
"#}

const BOARD_SIZE: u8 = 16;
const CARD_SIZE: (u8, u8) = (16, 24);
const NUM_ROWS: u8 = 4;
const ROW_SPACING: u8 = 12;

//COLORS
const BG_COLOR: u32 = 0x0F172Aff; // Navy background
const CARD_COLOR: u32 = 0x1E3A8Aff; // Deeper blue
const CARD_HIGHLIGHT: u32 = 0x2563EBff; // Brighter blue for hover
const CARD_BORDER: u32 = 0x3B82F6ff; // Light blue border
const CARD_FLIPPED_COLOR: u32 = 0xF0F0F0ff; // Off white

turbo::init! {
    struct GameState {
        board: Option<Board>,
    } = {
        Self {
            board: None,
        }
    }
}

//enum phase
//main_screen
//generating_board
//playing

turbo::go!({
    let mut state = GameState::load();
    clear!(BG_COLOR);
    let m = mouse(0);
    let m_pos = (m.position[0], m.position[1]);
    state.board = watch_file("card_search", "board", &[("stream", "true")])
        .data
        .and_then(|file| Board::try_from_slice(&file.contents).ok());
    let gp = gamepad(0);
    //press Z to make a new board
    if gp.a.just_pressed() {
        os::client::exec("card_search", "generate_board", &[]);
    }

    match &mut state.board {
        None => {}
        Some(b) => {
            for c in &mut b.cards {
                c.draw(m_pos);
                if m.left.just_pressed() {
                    c.on_click(m_pos);
                }
            }
        }
    }
    state.save();
});

#[export_name = "turbo/card_click"]
unsafe extern "C" fn on_card_click() -> usize {
    //read the current board
    let mut board = os::server::read_or!(Board, "board", Board { cards: Vec::new() });
    if board.cards.len() == 0 {
        return os::server::CANCEL;
    } else {
        let num = os::server::command!(u8);
        for c in &mut board.cards {
            if c.id == num && c.is_flipped == false {
                c.is_flipped = true;
                if c.is_crown {
                    //send a message
                }
            }
        }
        //write the file
        let Ok(_) = os::server::write!("board", board) else {
            return os::server::CANCEL;
        };
    }

    return os::server::COMMIT;
}

#[export_name = "turbo/generate_board"]
unsafe extern "C" fn on_generate_board() -> usize {
    let file_path = format!("board");
    let mut board = generate_board();
    let mut num: u32 = os::server::random_number();
    num = num % BOARD_SIZE as u32;
    for c in &mut board.cards {
        if c.id as u32 == num {
            c.is_crown = true;
        }
    }

    let Ok(_) = os::server::write!(&file_path, board) else {
        return os::server::CANCEL;
    };

    os::server::log!("Crown: {}", num);
    return os::server::COMMIT;
}

fn generate_board() -> Board {
    //generate cards
    let mut i = 0;
    let mut cards = Vec::new();
    while i < BOARD_SIZE {
        let card = Card::new(i);
        cards.push(card);
        i += 1;
    }
    //choose a random card to be the crown
    let board = Board { cards };
    board
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
struct Card {
    is_crown: bool,
    id: u8,
    is_flipped: bool,
}

impl Card {
    fn new(id: u8) -> Self {
        Self {
            is_crown: false,
            id,
            is_flipped: false,
        }
    }

    fn get_position_from_id(&self) -> (u32, u32) {
        let margin_x = 14;
        let margin_y = 48; // Increased top padding
        let row = self.id / 4;
        let col = self.id % 4;

        let x = margin_x + (col as u32 * (CARD_SIZE.0 as u32 + ROW_SPACING as u32));
        let y = margin_y + (row as u32 * (CARD_SIZE.1 as u32 + ROW_SPACING as u32));

        (x, y)
    }

    fn on_click(&mut self, pos: (i32, i32)) {
        if self.is_hovered(pos) && self.is_flipped != true {
            let bytes = self.id.to_le_bytes();
            os::client::exec("card_search", "card_click", &bytes);
        }
    }

    fn is_hovered(&self, pos: (i32, i32)) -> bool {
        let (card_x, card_y) = self.get_position_from_id();
        let (px, py) = pos;
        px >= card_x as i32
            && px <= card_x as i32 + CARD_SIZE.0 as i32
            && py >= card_y as i32
            && py <= card_y as i32 + CARD_SIZE.1 as i32
    }

    fn draw(&self, mouse_pos: (i32, i32)) {
        let (x, y) = self.get_position_from_id();

        let mut color = CARD_COLOR;
        if self.is_hovered(mouse_pos) {
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
            border_radius = 2,
            border_color = CARD_BORDER
        );
        if self.is_flipped && self.is_crown {
            sprite!("crown", x = x, y = y);
        }
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
struct Board {
    cards: Vec<Card>,
}
