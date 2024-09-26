use crate::*;

// Dreamer class
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Dreamer {
    pub hitbox: (i32, i32, i32, i32),
    pub selected: bool,
    pub awake: bool,
    pub sleeping: bool,
    pub awake_limit: i32,
    awake_timer: f32,
    pub feelings: Vec<Feeling>
}

impl Dreamer {
    pub fn new(x: i32, y: i32, awake_limit: i32, pool: u32) -> Self {
        Self {
            hitbox: (x, y, 32, 18),
            selected: false,
            awake: false,
            sleeping: false,
            awake_limit,
            awake_timer: 0.,
            feelings: Feeling::rnd_vec(4, pool),
        }
    }

    // Function used to spawn new Dreamers in grid
    pub fn spawn(count: i32, feelings_count: u32, awake_limit: i32) -> Vec<Dreamer> {
        // Define grid scaling variables
        let (x_size, y_size, x_scale, y_scale) = (
            6, 3, 40, 25
        );
        // Create grid
        let mut grid: Vec<(i32, i32)> = vec![];
        for x in 0..x_size {
            for y in 0..y_size {
                // Remove corners and one in middle row
                if !((x == 0 && y == 0) // top left
                    || (x == 0 && y == y_size - 1) // top right
                    || (x == x_size - 1 && y == 0) // bottom left
                    || (x == x_size - 1 && y == y_size - 1) // bottom right
                    || (x == x_size - 1 && y == 1)) { // from middle row
                        grid.push((x, y)); // Add coordinates to grid
                }
            } 
        }
        // Create empty vector
        let mut d = vec![];
        // Loop through count of Dreamers
        for _ in 0..count {
            // Remove randome index from grid
            let i = (grid.len() as f32 * Util::rand01()).floor() as usize;
            let mut tup = grid.remove(i);
            // Set offset if in middle row
            let x_offset = if tup.1 == 1 {
                x_scale/2
            } else {
                0
            };
            // Set final dimensions
            tup = 
                (127 - (x_size/2) * x_scale + tup.0 * x_scale + x_offset, 
                80 - (y_size/2) * y_scale + (tup.1 as f32 * y_scale as f32).floor() as i32);

            // Create Dreamer at random index, append to vector
            d.push(Dreamer::new(tup.0, tup.1, awake_limit, feelings_count));
        }
        // Return vector
        d
    }


    pub fn toggle_select(&mut self, s: bool) {
        self.selected = s;
    }

    // Function to put dreamer to sleep, returns dreamers satisfaction score as i32
    pub fn sleep(&mut self, v: Vial) -> i32{
        self.selected = false;
        self.awake = false;
        self.sleeping = true;

        let mut satisfaction: i32 = 0;

        // Create a new vec of all Vial.contents summed -- merges layers if split
        let mut sum: Vec<(Feeling, i32)> = vec!();
        for tup in v.contents {
            let mut add_new = true;
            for s in sum.iter_mut() {
                if s.0 == tup.0 { // Feeling in vec, add to its value
                    s.1 += tup.1 as i32;
                    add_new = false;
                }
            }
            if add_new { // Feeling not in vec, push new entry
                sum.push((tup.0, tup.1 as i32));
            }
        }
        // Calculate satisfaction
        // First loop through dreamer's desired feelings
        for f in self.feelings.iter() {
            // If the Vial contains a matching feeling, find it's index in the vec
            if let Some(index) = sum.iter().position(|tup| tup.0 == f.clone()) {
                // Get a mutable reference to the Sand in the Vial contents
                if let Some(contents) = sum.get_mut(index) {
                    satisfaction += ((contents.1 as f32 / (30. / self.feelings.len() as f32) as f32).clamp(0., 1.) * 25.) as i32;
                    // Remove the proportional amount for 1 unit of Feelings from the Vial
                    contents.1 -= (30. / self.feelings.len() as f32) as i32;
                    // Remove the tuple from the vec if Feeling is reduced below 0
                    if contents.1 < 0 {
                        sum.remove(index);
                    }
                }
            } 
            else {
                satisfaction -= (100 / self.feelings.len() as u32) as i32;
            }
        }
        // Then reduce score for any undesired Sand used
        for s in sum.iter_mut() {
            satisfaction -= (s.1 as f32 / (30. / self.feelings.len() as f32) * 12.5) as i32
        }
        // Add Dreamer timer bonus
        if satisfaction > 0 {
            satisfaction += (((self.awake_limit as f32 - self.awake_timer)/self.awake_limit as f32) * 100.) as i32;
        }
        //log!("{:?}", (self.awake_limit as f32 - self.awake_timer)/self.awake_limit as f32 * 100.);
        // Return satisfaction
        satisfaction
    }

    // Update Dreamer
    pub fn update(&mut self, ui: &mut GameUI, mx: i32, my: i32) -> i32{
        // Hover state
        if let Some(_) = self.hover(self.hitbox, mx, my) {
            if self.awake && !self.selected {
                self.toggle_select(true);
            }
        } else if self.selected {
            self.toggle_select(false);
        }

        // Awake timer
        if self.awake {
            if self.awake_timer < self.awake_limit as f32 {
                self.awake_timer += 1./60.;
                0
            } else {
                let s = self.sleep(Vial::new(-32, -32));
                ui.floating_text.push(FloatingText::new(&s.to_string(), self.hitbox.0 + 16, self.hitbox.1, 0));
                s
            }
        } else {
            0
        }
    }

    // Draws house portion of Dreamer visuals
    pub fn draw_home(&self) {

        if self.awake {
            rect!(x = self.hitbox.0 + 10, y = self.hitbox.1 + 15, w = 14, h = 8, color = 0xa6884aff);
            circ!(x = self.hitbox.0 + 15, y = self.hitbox.1 + 17 + (((self.awake_timer - (self.awake_limit as f32 / 4.)).clamp(0., self.awake_limit as f32) / (self.awake_limit as f32 / 2.)) * 3.) as i32, d = 4, color = 0x906c30ff);
            rect!(x = self.hitbox.0 + 10, y = self.hitbox.1 + 15, w = 14, h = 9. * self.awake_timer/self.awake_limit as f32, color = 0x906c30ff);
            if self.selected {
                sprite!("dreamer_awake_hover", x = self.hitbox.0, y = self.hitbox.1, sw = 32, fps = fps::SLOW);
            } else {
                sprite!("dreamer_awake", x = self.hitbox.0, y = self.hitbox.1, sw = 32,fps = fps::SLOW);
            }
        } else if self.sleeping {
            sprite!("dreamer_asleep", x = self.hitbox.0, y = self.hitbox.1, sw = 32, fps = fps::SLOW);   
        } else {
            sprite!("dreamer_idle", x = self.hitbox.0, y = self.hitbox.1, sw = 32, fps = fps::SLOW);   
        }
    }
    
    // Draws thought bubble portion of Dreamer visuals
    pub fn draw_dreamer(&self) {
        sprite!("dream_bubble", x = self.hitbox.0 - 16, y = self.hitbox.1 - 50, fps = fps::SLOW);

        // Draw Dreamer's Feelings
        let mut i = 0;
        let mut y: i32 = -38;
        let mut x: i32 = 0;
        for f in self.feelings.iter() {
            if (i + 1) % 3 == 0 {
                x = 0;
                y += 17;
            } 
            sprite!(Feeling::sprite(f), x = self.hitbox.0 as i32 + x, y = self.hitbox.1 as i32 + y);
            x += 17;
            i += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
// Player area struct, contains Taps, VialSource and Clock. Used to anchor their position during tweening
pub struct PlayerArea {
    hitbox: (i32, i32, i32, i32),
    pub vial_source: VialSource,
    pub taps: Vec<SandTap>,
    pub clock: Clock,
    tween: Option<Tween<i32>>
}

impl PlayerArea {
    pub fn new(taps: u32, time: f32) -> Self {
        Self {
            hitbox: (0, 255, 255, 112), // x, y, w, h
            vial_source: VialSource::new(),
            taps: SandTap::spawn(taps),
            clock: Clock::new(time),
            tween: None,
        }
    }

    pub fn tween_area(&mut self, out: bool) {
        if out {
            for t in self.taps.iter_mut() {
                t.flow = 0;
            }
            self.tween = Some(Tween::new(self.hitbox.1).set(255).duration(40).ease(Easing::EaseInOutCirc));
        } else {
            self.tween = Some(Tween::new(self.hitbox.1).set(143).duration(40).ease(Easing::EaseInOutCirc));
        }
    }

    pub fn update(&mut self) {
        let mut clear = false;
        // If there is a tween
        if let Some(mut t) = self.tween {
            self.hitbox.1 = t.get();
            for t in self.taps.iter_mut() {
                t.anchor_y(self.hitbox.1);
            }
            self.vial_source.anchor_y(self.hitbox.1);
            self.clock.anchor_y(self.hitbox.1);
            if t.done() {
                clear = true;
            }
        }
        if clear {
            self.tween = None;
        }
    }

    pub fn draw(&self) {
        sprite!("backsplash", x = self.hitbox.0, y = self.hitbox.1);

        // Draw Vial Source
        self.vial_source.draw();

        // Draw Sand Taps
        for t in self.taps.iter() {
            t.draw();    
        }

        self.clock.draw();
    }
}

// Sand Tap class
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct SandTap {
    pub x: i32,
    pub anchor_offset: i32,
    pub y: i32,
    pub spiggot_hitbox: (i32, i32, i32, i32),
    pub handle_hitbox: (i32, i32, i32, i32),
    spiggot_hover: bool, 
    handle_hover: bool,
    pub feeling: Feeling,
    pub flow: i32,
    pub state: ObjState,
    pub vial: Option<Vial>,
    overflow_timer: u32,
}

impl SandTap {

    pub fn new(x: i32, y: i32, feeling: Feeling) -> Self {
        Self {
            x,
            anchor_offset: y,
            y: 255 + y,
            spiggot_hitbox: (x + 5, y + 255, 22, 22),
            handle_hitbox: (x + 5, y + 255, 22, 22),
            spiggot_hover: false,
            handle_hover: false,
            feeling,
            flow: 0,
            state: ObjState::Loose,
            vial: None,
            overflow_timer: 0,
        }
    }
    
    pub fn spawn(pool: u32) -> Vec<SandTap> {
        let mut t = vec![];
    
        let mut s = 0;
        for f in Feeling::all(pool) {
            let x = 43 + (21 * (5 - pool as i32)) + s * 34; // Math to center array of sprites in screen space
            let y = 32 as f64+ (1.0 - ((3.145 * (s+1) as f64/ (pool + 1) as f64) as f64).sin().powi(2)) * 8.; // Math to arc the array's y value
    
            t.push(SandTap::new(x, y as i32, f.clone()));
            s += 1;
        }
        t
    }
    
    // Called from PlayerArea to anchor relative position when UI is tweening
    pub fn anchor_y(&mut self, y: i32) {
        self.y = y + self.anchor_offset;
        self.handle_hitbox = (
            self.x,
            (self.y - 2) as i32, 
            32,
            16
        );
        self.spiggot_hitbox = (self.x + 5, self.y + 14, 22, 32);
        if let Some(v) = &mut self.vial {
            v.hitbox.1 = self.y + 18;
        }
    }

    pub fn change_flow(&mut self, mx: i32) {
        let t = (((mx.clamp(self.x, self.x+32) - self.x) / 8)).clamp(0, 3) as i32;
        self.flow = t;
        self.handle_hitbox = (
            self.x,
            (self.y - 2) as i32, 
            32,
            16
        );
    }


    pub fn update(&mut self, vial_held: bool, mx: i32, my:i32) -> i32 {
        if !vial_held {
            self.spiggot_hover = false;
            if self.hover(self.handle_hitbox, mx, my) != None {
                self.handle_hover = true;
                if let Some(v) = &mut self.vial {
                    v.hovered = false;
                }
            } else if let Some(v) = &mut self.vial {
                self.handle_hover = false;
                if v.hover(v.hitbox, mx, my) != None {
                    v.hovered = true;
                } else {
                    v.hovered = false;
                }
            } else {
                self.handle_hover = false;
            }
        } else {
            self.handle_hover = false;
            if let Some(v) = &mut self.vial {
                v.hovered = false;
            }
            if self.vial == None && self.hover(self.spiggot_hitbox, mx, my) != None {
                self.spiggot_hover = true;
            } else {
                self.spiggot_hover = false;
            }
        }
    
        if let Some(v) = &mut self.vial {
            v.update(0, 0);
            if self.flow > 0 {
                let f = self.flow.clone();
                v.filling = true;
                // Return value from Vial's fill method - 0 normally, >0 when overflowing
                v.fill(f, self.feeling.clone())
            } else {
                v.filling = false;
                0
            }
        } else {
            if sys::tick() as u32 > self.overflow_timer {
                self.overflow_timer = sys::tick() as u32 + 60;
                -3 * self.flow
            } else {
                0
            }
        }

    }

    pub fn draw(&self) {
        match &self.vial {
            Some(v) => sprite!("vial_bg", x = v.hitbox.0, y = v.hitbox.1, color = 0xffffff80),
            None => ()
        }

        // Draw pouring sand
        let h: i32;
        if self.vial != None {
            h = 23
        } else {
            h = 215 - self.y;
        }
        let f: u32 = self.feeling.color();
        let c: u32 = 0xccdee3ff; // off white
        match self.flow {
            1 => {
                rect!(w = 3, h = h, x = self.x + 15, y = self.y + 26, color = c);
                rect!(w = 1, h = h, x = self.x + 16, y = self.y + 26, color = f);
                rect!(w = 5, h = 2, x = self.x + 14, y = self.y + 25 + h, color = c);
                rect!(w = 3, h = 1, x = self.x + 15, y = self.y + 25 + h, color = f);
            },
            2 => {
                rect!(w = 4, h = h, x = self.x + 14, y = self.y + 26, color = c);
                rect!(w = 2, h = h, x = self.x + 15, y = self.y + 26, color = f);
                rect!(w = 6, h = 2, x = self.x + 13, y = self.y + 25 + h, color = c);
                rect!(w = 4, h = 1, x = self.x + 14, y = self.y + 25 + h, color = f);
            },
            3 => {
                rect!(w = 5, h = h, x = self.x + 14, y = self.y + 26, color = c);
                rect!(w = 3, h = h, x = self.x + 15, y = self.y + 26, color = f);
                rect!(w = 7, h = 2, x = self.x + 13, y = self.y + 25 + h, color = c);
                rect!(w = 5, h = 1, x = self.x + 14, y = self.y + 25 + h, color = f);
            },
            _ => ()
        }

        // Draw static elements
        sprite!("tap_label", x = self.x + 6, y = self.y - 16);
        sprite!(self.feeling.sprite(), x = self.x + 8, y = self.y - 14);
        if self.spiggot_hover {
            sprite!("tap_spigot_hover", x = self.x, y = self.y);
        } else {
            sprite!("tap_spigot", x = self.x, y = self.y);
        }
        if self.handle_hover || self.state == ObjState::Held {
            sprite!("tap_handle_hover", x = (self.x - 4 + (self.flow as i32 * 8)) as i32, y = self.handle_hitbox.1);
        } else {
            sprite!("tap_handle", x = (self.x - 4 + (self.flow as i32 * 8)) as i32, y = self.handle_hitbox.1);
        }
        match &self.vial {
            Some(v) => v.draw(),
            None => ()
        }
        //rect!(w = self.spiggot_hitbox.2, h = self.spiggot_hitbox.3, x = self.spiggot_hitbox.0, y = self.spiggot_hitbox.1);
        //rect!(w = self.handle_hitbox.2, h = self.handle_hitbox.3, x = self.handle_hitbox.0, y = self.handle_hitbox.1, color = 0x000000ff);
    }


}


// Vial class
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Vial {
    pub hitbox: (i32, i32, i32, i32),
    pub state: ObjState,
    pub contents: Vec<(Feeling, f32)>,
    pub filling: bool,
    pub overflow: (Feeling, u32),
    overflow_timer: u32,
    pub hovered: bool,
}

impl Vial {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            hitbox: (x, y, 12, 32),
            state: ObjState::Loose,
            contents: vec![],
            filling: false,
            overflow: (Feeling::Wonder, 0),
            overflow_timer: 0,
            hovered: false,
        }
    }

    pub fn contents_sum(&self) -> i32 {
        let s: f32 = self.contents.iter()
            .map(|&(_, fill)| fill)
            .sum();
        s as i32
    }

    // Function to fill vial with sand from SandTaps
    pub fn fill(&mut self, fl: i32, fe: Feeling) -> i32 {
        // Vial has space to fill
        if self.contents_sum() < 31 {
            self.overflow = (fe.clone(), 0);
            let mut f: f32 = 0.;
            match fl {
                1 => f = 0.01,
                2 => f = 0.05,
                3 => f = 0.1,
                _ => () 
            }
            
            // If top layer is the same Feeling continue adding to it
            if self.contents.last().is_some_and(|c| c.0 == fe.clone()) {
                let l = self.contents.pop();
                match l {
                    Some(mut c) => {
                        if c.0 == fe {
                            c.1 += f;
                        }
                        self.contents.push(c);
                    },
                    None => ()
                }
            } 
            // Otherwise start a new layer
            else {
                self.contents.push((fe.clone(), f.clone()));
            }
            0
        } 
        // Attached Vial is overflowing
        else {
            self.overflow = (fe.clone(), fl as u32);
            if sys::tick() as u32 > self.overflow_timer  {
                self.overflow_timer = sys::tick() as u32 + 60;
                -3 * fl // return flow amount to deduct from spillage_score
            } else {
                0
            }
        }
    }


    pub fn update(&mut self, mx: i32, my: i32) {
        self.overflow = (Feeling::Wonder, 0);
        if self.state == ObjState::Held {
            self.hitbox.0 = mx - 6;
            self.hitbox.1 = my - 8;
        }
        if self.state != ObjState::Attached && self.hover(self.hitbox, mx, my) != None{
            self.hovered = true;
        } else if self.state != ObjState::Attached {
            self.hovered = false;
        }
    }

    pub fn draw(&self) {
        if self.state != ObjState::Attached {
            sprite!("vial_bg", x = self.hitbox.0, y = self.hitbox.1, color = 0xffffff80);
        }
        
        
        let mut fill_i: i32 = 0;
        let c: u32 = 0xccdee3ff; // off white
        for c in &self.contents {
            rect!(w = 8, h = c.1 as i32, x = self.hitbox.0 + 4, y = self.hitbox.1 + 32 - c.1 as i32 - fill_i , color = c.0.color());
            fill_i += c.1 as i32;
        }
        if self.filling {
            rect!(w = 8, h = 1, x = self.hitbox.0 + 4, y = self.hitbox.1 + 31 - fill_i , color = c);
        }
        
        if self.state != ObjState::Attached {
            if self.hovered {
                sprite!("vial_hover", x = self.hitbox.0, y = self.hitbox.1)
            } else {
                sprite!("vial", x = self.hitbox.0, y = self.hitbox.1)
            }
        }
        else {
            if self.hovered {
                sprite!("vial_attached_hover", x = self.hitbox.0, y = self.hitbox.1)
            } else {
                sprite!("vial_attached", x = self.hitbox.0, y = self.hitbox.1)
            }
        }
        if self.overflow.1 > 0 {
            // Streams
            rect!(w = 3, h = 240 - self.hitbox.1, x = self.hitbox.0 + 1, y = self.hitbox.1+1, color = c);
            rect!(w = 3, h = 240 - self.hitbox.1, x = self.hitbox.0 + 12, y = self.hitbox.1+1, color = c);
            rect!(w = 1, h = 239 - self.hitbox.1, x = self.hitbox.0 + 2, y = self.hitbox.1 + 2, color = self.overflow.0.color());
            rect!(w = 1, h = 239 - self.hitbox.1, x = self.hitbox.0 + 13, y = self.hitbox.1 + 2, color = self.overflow.0.color());
            // Piles
            rect!(w = 5, h = 2, x = self.hitbox.0, y = 240, color = c);
            rect!(w = 5, h = 2, x = self.hitbox.0 + 11, y = 240, color = c);
            rect!(w = 3, h = 1, x = self.hitbox.0 + 1, y = 240, color = self.overflow.0.color());
            rect!(w = 3, h = 1, x = self.hitbox.0 + 12, y = 240, color = self.overflow.0.color());
            // Top
            rect!(w = 12, h = 2, x = self.hitbox.0 + 2, y = self.hitbox.1 + 1, color = self.overflow.0.color());
        }
    }

}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct VialSource {
    pub vials: i32,
    pub rack_hitbox: (i32, i32, i32, i32),
    pub rack_hover: bool,
    pub trash_hitbox: (i32, i32, i32, i32),
    pub trash_hover: bool,

}

impl VialSource {

    pub fn new() -> Self {
        Self {
            vials: 3,
            rack_hitbox: (8, 327, 32, 32), // x, y, w, h
            rack_hover: false,
            trash_hitbox: (215, 327, 32, 32),
            trash_hover: false,
        }
    }

    // Called from PlayerArea to anchor relative position when UI is tweening
    pub fn anchor_y(&mut self, y: i32) {
        self.rack_hitbox.1 = y + 60;
        self.trash_hitbox.1 = y + 74;
    }

    pub fn update(&mut self, mx: i32, my: i32, held_vial: bool) {
        // Mouse over rack hitbox
        if let Some(_) = self.hover(self.rack_hitbox, mx, my) {
            if !held_vial && !self.rack_hover {
                self.rack_hover = true;
            } else if held_vial && self.rack_hover {
                self.rack_hover = false;
            }
            
        } else {
            self.rack_hover = false;
        }
        // Mouse over rack hitbox
        if let Some(_) = self.hover(self.trash_hitbox, mx, my) {
            if held_vial && !self.trash_hover {
                self.trash_hover = true;
            }
            else if !held_vial && self.trash_hover{
                self.trash_hover = false;
            }
        } else {
            self.trash_hover = false;
        }
    }

    pub fn draw(&self) {
        match self.vials {
            0 => {
                sprite!("0rack_idle", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
            },
            1 => {
                if self.rack_hover {
                    sprite!("1rack_hover", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
                } else {
                    sprite!("1rack_idle", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
                }
            },
            2 => {
                if self.rack_hover {
                    sprite!("2rack_hover", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
                } else {
                    sprite!("2rack_idle", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
                }
            },
            3 => {
                if self.rack_hover {
                    sprite!("3rack_hover", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
                } else {
                    sprite!("3rack_idle", x = self.rack_hitbox.0, y = self.rack_hitbox.1);
                }
            },
            _ => ()
        }

        if self.trash_hover {
            sprite!("trash_hover", x = self.trash_hitbox.0, y = self.trash_hitbox.1, fps = fps::FAST);
        } else {
            sprite!("trash", x = self.trash_hitbox.0, y = self.trash_hitbox.1);
        }
        
    }

}


#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum ObjState {
    Loose,
    Held, 
    Attached,
    Pouring
}


#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Clock {
    hitbox: (i32, i32, i32, i32),
    time: f32,
    limit: f32,
    pub running: bool,
    intrvl: bool,
    rot: f32,
}

impl Clock { 
    pub fn new(l: f32) -> Self {
        Self {
            hitbox: (8, 282, 32, 32),
            time: 0.,
            limit: l,
            running: true,
            intrvl: false,
            rot: 45.,
        }
    }
    
    pub fn score_remaining(&self) -> i32 {
        (self.limit - self.time) as i32 * 3
    }

    // Called from PlayerArea to anchor relative position when UI is tweening
    pub fn anchor_y(&mut self, y: i32) {
        self.hitbox.1 = y + 25;
    }

    pub fn update(&mut self, dreamers: & Vec<Dreamer>) {
        self.time += 0.015;
        
        if (self.limit - self.time).floor() % 5. == 0. {
            if !self.intrvl {
               self.intrvl = true;
            }
        } else {
            self.intrvl = false;
        }

        let mut done = true;
        for d in dreamers {
            if !d.sleeping {
                done = false;
            }
        }
        self.rot = (self.time / self.limit) * -270.;

        if self.time > self.limit as f32 
            || done {
            self.running = false;
        }
    }

    pub fn draw(&self) {
        sprite!("clock", x = self.hitbox.0, y = self.hitbox.1, rotate = self.rot, fps = fps::MEDIUM);
        sprite!("clock_frame", x = self.hitbox.0, y = self.hitbox.1);
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
// Struct contains cloud vignette tweening, floating scoring text, and night counter
pub struct GameUI {
    pub floating_text: Vec<FloatingText>,
    score: i32,
    pub night_count: u32,
    vignette_tween: Option<(Tween<i32>, Tween<i32>, Tween<i32>)>, // Top, bottom, night counter
    vignette_pos: (i32, i32, i32), // top y, bottom y, night counter y
}
impl GameUI {
    pub fn new() -> Self {
        Self {
            floating_text: vec!(),
            score: 50,
            night_count: 0,
            vignette_tween: None,
            vignette_pos: (-33, 0, -33),
        }
    }
    
    // Tweens clouds in our out based on to_menu bool
    pub fn tween_vignette(&mut self, to_menu: bool) {
        if to_menu {
            self.vignette_tween = Some(
                (Tween::new(self.vignette_pos.0).set(-33).duration(60).ease(Easing::EaseInOutCirc),
                Tween::new(self.vignette_pos.1).set(0).duration(60).ease(Easing::EaseInOutCirc),
                Tween::new(self.vignette_pos.2).set(-33).duration(60).ease(Easing::EaseInOutCirc))
            );
        } else {
            self.vignette_tween = Some(
                (Tween::new(self.vignette_pos.0).set(-132).duration(60).ease(Easing::EaseInOutCirc),
                Tween::new(self.vignette_pos.1).set(60).duration(60).ease(Easing::EaseInOutCirc),
                Tween::new(self.vignette_pos.2).set(4).duration(60).ease(Easing::EaseInOutCirc))
            );
        }
    }

    pub fn update(&mut self, score: i32) {
        self.score = score;
        
        // Update floating text
        self.floating_text.retain_mut(|f| {
            // If floating text's lifetime hasn't expired
            if f.timer < f.lifetime {
                f.update();
                true // Retain ownership
            } else {
                false // Drop ownership
            }
        });

        // Update cloud vignette's based on active tween
        let mut clear = false;
        // If there is a tween
        if let Some(mut t) = self.vignette_tween {
            self.vignette_pos.0 = t.0.get();
            self.vignette_pos.1 = t.1.get();
            self.vignette_pos.2 = t.2.get();
            if t.0.done() && t.1.done() && t.2.done() {
                clear = true;
            }
        }
        // Drop tween ownership
        if clear {
            self.vignette_tween = None;
        }
    }
    
    // Draws cloud vignette and night counter
    pub fn draw_bottom(&self) {
        sprite!("cloud_vignette", x = 0, y = self.vignette_pos.0);
        sprite!("cloud_vignette", x = 0, y = self.vignette_pos.1, rotate = 180);
        sprite!("night_counter", x = 8, y = self.vignette_pos.2);
        text!("NIGHT", x = 12, y = self.vignette_pos.2 + 13, font = Font::M);
        text!("{:?}/7", self.night_count; x = 17, y = self.vignette_pos.2 + 21, font = Font::S);
        
    }
    
    // Draws floating text
    pub fn draw_top(&self) {
        for t in self.floating_text.iter() {
            t.draw();
        }
    }
}

// Draws textboxes and buttons
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct MenuUI {
    pub start_button: UIButton,
    pub how_to_button: UIButton,
    pub game_button: UIButton,
    pub continue_button: UIButton,
    pub quit_button: UIButton,
    pub pause_button: UIButton,
    pub resume_button: UIButton,
}
impl MenuUI {
    pub fn new() -> Self {
        Self {
            start_button: UIButton::new("play",(102, 165, 50, 15)),
            how_to_button: UIButton::new("how to",(182, 165, 50, 15)),
            game_button: UIButton::new("begin",(102, 126, 50, 15)),
            continue_button: UIButton::new("continue",(132, 165, 50, 15)),
            quit_button: UIButton::new("quit",(72, 165, 50, 15)),
            pause_button: UIButton::new("",(235, 5, 15, 15)),
            resume_button: UIButton::new("resume",(132, 165, 50, 15)),
        }
    }

    fn score_dots(title: &str, score: i32) -> String {
        let l: usize;
        if title.chars().next().map_or(false, |c| c.is_uppercase()) {
            l = 13 - title.len() - score.to_string().len();
        }
        else {
            l = 20 - title.len() - score.to_string().len();
        }
        let mut d = "".to_string();
        for _i in 0..l {
            d.push('.');
        }
        d
    }

    pub fn draw(&mut self, state: &GameState) {
        match state.play_state {
            PlayState::MainMenu => {
                sprite!("title", x = 6, y = 64, sw = 246, fps = fps::MEDIUM);
                text!("made with", x = 90, y = 121, font = Font::L, color = 0xccdee3ff);
                text!("made with", x = 90, y = 120, font = Font::L, color = 0x323b42ff);
                text!("Turbo!", x = 105, y = 131, font = Font::L, color = 0xccdee3ff);
                text!("Turbo!", x = 105, y = 130, font = Font::L, color = 0x323b42ff);
                self.start_button.draw();
                
            },
            PlayState::Prelude => {
                sprite!("bed_time", x = 34, y = 8, sw = 188, fps = fps::MEDIUM);
                Util::nine_slice("9slice", 5, 180, 150, 37, 75);
                text!("The sleepy town below yearns for", x = 45, y = 83);
                text!("sweet dreams. ", x = 45, y = 91);
                text!("Concoct sleep by pouring sand", x = 45, y = 99);
                text!("into vials, and give one to", x = 45, y = 107);
                text!("each dreamer before daybreak.", x = 45, y = 115);
                
                Util::nine_slice("9slice", 5, 78, 78, 47, 123);
                sprite!("gif_dreamers", x = 50, y = 126, sw = 72, fps = fps::MEDIUM);
                text!("Hover over awake", x = 46, y = 202, font = Font::S);
                text!("dreamers to see", x = 48, y = 208, font = Font::S);
                text!("their feelings", x = 50, y = 214, font = Font::S);

                Util::nine_slice("9slice", 5, 78, 78, 129, 123);
                sprite!("gif_vial", x = 132, y = 126, sw = 72, fps = fps::MEDIUM);
                text!("Attach vials to", x = 132, y = 202, font = Font::S);
                text!("taps to fill,", x = 136, y = 208, font = Font::S);
                text!("give to dreamers", x = 129, y = 214, font = Font::S);
                
                
                self.continue_button.draw();
            },
            PlayState::GameMenu => {
                sprite!("bed_time", x = 34, y = 8, sw = 188, fps = fps::MEDIUM);
                Util::nine_slice("9slice", 5, 80, 55, 87, 65);
                text!("NIGHT {:?}", state.round; x = 94, y = 72, font = Font::L);
                text!("{:?} dreamers", state.dreamers.len(); x = 99, y = 84);
                text!("{:?} feelings", match state.round { 1 => 3, 2 => 4, _ => 5 };  x = 99, y = 94);
                text!("{:?} seconds", state.player.clock.limit as i32;  x = 99, y = 104);
                self.game_button.draw();
            },
            PlayState::Game => {
                self.pause_button.draw();
                if self.pause_button.hovered {
                    sprite!("pause_icon_hover", x = 235, y = 5);
                } else {
                    sprite!("pause_icon", x = 235, y = 5);
                }
            },
            PlayState::Scoring => {
                // Screen title
                sprite!("score", x = 61, y = 8, sw = 133, fps = fps::MEDIUM);
                
                
                // 9 Slice box
                Util::nine_slice("9slice", 5, 120, 95, 67, 65);
                // Box title
                text!("NIGHT {:?}/7", state.round; x = 74, y = 72, font = Font::L);
                
                // Dreamer score
                let s = format!("{}{}{}", "dreamers", game_structs::MenuUI::score_dots("dreamers", state.dreamer_score), state.dreamer_score);
                text!(s.as_str(), x = 77, y = 85);
                // Time score
                let mut s = format!("{}{}{}", "time", game_structs::MenuUI::score_dots("time", state.time_score), state.time_score);
                text!(s.as_str(), x = 77, y = 95);
                // Spill score
                s = format!("{}{}{}", "spillage", game_structs::MenuUI::score_dots("spillage", state.spillage_score as i32), state.spillage_score as i32);
                text!(s.as_str(), x = 77, y = 105);
                
                //Divder line
                rect!(x = 72, w = 106, y = 116, h = 1);
                
                // Total night score
                let mut n = state.dreamer_score + state.time_score + state.spillage_score as i32;
                if n < 0 { 
                    n = 0;
                }
                let s = format!("{}{}{}{}", "night ", state.round, game_structs::MenuUI::score_dots(format!("{}{}", "night ", state.round).as_str(), n), n);
                text!(s.as_str(), x = 77, y = 120);
                // Previous nights score
                let s = format!("{}{}{}", "prev nights", game_structs::MenuUI::score_dots("prev nights", state.game_score - n), state.game_score - n);
                text!(s.as_str(), x = 77, y = 130);
                // Game total score
                let s = format!("{}{}{}", "TOTAL", game_structs::MenuUI::score_dots("TOTAL", state.game_score), state.game_score);
                text!(s.as_str(), x = 74, y = 142, font = Font::L);
                
                // Draw buttons
                // Only draw continue button if round number doesn't exceed game length
                if state.round <= 6 {
                    self.quit_button.hitbox.0 = 75;
                    self.continue_button.draw();
                } else {
                    self.quit_button.hitbox.0 = 95;
                }
                self.quit_button.draw();
            },
            PlayState::Paused =>  {
                rect!(w = 255, h = 255, x = 0, y = 0, color = 0x000000BF);
                sprite!("paused", x = 50, y = 64, sw = 157, fps = fps::MEDIUM);
                self.quit_button.draw();
                self.resume_button.draw();
            }
        }

    }
}
