turbo::cfg! {r#"
    name = "Pong"
    version = "1.0.0"
    author = "Turbo"
    description = "A simple Pong game"
    [settings]
    resolution = [256, 144]
"#}

turbo::init! {
    struct GameState {
        p1_score: u32,
        p2_score: u32,
        paddle1: struct Paddle {
            x: f32,
            y: f32,
            height: f32,
        },
        paddle2: Paddle,
        ball: struct Ball {
            x: f32,
            y: f32,
            velocity_x: f32,
            velocity_y: f32,
            radius: f32,
        },
    } = {
        let res = resolution();
        let w = res[0] as f32;
        let h = res[1] as f32;
        let paddle_height = 32.0;
        let paddle_width = 8.0;
        let ball_radius = 4.0;
        Self {
            p1_score: 0,
            p2_score: 0,
            paddle1: Paddle { x: 10.0, y: h / 2.0 - paddle_height / 2.0, height: paddle_height },
            paddle2: Paddle { x: w - paddle_width - 10.0, y: h / 2.0 - paddle_height / 2.0, height: paddle_height },
            ball: Ball { x: w / 2.0, y: h / 2.0, velocity_x: 2.0, velocity_y: 2.0, radius: ball_radius },
        }
    }
}

turbo::go! ({
    let mut state = GameState::load();

    let paddle_speed = 4.0;

    let res = resolution();
    let screen_w = res[0] as f32;
    let screen_h = res[1] as f32;

    let gp1 = gamepad(0);
    let gp2 = gamepad(1);

    // Debug log state
    if gp1.start.pressed() || gp2.start.pressed() {
        log!("{state:?}");
    }

    // Move paddle 1
    if gp1.up.pressed() && state.paddle1.y > 0.0 {
        state.paddle1.y -= paddle_speed;
    }
    if gp1.down.pressed() && state.paddle1.y + state.paddle1.height < screen_h {
        state.paddle1.y += paddle_speed;
    }

    // Move paddle 2
    if gp2.up.pressed() && state.paddle2.y > 0.0 {
        state.paddle2.y -= paddle_speed;
    }
    if gp2.down.pressed() && state.paddle2.y + state.paddle2.height < screen_h {
        state.paddle2.y += paddle_speed;
    }

    // Update ball position
    state.ball.x += state.ball.velocity_x;
    state.ball.y += state.ball.velocity_y;

    // Ball out of bounds (scoring)
    let did_p1_score = state.ball.x + state.ball.radius * 2.0 >= screen_w as f32;
    if did_p1_score {
        state.p1_score += 1;
    }
    let did_p2_score = state.ball.x < 0.0;
    if did_p2_score {
        state.p2_score += 1;
    }
    if did_p1_score || did_p2_score {
        // Reset ball position
        state.ball.x = screen_w as f32 / 2.0;
        state.ball.y = screen_h / 2.0;
    }

    // Ball collisions with paddles
    if (state.ball.x - state.ball.radius < state.paddle1.x + 8.0 &&
        state.ball.y > state.paddle1.y &&
        state.ball.y < state.paddle1.y + state.paddle1.height) ||
       (state.ball.x + state.ball.radius > state.paddle2.x &&
        state.ball.y > state.paddle2.y &&
        state.ball.y < state.paddle2.y + state.paddle2.height) {
        state.ball.velocity_x = -state.ball.velocity_x;
    }

    // Ball collisions with top and bottom
    if state.ball.y - state.ball.radius < 0.0 || state.ball.y + state.ball.radius > screen_h {
        state.ball.velocity_y = -state.ball.velocity_y;
    }

    // Draw paddles and ball
    rect!(
        x = state.paddle1.x,
        y = state.paddle1.y,
        w = 8,
        h = state.paddle1.height,
        color = 0xffffffff
    );
    rect!(
        x = state.paddle2.x,
        y = state.paddle2.y,
        w = 8,
        h = state.paddle2.height,
        color = 0xffffffff
    );
    circ!(
        x = state.ball.x,
        y = state.ball.y,
        d = state.ball.radius,
        color = 0xffffffff
    );
    text!("P1: {}", state.p1_score; font = Font::L, x = 64);
    text!("P2: {}", state.p2_score; font = Font::L, x = (screen_w / 2.0) + 64.0);

    // Save game state for the next frame
    state.save();
})
