use turbo::*;

// colors
const BACKGROUND_COLOR: u32 = 0x2B2B2Bff;
const WHITE_COLOR: u32 = 0xFFFFFFff;
const GREEN_COLOR: u32 = 0x00FF7Fff;
const RED_COLOR: u32 = 0xFF4040ff;
const BUTTON_COLOR: u32 = 0x4169E1ff;
const BUTTON_TEXT_COLOR: u32 = 0xF0F8FFff;

#[turbo::game]
struct GameState;
impl GameState {
    fn update(&mut self) {
        clear(BACKGROUND_COLOR);

        let (w, h) = (30, 20);
        let (x_minus, y_minus) = (20, 180);
        let (x_plus, y_plus) = (82, 180);

        draw_button(w, h, x_minus, y_minus);
        draw_button(w, h, x_plus, y_plus);

        text!(
            "-",
            x = 32,
            y = 187,
            font = "large",
            color = BUTTON_TEXT_COLOR
        );
        text!(
            "+",
            x = 94,
            y = 187,
            font = "large",
            color = BUTTON_TEXT_COLOR
        );

        let pointer = pointer::screen();
        if pointer.just_pressed() {
            if pointer.intersects(x_minus, y_minus, w, h) {
                counter::IncrementCounter::Minus(1).exec();
            }
            if pointer.intersects(x_plus, y_plus, w, h) {
                counter::IncrementCounter::Plus(1).exec();
            }
        }

        if let Some(ref id) = os::client::user_id() {
            let truncated = if id.len() > 8 {
                format!("{}...", &id[..8])
            } else {
                id.to_string()
            };
            let user_line = format!("User: {}", truncated);
            text!(
                &user_line,
                x = 10,
                y = 10,
                font = "medium",
                color = WHITE_COLOR
            );

            let program_files_path =
                std::path::PathBuf::new().join(counter::IncrementCounter::PROGRAM_ID);
            let user_count_filepath = program_files_path.join("users").join(id);
            let query = os::client::fs::watch(&user_count_filepath);
            let user_count = query.parse().unwrap_or(0);
            let user_line = format!("Your Count: {}", user_count);
            text!(
                &user_line,
                x = 10,
                y = 25,
                font = "medium",
                color = WHITE_COLOR
            );

            let global_count_filepath = program_files_path.join("global_count");
            let global_count = os::client::fs::watch(&global_count_filepath)
                .parse()
                .unwrap_or(0);
            let color = if global_count < 0 {
                RED_COLOR
            } else {
                GREEN_COLOR
            };
            let global_line = format!("Global Count: {}", global_count);
            text!(&global_line, x = 10, y = 40, font = "medium", color = color);
        }
    }
}

fn draw_button(w: i32, h: i32, x: i32, y: i32) {
    rect!(
        w = w,
        h = h,
        x = x,
        y = y,
        color = BUTTON_COLOR,
        border_radius = 2
    );
}

pub mod counter {
    use super::*;

    #[turbo::os::command(program = "counter", name = "increment_counter")]
    pub enum IncrementCounter {
        Plus(i32),
        Minus(i32),
    }
    impl IncrementCounter {
        pub fn amount(&self) -> i32 {
            match self {
                Self::Plus(n) => *n,
                Self::Minus(n) => -*n,
            }
        }
    }
    impl CommandHandler for IncrementCounter {
        fn run(&mut self, user_id: &str) -> Result<(), std::io::Error> {
            use os::server::*;
            log!("Running increment command: {self:?}");

            let delta = self.amount();

            let user_path = format!("users/{}", user_id);
            let mut user_counter = fs::read(&user_path).unwrap_or(0);
            user_counter += delta;
            fs::write(&user_path, &user_counter)?;

            let global_path = "global_count";
            let mut global_counter = fs::read(global_path).unwrap_or(0);
            global_counter += delta;
            fs::write(global_path, &global_counter)?;

            log!("Global Counter: {}", global_counter);
            Ok(())
        }
    }
}
