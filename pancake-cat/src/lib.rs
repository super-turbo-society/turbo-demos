use turbo::*;

#[turbo::serialize]
struct Pancake {
    x: f32,
    y: f32,
    vel: f32,
    radius: f32,
}

#[turbo::game]
struct GameState {
    last_munch_at: usize,
    cat_x: f32,
    cat_y: f32,
    cat_r: f32,
    pancakes: Vec<Pancake>,
    score: u32,
}

impl GameState {
    fn new() -> Self {
        Self {
            last_munch_at: 0,
            cat_x: 128.0,
            cat_y: 112.0,
            cat_r: 8.0,
            pancakes: vec![],
            score: 0,
        }
    }
    fn update(&mut self) {
        // Handle user input
        if gamepad::get(0).left.pressed() {
            self.cat_x -= 2.;
        }
        if gamepad::get(0).right.pressed() {
            self.cat_x += 2.;
        }

        // Generate new pancakes at random intervals
        if random::u32() % 64 == 0 {
            // Create a new pancake with random attributes
            let pancake = Pancake {
                x: (random::u32() % 256) as f32,
                y: 0.0,
                vel: (random::u32() % 3 + 1) as f32,
                radius: (random::u32() % 10 + 5) as f32,
            };
            self.pancakes.push(pancake);
        }

        // Get the current frame
        let t = time::tick();

        // Update pancake positions and check for collisions with the cat
        let cat_center = (self.cat_x + self.cat_r, self.cat_y + self.cat_r);
        self.pancakes.retain_mut(|pancake| {
            pancake.y += pancake.vel;

            // Check for collision with the cat
            let pancake_center = (pancake.x + pancake.radius, pancake.y + pancake.radius);

            // Calculate the distance between the cat and the pancake
            let dx = cat_center.0 - pancake_center.0;
            let dy = cat_center.1 - pancake_center.1;

            let distance = (dx * dx + dy * dy).sqrt();
            let radii_sum = self.cat_r + pancake.radius;
            let radii_diff = (self.cat_r - pancake.radius).abs();

            if radii_diff <= distance && distance <= radii_sum {
                // Cat caught the pancake
                self.score += 1;
                self.last_munch_at = t;
                false // Remove the pancake from the game
            } else if pancake.y < 144. + (pancake.radius * 2.) {
                true // Keep the pancake in the game if it's within the screen
            } else {
                false // Remove the pancake if it's off-screen
            }
        });

        // Set the background color
        clear(0x00ffffff);

        // Draw a tiled background of moving sprites
        sprite!(
            "heart",
            bounds = bounds::screen(),
            repeat = true,
            tx = t / 2,
            ty = t / 2
        );

        // Draw a speech bubble when the cat eats a pancake
        if t >= 64 && t.saturating_sub(self.last_munch_at) <= 60 {
            rect!(w = 30, h = 10, x = self.cat_x + 32.0, y = self.cat_y);
            circ!(d = 10, x = self.cat_x + 28.0, y = self.cat_y);
            rect!(w = 10, h = 5, x = self.cat_x + 28.0, y = self.cat_y + 5.0);
            circ!(d = 10, x = self.cat_x + 56.0, y = self.cat_y);
            text!(
                "MUNCH!",
                x = self.cat_x + 33.0,
                y = self.cat_y + 3.0,
                font = "small",
                color = 0x000000ff
            );
        }

        // Draw the cat
        sprite!(
            "munch_cat",
            x = self.cat_x - self.cat_r,
            y = self.cat_y - 16.0
        );

        // Draw the falling pancakes
        for pancake in &self.pancakes {
            circ!(
                x = pancake.x,
                y = pancake.y + 1.0,
                d = pancake.radius + 2.,
                color = 0x000000aa
            );
            circ!(
                x = pancake.x,
                y = pancake.y,
                d = pancake.radius + 1.,
                color = 0xf4d29cff
            );
            circ!(
                x = pancake.x,
                y = pancake.y,
                d = pancake.radius,
                color = 0xdba463ff
            );
        }

        // Draw the score
        text!("Score: {}", self.score; x = 10, y = 10, font = "large", color = 0xffffffff);

        // Uncomment to print game self for debugging
        // text!(&format!("{:#?}", self), y = 24);
    }
}
