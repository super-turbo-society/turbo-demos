turbo::cfg! {r#"
    name = "Brick Breaker"
    version = "1.0.0"
    author = "Turbo"
"#}

turbo::init! {
    struct GameState {
        paddle: struct Paddle {
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            speed: f32,
           
        },
        ball: struct Ball {
            x: f32,
            y: f32,
            radius: f32,
            velocity_x: f32,
            velocity_y: f32,
        },
        bricks: Vec<struct Brick {
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            color: u32,
            visible: bool,
        }>,
        score: u32,
        lives: u32,
        game_over: bool,
    } = {
        let res = resolution();
        let w = res[0] as f32;
        let h = res[1] as f32;
        let paddle_width = 80.0;
        let paddle_height = 10.0;
        let ball_radius = 10.0;
        let brick_width = 50.0;
        let brick_height = 20.0;
        let brick_rows = 5;
        let brick_cols = 10;
        let brick_spacing = 2.0;
        let brick_top_offset = 50.0;
        let brick_colors = [
            0xff0000ff, 0xff8000ff, 0xffff00ff, 0x00ff00ff, 0x00ffffff,
            0x0000ffff, 0x8000ffff, 0xff00ffff,
        ];

        let mut bricks = Vec::new();
        for row in 0..brick_rows {    
            for col in 0..brick_cols {
                let color_index = row % brick_colors.len();
                let brick_x = col as f32 * (brick_width + brick_spacing) + 1.0;
                let brick_y = row as f32 * (brick_height + brick_spacing) + brick_top_offset;
                bricks.push(Brick {
                    x: brick_x,
                    y: brick_y,
                    width: brick_width,
                    height: brick_height,
                    color: brick_colors[color_index],
                    visible: true,
                });
            }
        }

        Self {
            paddle: Paddle {
                x: (w - paddle_width) / 2.0,
                y: h - 40.0,
                width: paddle_width,
                height: paddle_height,
                speed: 5.0,
            },
            ball: Ball {
                x: w / 2.0,
                y: h / 2.0,
                radius: ball_radius,
                velocity_x: 3.0,
                velocity_y: -3.0,
            },
            bricks,
            score: 0,
            lives: 3,
            game_over: false,
        }
    }



}
impl GameState {
    // Create a new game state
    fn new() -> Self {
        let res = resolution();
        let w = res[0] as f32;
        let h = res[1] as f32;
        let paddle_width = 80.0;
        let paddle_height = 10.0;
        let ball_radius = 10.0;
        let brick_width = 50.0;
        let brick_height = 20.0;
        let brick_rows = 5;
        let brick_cols = 10;
        let brick_spacing = 2.0;
        let brick_top_offset = 50.0;
        let brick_colors = [
            0xff0000ff, 0xff8000ff, 0xffff00ff, 0x00ff00ff, 0x00ffffff, 0x0000ffff, 0x8000ffff,
            0xff00ffff,
        ];

        let mut bricks = Vec::new();
        for row in 0..brick_rows {
            for col in 0..brick_cols {
                let color_index = row % brick_colors.len();
                let brick_x = col as f32 * (brick_width + brick_spacing) + 1.0;
                let brick_y = row as f32 * (brick_height + brick_spacing) + brick_top_offset;
                bricks.push(Brick {
                    x: brick_x,
                    y: brick_y,
                    width: brick_width,
                    height: brick_height,
                    color: brick_colors[color_index],
                    visible: true,
                });
            }
        }

        GameState {
            paddle: Paddle {
                x: (w - paddle_width) / 2.0,
                y: h - 40.0,
                width: paddle_width,
                height: paddle_height,
                speed: 5.0,
            },
            ball: Ball {
                x: w / 2.0,
                y: h / 2.0,
                radius: ball_radius,
                velocity_x: 3.0,
                velocity_y: -3.0,
            },
            bricks,
            score: 0,
            lives: 3,
            game_over: false,
        }
    }
}

turbo::go! {
    let mut state = GameState::load();
    let paddle = &mut state.paddle;
    let ball = &mut state.ball;
     // Move the paddle
     let gp1 = gamepad(0);

    if state.lives > 0 {
        // Draw the paddle
        rect!(
            x = paddle.x as i32,
            y = paddle.y as i32,
            w = paddle.width as u32,
            h = paddle.height as u32,
            color = 0xffffffff
        );

        // Draw the ball
        circ!(
            x = ball.x as i32,
            y = ball.y as i32,
            d = ball.radius as u32,
            color = 0xffffffff
        );

        // Draw the bricks
        for brick in &state.bricks {
            if brick.visible {
                rect!(
                    x = brick.x as i32,
                    y = brick.y as i32,
                    w = brick.width as u32,
                    h = brick.height as u32,
                    color = brick.color
                );
            }
        }


        if gp1.left.pressed() && paddle.x > 0.0 {
            paddle.x -= paddle.speed;
        }
        if gp1.right.pressed() && paddle.x + paddle.width < 256.0 {
            paddle.x += paddle.speed;
        }

        // Move the ball
        ball.x += ball.velocity_x;
        ball.y += ball.velocity_y;

        // Check ball collision with walls
        if ball.x - ball.radius < 0.0 || ball.x + ball.radius > 256.0 {
            ball.velocity_x = -ball.velocity_x;
        }
        if ball.y - ball.radius < 0.0 {
            ball.velocity_y = -ball.velocity_y;
        }

        // Check ball collision with paddle
        if ball.y + ball.radius > paddle.y
            && ball.x > paddle.x
            && ball.x < paddle.x + paddle.width
        {
            ball.velocity_y = -ball.velocity_y;
        }

        // Check ball collision with bricks
        for brick in &mut state.bricks {
            if brick.visible {
                if ball.x + ball.radius > brick.x
                    && ball.x - ball.radius < brick.x + brick.width
                    && ball.y + ball.radius > brick.y
                    && ball.y - ball.radius < brick.y + brick.height
                {
                    brick.visible = false;
                    ball.velocity_y = -ball.velocity_y;
                    state.score += 1;
                    break;
                }
            }
        }

        // Check ball falling out of screen
        if ball.y + ball.radius > 144.0 {
            state.lives -= 1;
            if state.lives == 0 {
                // Game over logic
                state.game_over = true;
            } else {
                ball.x = paddle.x + paddle.width / 2.0;
                ball.y = paddle.y - ball.radius;
                ball.velocity_x = 3.0;
                ball.velocity_y = -3.0;
            }
        }
    }

    if state.game_over {
        // Draw game over text
        text!("Game Over", font = Font::L, x = 100, y = 50, color = 0xffffffff);
        text!("- press start -", x = 88, y = 84, font = Font::M, color = 0xffffffff);
        if   gp1.start.just_pressed() {
            state = GameState::new()
        }
    }

     // Draw score on the screen
     text!(
        &format!("Score: {}", state.score),
        font = Font::L,
        x = 10,
        y = 10,
        color = 0xffffffff
    );

    // Draw score on the screen
    text!(
        &format!("Lives: {}", state.lives),
        font = Font::L,
        x = 100,
        y = 10,
        color = 0xffffffff
    );


    // Save the game state for the next frame
    state.save();
}
