use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TrapType {
    Poop,
    Healing,
    Acidleak,
    Landmine,
    Spikes,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Trap {
    pub trap_type: TrapType,
    pub size: f32,
    pub pos: (f32, f32),
    pub damage: f32,
    pub on_dur: i32,
    pub off_dur: i32,
    pub timer: i32,
    pub sprite_name: String,
}

impl Trap {
    // New trap with 4 parameters, timer always starts at 0
    pub fn new(pos: (f32, f32), trap_type: TrapType) -> Self {
        let (size, damage, on_dur, off_dur, sprite_name) = match trap_type {
            TrapType::Poop => (
                8.0,                // size
                0.0,                // damage
                1,                  // on_dur
                0,                  // off_dur
                "poop".to_string(), // sprite_name
            ),
            TrapType::Healing => (
                8.0,                   // size
                0.0,                   // damage (negative for healing)
                1,                     // on_dur
                0,                     // off_dur
                "healing".to_string(), // sprite_name
            ),
            TrapType::Acidleak => (
                8.0,                    // size
                0.0,                    // damage
                1,                      // on_dur
                0,                      // off_dur
                "acidleak".to_string(), // sprite_name
            ),
            TrapType::Landmine => (
                8.0,                    // size
                100.0,                  // damage
                1,                      // on_dur (instant explosion)
                0,                      // off_dur (long cooldown)
                "landmine".to_string(), // sprite_name
            ),
            TrapType::Spikes => (
                8.0,                  // size
                25.0,                 // damage
                90,                   // on_dur
                90,                   // off_dur
                "spikes".to_string(), // sprite_name
            ),
        };

        Trap {
            size,
            pos,
            damage,
            on_dur,
            off_dur,
            sprite_name,
            timer: 0,
            trap_type,
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
        if self.is_active() {
            sprite!(
                self.sprite_name.as_str(),
                x = self.sprite_draw_pos().0,
                y = self.sprite_draw_pos().1,
                fps = fps::FAST
            );
        }
    }

    pub fn draw_pos(&self) -> (f32, f32) {
        (self.pos.0 - self.size / 2., self.pos.1 - self.size / 2.)
    }

    pub fn set_inactive(&mut self) {
        self.on_dur = -1;
    }
    //this might need to be dependent on trap type, we'll see
    pub fn sprite_draw_pos(&self) -> (f32, f32) {
        (self.pos.0 - self.size as f32, self.pos.1 - self.size)
    }
    // Helper function to check if the trap is currently active
    pub fn is_active(&self) -> bool {
        self.timer <= self.on_dur
    }
}
