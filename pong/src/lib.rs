use turbo::*;


#[turbo::game]
struct GameState {
    p1_score: u32,
    p2_score: u32,
    paddle1: Paddle,
    paddle2: Paddle,
    ball: Ball,
}

impl GameState {
    fn new() -> Self {
        let canvas_size = resolution();
        let w = canvas_size.0 as f32;
        let h = canvas_size.1 as f32;
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
    fn update(&mut self) {

        let paddle_speed = 4.0;

        let canvas_size = resolution();
        let screen_w = canvas_size.0 as f32;
        let screen_h = canvas_size.1 as f32;

        let gp1 = gamepad::get(0);
        let gp2 = gamepad::get(1);

        // Debug log self
        if gp1.start.pressed() || gp2.start.pressed() {
            log!("{self:?}");
        }

        // Move paddle 1
        if gp1.up.pressed() && self.paddle1.y > 0.0 {
            self.paddle1.y -= paddle_speed;
        }
        if gp1.down.pressed() && self.paddle1.y + self.paddle1.height < screen_h {
            self.paddle1.y += paddle_speed;
        }

        // Move paddle 2
        if gp2.up.pressed() && self.paddle2.y > 0.0 {
            self.paddle2.y -= paddle_speed;
        }
        if gp2.down.pressed() && self.paddle2.y + self.paddle2.height < screen_h {
            self.paddle2.y += paddle_speed;
        }

        // Update ball position
        self.ball.x += self.ball.velocity_x;
        self.ball.y += self.ball.velocity_y;

        // Ball out of bounds (scoring)
        let did_p1_score = self.ball.x + self.ball.radius * 2.0 >= screen_w as f32;
        if did_p1_score {
            self.p1_score += 1;
        }
        let did_p2_score = self.ball.x < 0.0;
        if did_p2_score {
            self.p2_score += 1;
        }
        if did_p1_score || did_p2_score {
            // Reset ball position
            self.ball.x = screen_w as f32 / 2.0;
            self.ball.y = screen_h / 2.0;
        }

        // Ball collisions with paddles
        if (self.ball.x - self.ball.radius < self.paddle1.x + 8.0
            && self.ball.y > self.paddle1.y
            && self.ball.y < self.paddle1.y + self.paddle1.height)
            || (self.ball.x + self.ball.radius > self.paddle2.x
                && self.ball.y > self.paddle2.y
                && self.ball.y < self.paddle2.y + self.paddle2.height)
        {
            self.ball.velocity_x = -self.ball.velocity_x;
        }

        // Ball collisions with top and bottom
        if self.ball.y - self.ball.radius < 0.0 || self.ball.y + self.ball.radius > screen_h {
            self.ball.velocity_y = -self.ball.velocity_y;
        }

        // Draw paddles and ball
        rect!(
            x = self.paddle1.x as i32,
            y = self.paddle1.y as i32,
            w = 8,
            h = self.paddle1.height as u32,
            color = 0xffffffff
        );
        rect!(
            x = self.paddle2.x as i32,
            y = self.paddle2.y as i32,
            w = 8,
            h = self.paddle2.height as u32,
            color = 0xffffffff
        );
        circ!(
            x = self.ball.x as i32,
            y = self.ball.y as i32,
            d = self.ball.radius as u32,
            color = 0xffffffff
        );
        text!("P1: {}", self.p1_score; font = "large", x = 64);
        text!(
            "P2: {}", self.p2_score;
            font = "large",
            x = (screen_w as i32 / 2) + 64
        );
    }
}

#[turbo::serialize]
struct Paddle {
    x: f32,
    y: f32,
    height: f32,
}
#[turbo::serialize]
struct Ball {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    radius: f32,
}
