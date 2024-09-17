use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Trap {
    pub size: f32,
    pub pos: (f32, f32),
    pub damage: f32,
    pub on_dur: i32,
    pub off_dur: i32,
    pub timer: i32,
}

impl Trap {
    // New trap with 4 parameters, timer always starts at 0
    pub fn new(size: f32, pos: (f32, f32), damage: f32, on_dur: i32, off_dur: i32) -> Self {
        Trap {
            size,
            pos,
            damage,
            on_dur,
            off_dur,
            timer: 0,
        }
    }

    // Update function: add 1 to timer, if timer is greater than off_dur+on_dur reset it to 0
    pub fn update(&mut self) {
        self.timer += 1;
        if self.timer > self.off_dur + self.on_dur {
            self.timer = 0;
        }
    }

    pub fn draw(&self) {
        if self.timer <= self.on_dur {
            circ!(
                x = self.draw_pos().0,
                y = self.draw_pos().1,
                d = self.size,
                color = 0xFF0000ff
            );
        }
    }

    pub fn draw_pos(&self) -> (f32, f32) {
        (self.pos.0 - self.size / 2., self.pos.1 - self.size / 2.)
    }
    // Helper function to check if the trap is currently active
    pub fn is_active(&self) -> bool {
        self.timer <= self.on_dur
    }
}