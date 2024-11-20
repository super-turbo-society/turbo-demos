use os::server;
use turbo::borsh::*;

turbo::cfg! {r#"
    name = "Counter"
    version = "1.0.0"
    author = "Turbo"
    description = "Set Up a Counter in Turbo OS"
    [settings]
    resolution = [132, 224]
    [turbo-os]
    api-url = "http://localhost:8000"
"#}

//colors
const BACKGROUND_COLOR: u32 = 0x2B2B2Bff;
const WHITE_COLOR: u32 = 0xFFFFFFff;
const GREEN_COLOR: u32 = 0x00FF7Fff;
const RED_COLOR: u32 = 0xFF4040ff;
const BUTTON_COLOR: u32 = 0x4169E1ff;
const BUTTON_TEXT_COLOR: u32 = 0xF0F8FFff;

turbo::go!({
    clear!(BACKGROUND_COLOR);
    //draw the minus button
    let (w, h) = (30, 20);
    let (x, y) = (20, 180);
    draw_button(w, h, x, y);
    text!(
        "-",
        x = 32,
        y = 187,
        font = Font::L,
        color = BUTTON_TEXT_COLOR
    );
    let m = mouse(0);

    //subtract 1 if the minus button is clicked
    if m.left.just_pressed() && button_contains_pos(m.position[0], m.position[1], w, h, x, y) {
        let delta: i32 = -1;
        let bytes = delta.to_le_bytes();
        os::client::exec("counter", "increment_counter", &bytes);
    }

    //draw the plus button
    let (x, y) = (82, 180);
    draw_button(w, h, x, y);
    text!(
        "+",
        x = 94,
        y = 187,
        font = Font::L,
        color = BUTTON_TEXT_COLOR
    );

    //Add one to the counter if plus button is clicked
    if m.left.just_pressed() && button_contains_pos(m.position[0], m.position[1], w, h, x, y) {
        let delta: i32 = 1;
        let bytes = delta.to_le_bytes();
        os::client::exec("counter", "increment_counter", &bytes);
    }

    //draw texts on top of screen
    let userid = os::client::user_id();
    if let Some(ref id) = userid {
        let truncated = if id.len() > 8 {
            format!("{}...", &id[..8])
        } else {
            id.to_string()
        };
        let txt = format!("User: {}", truncated);
        //draw the user ID, truncated for only 8 digits
        text!(&txt, x = 10, y = 10, font = Font::M, color = WHITE_COLOR);

        //draw the user's saved counter
        let filepath = format!("users/{}", id);
        //read the number from the server using watch_file
        let num = os::client::watch_file("counter", &filepath, &[("stream", "true")])
            .data
            .and_then(|file| i32::try_from_slice(&file.contents).ok())
            .unwrap_or(0); //set to 0 if the file doesn't exist
        let txt = format!("Your Count: {}", num);
        text!(&txt, x = 10, y = 25, font = Font::M, color = WHITE_COLOR);

        //draw the global count
        let filepath = "global_count";
        //read the number from the server using watch_file
        let num = os::client::watch_file("counter", &filepath, &[("stream", "true")])
            .data
            .and_then(|file| i32::try_from_slice(&file.contents).ok())
            .unwrap_or(0); //set to 0 if the file doesn't exist
        let txt = format!("Global Count: {}", num);

        let color = if num < 0 { RED_COLOR } else { GREEN_COLOR };
        text!(&txt, x = 10, y = 40, font = Font::M, color = color);
    }
});

fn draw_button(w: i32, h: i32, x: i32, y: i32) {
    rect!(
        w = w,
        h = h,
        y = y,
        x = x,
        color = BUTTON_COLOR,
        border_radius = 2
    );
}

fn button_contains_pos(px: i32, py: i32, w: i32, h: i32, x: i32, y: i32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

#[export_name = "turbo/increment_counter"]
unsafe extern "C" fn on_increment_counter() -> usize {
    let userid = os::server::get_user_id();
    let file_path = format!("users/{}", userid);
    //read the current number from the users file, or set it to 0 if it doesn't exist
    let mut counter = os::server::read_or!(i32, &file_path, 0);
    //get command data from the function call
    let increment_amt = os::server::command!(i32);

    counter += increment_amt;

    let Ok(_) = os::server::write!(&file_path, counter) else {
        return os::server::CANCEL;
    };

    let file_path = "global_count";
    let mut counter = os::server::read_or!(i32, &file_path, 0);
    counter += increment_amt;
    let Ok(_) = os::server::write!(&file_path, counter) else {
        return os::server::CANCEL;
    };
    os::server::log!("Counter: {}", counter);
    return os::server::COMMIT;
}
