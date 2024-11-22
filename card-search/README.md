# Card Search

![screenshot](./card-search.png)

## Description

Flip cards until you find the Crown! This game uses Turbo OS to support networked multiplayer.

The board is generated into a file named "board", and all players watch that file to see the same board. Whenever a player clicks on a card, the board file is updated. 
## Key Code Snippets

## Generate the Board on Turbo OS

This function creates a board (which is just a [BOARD_SIZE] number of cards), gets a random number from the server, and sets one card to be the crown based on the random number. Then it writes the new board to a file at the filepath "board".

Lastly it logs the card number of the crown, and then commits the changes.

```rust
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
```
## Update the board when a card is clicked

This function 
```rust
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
```