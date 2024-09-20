use crate::*;


// Feeling enum and functions
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Feeling {
    Wonder,
    Love,
    Ritual,
    Stress,
    Daze,
    Gloom,
}

impl Feeling {
    // Function to return the color of each feeling
    pub fn color(&self) -> u32 {
        let c: u32;
        match self {
            Feeling::Wonder => c = 0xa6884aff,
            Feeling::Love => c = 0x9d405cff,
            Feeling::Ritual => c = 0x4e7499ff,
            Feeling::Stress => c = 0x83473dff,
            Feeling::Daze => c = 0x645360ff,
            Feeling::Gloom => c = 0x595652ff
        }
        c
    }

    // Function to return the sprite of each feeling
    pub fn sprite(&self) -> &str {
        let c: &str;
        match self {
            Feeling::Wonder => c = "wonder",
            Feeling::Love => c = "love",
            Feeling::Ritual => c = "ritual",
            Feeling::Stress => c = "stress",
            Feeling::Daze => c = "daze",
            Feeling::Gloom => c = "gloom"
        }
        c
    }

    // Function to return a vec of feelings of a specified size
    pub fn all(count: u32) -> &'static[Feeling] {
        match count {
            3 => &[Feeling::Wonder, Feeling::Ritual, Feeling::Daze],
            4 => &[Feeling::Wonder, Feeling::Love, Feeling::Ritual, Feeling::Daze],
            5 => &[Feeling::Wonder, Feeling::Love, Feeling::Ritual, Feeling::Stress, Feeling::Daze],
            _ => &[Feeling::Wonder, Feeling::Love, Feeling::Ritual, Feeling::Stress, Feeling::Daze],
        }
    }

    // Function to return a vec of random feelings of a specified size and specified pool size
    pub fn rnd_vec(count: i32, pool: u32) -> Vec<Feeling> {
        let mut vec: Vec<Feeling> = vec![];
        for _ in 0..count {
            let rnd = (Util::rand01() * Feeling::all(pool).len() as f32).floor();
            vec.push(Feeling::all(pool)[rnd as usize].clone());
        }
        vec
    }
}


#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct UIButton {
    pub hitbox: (i32, i32, i32, i32),
    pub text: String,
    pub hovered: bool,

}
impl UIButton {
    pub fn new(text: &str, hitbox: (i32, i32, i32, i32)) -> Self {
        Self {
            hitbox,
            text: text.to_string(),
            hovered: false,
        }
    }

    // Update function soley to wait for animation
    pub fn update() {

    }

    pub fn draw(&self) {
        let c1: u32;
        let c2: u32;
        if self.hovered {
            Util::nine_slice("button_hover", 5, self.hitbox.2, self.hitbox.3, self.hitbox.0, self.hitbox.1);    
            c1 = 0xccdee3ff;
            c2 = 0x323b42ff;
            
        } else {
            Util::nine_slice("button", 5, self.hitbox.2, self.hitbox.3, self.hitbox.0, self.hitbox.1);    
            c1 = 0x323b42ff;
            c2 = 0xccdee3ff;
        }
        text!(&self.text, x = self.hitbox.0 + (self.hitbox.2/2) + 1 - (self.text.len() as f32 * 2.5) as i32, y = self.hitbox.1 + (self.hitbox.3/2) - 2, font = Font::M, color = c2);
        text!(&self.text, x = self.hitbox.0 + (self.hitbox.2/2) + 1 - (self.text.len() as f32 * 2.5) as i32, y = self.hitbox.1 + (self.hitbox.3/2) - 3, font = Font::M, color = c1);
    }
}


#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct FloatingText {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
    pub lifetime: u32,
    pub text: String,
    pub color: u32,
    pub color2: u32
}
impl FloatingText {
    pub fn new(text: &str, x: i32, y: i32, color: u32) -> Self {
        Self  {
            x,
            y,
            text: text.to_string(),
            timer: 0,
            lifetime: 90,
            color: {
                match text.parse::<i32>() {
                    Ok(value) => {
                        if value >= 0 {
                            0xffffffff
                        } else {
                            0x9d405cff
                        }
                    },
                    Err(_) => color
                }
            },
            color2: {
                match text.parse::<i32>() {
                    Ok(value) => {
                        if value >= 0 {
                            0xa6884aff
                        } else {
                            0xffffffff
                        }
                    },
                    Err(_) => color
                }
            },
        }
    }

    pub fn update(&mut self) {
        self.timer += 1;
        if self.timer%10 == 0 {
            self.y -= 1;
        }
    }

    pub fn draw(&self) {
        text!(&self.text, x = self.x, y = self.y + 1, font = Font::L, color = 0x323b42ff);
        text!(&self.text, x = self.x, y = self.y, font = Font::L, color = self.color);
    }
}


pub struct Util {}
impl Util {
    pub fn rand01() -> f32 {
        (rand() as f32 % 100.) / 100. // Random value between 0 and 1
    }

    pub fn nine_slice(name: &str, size: i32, w: i32, h: i32, x: i32, y: i32) {
        // center
        sprite!(
            name,
            x = x + size,
            y = y + size,
            sx = 1 * size,
            sy = 1 * size,
            sw = size,
            sh = size,
            w = w - size*2,
            h = h - size*2,
            repeat = true,
        );
    
        // top
        sprite!(
            name,
            x = x + size,
            y = y,
            sx = 1 * size,
            sy = 0 * size,
            sw = size,
            sh = size,
            w = w - size*2,
            repeat = true,
        );
        // bottom
        sprite!(
            name,
            x = x + size,
            y = y + h - size,
            sx = 1 * size,
            sy = 2 * size,
            sw = size,
            sh = size,
            w = w - size*2,
            repeat = true,
        );
        // left
        sprite!(
            name,
            x = x,
            y = y + size,
            sx = 0 * size,
            sy = 1 * size,
            sw = size,
            sh = size,
            h = h - size*2,
            repeat = true,
        );
        // right
        sprite!(
            name,
            x = x + w - size,
            y = y + size,
            sx = 2 * size,
            sy = 1 * size,
            sw = size,
            sh = size,
            h = h - size*2,
            repeat = true,
        );
    
        // top-left
        sprite!(
            name,
            x = x,
            y = y,
            sx = 0 * size,
            sy = 0,
            sw = size,
            sh = size,
        );
        // top-right
        sprite!(
            name,
            x = x + w - size,
            y = y,
            sx = 2 * size,
            sy = 0,
            sw = size,
            sh = size,
        );
    
        // bottom-left
        sprite!(
            name,
            x = x,
            y = y + h - size,
            sx = 0 * size,
            sy = 2 * size,
            sw = size,
            sh = size,
        );
        // bottom-right
        sprite!(
            name,
            x = x + w - size,
            y = y + h - size,
            sx = 2 * size,
            sy = 2 * size,
            sw = size,
            sh = size,
        );
    }
}