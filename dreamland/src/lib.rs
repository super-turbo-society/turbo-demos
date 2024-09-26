mod game_structs;
use game_structs::*;
mod util;
use util::*;
mod mouse_fn;
use mouse_fn::*;

turbo::cfg! {r#"
    name = "dreamland"
    version = "1.0.0"
    author = "jauntybot"
    description = "help lull a sleepy town to dreamland."
    [settings]
    resolution = [255, 255]
"#}

turbo::init! {
    struct GameState {
        frame: u32,
        play_state: PlayState, // controls state machine
        menu: MenuUI, // draws menus based on playstate
        player: PlayerArea, // owns and anchors VialSource, Trash, Clock, and SandTaps
        round: u32, // Count of rounds played
        dreamers: Vec<Dreamer>,
        min_awake: i32,
        wake_timer: i32, // How long dreamers are awake
        wake_intrvl: i32, // Timer for waking new dreamers
        vials: Vec<Vial>,
        held_vial: Option<Vial>,
        ui: GameUI, // Draws clouds, night counter and floating points text
        game_score: i32,
        dreamer_score: i32,
        time_score: i32,
        spillage_score: i32,
    } = {
        Self::new(PlayState::MainMenu, 0, 0)
    }
}

// Implement GameState outside of init! in order to reload/rebuild GameState between rounds
impl GameState {
    pub fn new(state: PlayState, round: u32, score: i32) -> Self {
        // Variables used to build the Game State
        let dreamers;
        let feelings_count;
        let min_awake;
        let wake_intrvl;
        let awake_timer;
        // Set variables based on current round
        match round {
            1 => {
                dreamers = 4;
                feelings_count = 3;
                min_awake = 1;
                wake_intrvl = 30;
                awake_timer = 60;
            }
            2 => {
                dreamers = 5;
                feelings_count = 3;
                min_awake = 2;
                wake_intrvl = 30;
                awake_timer = 55;
            }
            3 => {
                dreamers = 5;
                feelings_count = 4;
                min_awake = 2;
                wake_intrvl = 30;
                awake_timer = 55;
            }
            4 => {
                dreamers = 6;
                feelings_count = 4;
                min_awake = 2;
                wake_intrvl = 25;
                awake_timer = 50;
            }
            5 => {
                dreamers = 6;
                feelings_count = 5;
                min_awake = 2;
                wake_intrvl = 25;
                awake_timer = 50;
            }
            6 => {
                dreamers = 7;
                feelings_count = 5;
                min_awake = 3;
                wake_intrvl = 25;
                awake_timer = 45;
            }
            _ => {
                dreamers = 8;
                feelings_count = 5;
                min_awake = 3;
                wake_intrvl = 25;
                awake_timer = 45;
            }
        }
        // Set time to minimum required for each dreamer to expire on it's own
        let time = (awake_timer + (dreamers - min_awake) * awake_timer - wake_intrvl - (dreamers - min_awake - 1) * wake_intrvl) as f32;
        
        // Initialize game state with determined variables
        Self {
            frame: 0,
            play_state: state,
            menu: MenuUI::new(),
            round,
            player: PlayerArea::new(feelings_count, time),
            dreamers: Dreamer::spawn(dreamers, feelings_count, awake_timer),
            min_awake,
            wake_timer: 0,
            vials: vec![],
            held_vial: None,
            ui: GameUI::new(),
            game_score: score,
            dreamer_score: 0,
            time_score: 0,
            spillage_score: 0,
            wake_intrvl,
        }
    }

    // Switches state machine and transitions the game scene
    pub fn switch_play_states(&mut self, play_state: PlayState) {
        match play_state {
            PlayState::MainMenu => { // Reload GameState to 0
                *self = GameState::new(PlayState::MainMenu, 0, 0);
            },
            PlayState::Prelude => { // Move continue button
                self.menu.continue_button.hitbox = (102, 230, 50, 15);
            },
            PlayState::GameMenu => { // Rebuild GameState to next round, tween PlayerArea in
                *self = GameState::new(PlayState::MainMenu, self.round + 1, self.game_score);
                self.player.tween_area(false);
            },
            PlayState::Game => { // Tween clouds out, update night counter, reset wake timer
                if self.play_state == PlayState::GameMenu {
                    self.ui.night_count = self.round;
                    self.wake_timer = self.frame as i32 + self.wake_intrvl * 60;
                }
                self.ui.tween_vignette(false);
            },
            PlayState::Scoring => { // Move continue button, tween clouds in and PlayerArea out, reset some of GameState, calculate score
                self.menu.continue_button.hitbox = (132, 165, 50, 15);
                self.vials = Vec::new();
                self.time_score = self.player.clock.score_remaining();
                self.game_score += self.dreamer_score + self.time_score + self.spillage_score;
                if self.game_score < 0 {
                    self.game_score = 0;
                }
                self.ui.tween_vignette(true);
                self.player.tween_area(true);
            },
            PlayState::Paused => { // Tween clouds in
                self.ui.tween_vignette(true);
            }
        }
        self.play_state = play_state; // Set new PlayState
    }
}

// smol Turbo Update loop
turbo::go! ({
    let mut state = GameState::load();
    
    // RUN GAME
    game(&mut state);
    
    // DRAW GAME
    draw(&mut state);

    state.frame += 1;
    state.save();
});


fn game(state: &mut GameState) {
    // Player input
    let m = mouse(0);
    let [mx, my] = m.position;
    //Update Player Area
    state.player.update();
    
    // Implement state machine
    match state.play_state {
        // Main menu
        PlayState::MainMenu => { // Button goes to Prelude
            if let Some(_) = state.menu.start_button.hover(state.menu.start_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::Prelude);    
                }
            }
        },
        // Tutorial
        PlayState::Prelude => { // Button goes to GameMenu
            if let Some(_b) = state.menu.continue_button.hover(state.menu.continue_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::GameMenu);    
                }
            } 
        },
        // Game Menu
        PlayState::GameMenu => { // Button goes to Game
            if let Some(_) = state.menu.game_button.hover(state.menu.game_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::Game);    
                }
            }
        },
        // Game state
        PlayState::Game => {
            // Pause button goes to Pause
            if let Some(_) = state.menu.pause_button.hover(state.menu.pause_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::Paused);    
                }
            }
            // Player input
            player_input(state);
            // Update clock
            if state.player.clock.running {
                state.player.clock.update(&state.dreamers);
            } else {
                state.switch_play_states(PlayState::Scoring); // Times up! Go to Scoring
            }
            
    
            // Update dreamers and put new dreamers to sleep
            let mut awake = 0;
            for d in state.dreamers.iter_mut() {
                // Update
                d.update(&mut state.ui, mx, my);
                // Set dreamers awake
                if !d.sleeping && !d.awake {
                    // Below minimum awake dreamer threshold
                    if awake < state.min_awake {
                        d.awake = true;
                        awake += 1;
                        state.wake_timer = state.frame as i32 + state.wake_intrvl * 60; // Reset wake timer
                    }
                    //Exceeding wake timer
                    else if state.frame as i32 > state.wake_timer {
                        d.awake = true;
                        awake += 1;
                        state.wake_timer = state.frame as i32 + state.wake_intrvl * 60; // Reset wake timer
                    }
                } else if d.awake {
                    awake += 1;
                }
            }
    
            // Update held vial
            if let Some(v) = &mut state.held_vial {
                v.update(mx, my);
            } else {
                // Update loose Vials
                state.vials.retain_mut(|v| {
                    v.update(mx, my);
                    true
                });
            }
            // Update SandTaps
            state.player.taps.retain_mut(|t| {
                let spills = t.update(state.held_vial != None, mx, my);
                if spills < 0 {
                    state.spillage_score += spills;
                    state.ui.floating_text.push(FloatingText::new(&spills.to_string(), t.x + 20, t.y + 48, 0));
                }
                if t.state == ObjState::Held {
                    t.change_flow(mx);
                }
                true
            });
            // Update Vial Source
            state.player.vial_source.update(mx, my, state.held_vial != None);
            
        },
        PlayState::Scoring => {
            // Update scoring menu button input
            if state.round <= 6 {
                if let Some(_b) = state.menu.continue_button.hover(state.menu.continue_button.hitbox, mx, my) {
                    if m.left.just_pressed() {
                        state.switch_play_states(PlayState::GameMenu);    
                    }
                    
                } 
            }
            if let Some(_b) = state.menu.quit_button.hover(state.menu.quit_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::MainMenu);    
                }
            }
        },
        PlayState::Paused => {
            // Update paused menu button input
            if let Some(_b) = state.menu.resume_button.hover(state.menu.resume_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::Game);    
                }
            }
            if let Some(_b) = state.menu.quit_button.hover(state.menu.quit_button.hitbox, mx, my) {
                if m.left.just_pressed() {
                    state.switch_play_states(PlayState::MainMenu);    
                }
            }
        }
    }
    
    // Independent of state machine
    // Update Game UI
    state.ui.update(state.dreamer_score);
}

fn player_input(state: &mut GameState) -> Mouse<Button> {
    // Mouse input handler
    let m = mouse(0);
    let [mx, my] = m.position;

    // Player clicks down
    if m.left.just_pressed() {
        // Check if clicked on loose Vial
        for (i, vial) in state.vials.iter_mut().enumerate() {
            // Check if mouse is over Vial hitbox
            if let Some(v) = vial.hover(vial.hitbox, mx, my) {
                    // Start holding Vial
                    v.state = ObjState::Held;
                    state.held_vial = Some(v.clone());
                    state.vials.remove(i);
                    break;
                }
            }
        // Check if input should continue to be parsed - not clicked on vial
        if state.held_vial == None {
            // Check if clicked on SandTap
            for tap in state.player.taps.iter_mut() {
                // Clicked on lever
                if let Some(t) = tap.hover(tap.handle_hitbox, mx, my) {
                    t.state = ObjState::Held;
                    break;
                }
                // Clicked on attached Vial
                else if let Some(t) = tap.hover(tap.spiggot_hitbox, mx, my) {
                    if let Some(v) = &t.vial {
                        let mut cv = v.clone();
                        cv.state = ObjState::Held;
                        for c in cv.contents.iter_mut() {
                            c.1 = c.1.floor();
                        }
                        cv.filling = false;
                        state.held_vial = Some(cv);
                        t.vial = None;
                    }
                }                
            }
            // Check if clicked on Vial Source
            if let Some(vs) = state.player.vial_source.hover(state.player.vial_source.rack_hitbox, mx, my) {
                if vs.vials > 0 {
                    state.player.vial_source.vials -= 1;
                    let mut v = Vial::new(mx, my);
                    v.state = ObjState::Held;
                    state.held_vial = Some(v.clone());
                }
            }
        }
    }
    
    // Player releases click
    if m.left.just_released() {
        // If the player is releasing a Vial
        let mut retain = true;
        if let Some(v) = &mut state.held_vial {
            
            // Check if released Vial onto Tap
            for tap in state.player.taps.iter_mut() {
                if let Some(t) = tap.hover(tap.spiggot_hitbox, mx, my) {
                    // Check if Tap is empty
                    if t.vial == None {
                        // Clone vial to transfer ownership
                        let mut cv = v.clone();
                        cv.state = ObjState::Attached;
                        cv.hitbox.0 = t.x + 8;
                        cv.hitbox.1 = t.y + 18;
                        t.vial = Some(cv);

                        retain = false; // Flag held_vial's ownership as transferred
                    }

                }
            }
            
            // Check if released Vial onto a Dreamer
            for dreamer in state.dreamers.iter_mut() {
                if let Some(d) = dreamer.hover(dreamer.hitbox, mx, my) {
                    if d.awake && v.state == ObjState::Held {
                        // Put Dreamer to sleep, returns satisfaction score
                        let s = d.sleep(v.clone());
                        state.dreamer_score += s;

                        // Assign color based on sign of s -- TODO move to FloatingText impl
                        let c: u32;
                        let o: &str;
                        if s > 0 { 
                            c = 0xffffffff; 
                            o = "+";
                        } 
                        else { 
                            c = 0xac3232ff; 
                            o = "";
                        }
                        let t: &str = &format!("{}{}", o, &s);
                        // Create FloatingText
                        state.ui.floating_text.push(FloatingText::new(t, d.hitbox.0 + 28, d.hitbox.1, c));
                        // Refresh used Vial
                        state.player.vial_source.vials += 1;

                        retain = false; // Flag held_vial's ownership as transferred
                    }
                }
            }

            // Check if released Vial onto Trash
            if let Some(t) = state.player.vial_source.hover(state.player.vial_source.trash_hitbox, mx, my) {
                retain = false; // Flag held_vial's ownership as transferred
                t.vials += 1;
                let mut s = 0.;
                if v.contents.len() > 0 {
                    s = -v.contents[0].1;
                }
                state.spillage_score -= s as i32;
                state.ui.floating_text.push(FloatingText::new(&s.to_string(), t.trash_hitbox.0, t.trash_hitbox.1, 0));
            }
        }
        // Release held vial
        if let Some(mut v) = state.held_vial.take() {
            // If vial's ownership was not flagged, it needs to be retained
            if retain {
                v.state = ObjState::Loose;
                state.vials.push(v);
            } 
            state.held_vial = None;
        }
        
        // If the player is releasing a SandTap handle
        for t in state.player.taps.iter_mut() {
            if t.state == ObjState::Held {
                t.state = ObjState::Loose;
            }
        }
    }
    m
}

fn draw(state: &mut GameState) {
    // Draw background
    clear(0x00000ff);  
    sprite!("bg");
    
    // Draw dreamers
    for d in state.dreamers.iter() {
        d.draw_home();
    }
    
    // Draw bottom UI elements - clouds
    state.ui.draw_bottom();
    
    // Draw dreamer thought bubbles
    for d in state.dreamers.iter() {
        if d.selected == true {
            d.draw_dreamer();
        }
    }
    
    // Draw top UI elements - PlayerArea, floating text, or menu above all else
    state.player.draw();
    state.ui.draw_top();
    state.menu.draw(&state.clone());

    // Draw Vials
    if let Some(v) = &state.held_vial {
        v.draw();
    }
    for v in state.vials.iter() {
        v.draw();
    }
}

// State machine enum
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum PlayState {
    MainMenu,
    Prelude,
    GameMenu,
    Game,
    Scoring,
    Paused
}