use os::client::watch_file;

const BOARD_SIZE: u8 = 16;
const CARD_SIZE: (u8, u8) = (16, 24);
const ROW_SPACING: u8 = 12;

//COLORS
const CARD_COLOR: u32 = 0x1E3A8Aff;
const CARD_HIGHLIGHT: u32 = 0x2563EBff;
const CARD_FLIPPED_COLOR: u32 = 0xF0F0F0ff;

turbo::init! {
    struct GameState {
        board: Option<Board>,
    } = {
        Self {
            board: None,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    //draw the background
    draw_checkerboard();

    let m = pointer();

    //get the board from the file system.
    state.board = watch_file("card_search", "board")
        .data
        .and_then(|file| Board::try_from_slice(&file.contents).ok()); //deserialize the board

    //if we have a board, draw the cards and handle any clicks
    match &mut state.board {
        None => {}
        Some(b) => {
            let crown_found = is_crown_found(&b);
            for card in &mut b.cards {
                card.draw((m.x, m.y));
                if m.just_pressed() && !crown_found {
                    card.on_click((m.x, m.y));
                }
            }
        }
    }

    //Watch for alerts
    if let Some(event) = os::client::watch_events("card_search", Some("alert")).data {
        //Display an alert banner for notifications that are < 10s old
        let duration = 10_000;
        let millis_since_event = time::now() - event.created_at as u64 * 1000;
        if millis_since_event < duration {
            if let Ok(msg) = std::str::from_utf8(&event.data) {
                let txt = format!("User {}", msg);
                centered_text(&txt, 200, CARD_FLIPPED_COLOR);
                centered_text("Found the crown", 210, CARD_FLIPPED_COLOR);
            }
        }
    }
    //if you don't have a board (should never happen) or the crown is found
    //then show text saying press Z to start a new game
    if state.board.is_none() || state.board.as_ref().map_or(false, is_crown_found) {
        //show text to press z to get a new game
        centered_text("Press Z", 10, CARD_FLIPPED_COLOR);
        centered_text("To Start New Game", 20, CARD_FLIPPED_COLOR);
        let gp = gamepad(0);
        if gp.a.just_pressed() {
            //if you press Z (gamepad a), call the generate_board function
            os::client::exec("card_search", "generate_board", &[]);
        }
        //if crown isn't found, then show this text instead
    } else {
        centered_text("Find the Crown!", 10, CARD_FLIPPED_COLOR);
    }

    state.save();
});

fn is_crown_found(board: &Board) -> bool {
    for c in &board.cards {
        if c.is_crown && c.is_flipped {
            return true;
        }
    }
    false
}

#[export_name = "turbo/card_click"]
unsafe extern "C" fn on_card_click() -> usize {
    //read the current board from the server
    //or if there is no board, then make a blank board
    let mut board = os::server::read_or!(Board, "board", Board { cards: Vec::new() });
    //if there is no board, cancel the function
    if board.cards.len() == 0 {
        return os::server::CANCEL;
    } else {
        //num is the command data, which is the card id
        let num = os::server::command!(u8);
        for c in &mut board.cards {
            if c.id == num && c.is_flipped == false {
                c.is_flipped = true;
                if c.is_crown {
                    let userid = os::server::get_user_id();
                    let userid = truncate_string(&userid, 8);
                    //send alert of the user id that found the crown card
                    os::server::alert!("{}", userid);
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
    //get a random number from the server
    let mut num: u32 = os::server::random_number();
    //set the random to a number between 0 and 15
    num = num % BOARD_SIZE as u32;
    for c in &mut board.cards {
        if c.id as u32 == num {
            c.is_crown = true;
        }
    }
    //write the new board to the server
    let Ok(_) = os::server::write!(&file_path, board) else {
        return os::server::CANCEL;
    };

    os::server::log!("Crown: {}", num);
    return os::server::COMMIT;
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[..max_len].to_string()
    } else {
        s.to_string()
    }
}

//create 16 cards and give them id from 0 through 15
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
        let margin_y = 48;
        let row = self.id / 4;
        let col = self.id % 4;

        let x = margin_x + (col as u32 * (CARD_SIZE.0 as u32 + ROW_SPACING as u32));
        let y = margin_y + (row as u32 * (CARD_SIZE.1 as u32 + ROW_SPACING as u32));

        (x, y)
    }

    fn on_click(&mut self, pos: (i32, i32)) {
        if self.is_hovered(pos) && self.is_flipped != true {
            //serialize the data to bytes, then send the card_click command to the server
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

//draw the checkerboard background
fn draw_checkerboard() {
    let width = 132;
    let height = 224;
    let cols = 8;
    let rows = 14;

    let tile_width = (width + cols - 1) / cols;
    let tile_height = (height + rows - 1) / rows;

    let dark_color = 0x1A1A1Aff;
    let light_color = 0x202020ff;

    for row in 0..rows {
        for col in 0..cols {
            let x = col * tile_width;
            let y = row * tile_height;
            let color = if (row + col) % 2 == 0 {
                dark_color
            } else {
                light_color
            };

            rect!(
                x = x as i32,
                y = y as i32,
                w = tile_width as u8,
                h = tile_height as u8,
                color = color,
            );
        }
    }
}

//centers text of any length
fn centered_text(text: &str, y: i32, color: u32) {
    let x = centered_pos(text, 5, 132);
    text!(text, x = x, y = y, color = color,);
}

fn centered_pos(text: &str, char_width: i32, full_width: i32) -> i32 {
    (full_width - (text.len() as i32 * char_width)) / 2
}
