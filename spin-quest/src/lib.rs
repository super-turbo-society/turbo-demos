turbo::cfg! {r#"
    name = "SpinQuest"
    version = "1.0.0"
    author = "Turbo"
    description = "A multiplayer party game with a spinner"
    [settings]
    resolution = [144, 256]
"#}

#[derive(Debug, Copy, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
struct Timer {
    duration: u32,
    elapsed: u32,
}

turbo::init! {
    struct GameState {
        tick: u32,
        turn_phase: enum TurnPhase {
            PlayerAction,
            TurnEnding(u32),
            TurnStarting(u32, (i32, i32)),
            SpecialEvent(enum SpecialEvent {
                CoinRain,
            })
        },
        items: Vec<struct Item {
            kind: #[derive(Copy)]
                enum ItemKind {
                    Coin,
                    SmallKey,
                    BossKey,
                    Gem,
                    Diamond,
                },
            key: i32,
            x: i32,
            y: i32,
            state: enum ItemState {
                Available,
                // Hidden,
                Collecting(u32, bool), // timer, did_collect
            }
        }>,
        actors: Vec<struct Actor {
            is_active: bool,
            id: u32,
            name: String,
            ty: enum ActorType {
                NPC,
                Monster(enum MonsterKind {
                    GoblinGrunt,
                    BossGoblin,
                    Pudding,
                }),
                Player(enum PlayerKind {
                    Human,
                    Computer,
                }),
            },
            gold: u32,
            inventory: Vec<struct InventoryItem {
                kind: ItemKind,
                qty: u32,
            }>,
            hp: [u8; 2],
            ep: [u8; 2],
            dir: Direction,
            x: i32,
            y: i32,
            move_state: enum ActorMoveState {
                Waiting,
                Chilling,
                Ready,
                Spinning(struct Spin {
                    started_at: u32,
                    angle: i32,
                    speed: f32,
                    decel: f32,
                    value: Option<u32>,
                }),
                Moving(struct ActorMovement {
                    range: u32,
                    origin: (i32, i32) ,
                    directions: Vec<enum MovementDirection {
                        Done(Direction),
                        Pending {
                            direction: Direction,
                            timer: Timer,
                            is_undo: bool,
                        }
                    }>
                }),
            }
        }>,
    } = {
        let state = Self {
            tick: 0,
            // turn_phase: TurnPhase::TurnStarting(60 * 2, (3 * 16, 2 * 16)),
            turn_phase: TurnPhase::PlayerAction,
            items: vec![Item {
                kind: ItemKind::Coin,
                key: 19,
                x: 0,
                y: 0,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::Coin,
                key: 19,
                x: 2 * 16,
                y: 2 * 16,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::Coin,
                key: 19,
                x: 4 * 16,
                y: 3 * 16,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::Diamond,
                key: 68,
                x: 0 * 16,
                y: 7 * 16,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::Gem,
                key: 65,
                x: 5 * 16,
                y: 5 * 16,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::SmallKey,
                key: 36,
                x: 5 * 16,
                y: 2 * 16,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::SmallKey,
                key: 36,
                x: 2 * 16,
                y: 3 * 16,
                state: ItemState::Available,
            }, Item {
                kind: ItemKind::BossKey,
                key: 39,
                x: 5 * 16,
                y: 8 * 16,
                state: ItemState::Available,
            }],
            actors: vec![Actor {
                is_active: true,
                id: rand(),
                name: "Player 1".to_string(),
                ty: ActorType::Player(PlayerKind::Human),
                gold: 0,
                inventory: vec![],
                hp: [6; 2],
                ep: [10; 2],
                dir: Direction::Down,
                x: 3 * 16,
                y: 2 * 16,
                move_state: ActorMoveState::Ready,
            }, Actor {
                is_active: false,
                id: rand(),
                name: "Player 2".to_string(),
                ty: ActorType::Player(PlayerKind::Human),
                gold: 0,
                inventory: vec![],
                hp: [6; 2],
                ep: [10; 2],
                dir: Direction::Down,
                x: 5 * 16,
                y: 3 * 16,
                move_state: ActorMoveState::Chilling,
            }, Actor {
                is_active: false,
                id: rand(),
                name: "Player 3".to_string(),
                ty: ActorType::Player(PlayerKind::Computer),
                gold: 0,
                inventory: vec![],
                hp: [6; 2],
                ep: [10; 2],
                dir: Direction::Down,
                x: 7 * 16,
                y: 5 * 16,
                move_state: ActorMoveState::Chilling,
            }, Actor {
                is_active: false,
                id: rand(),
                name: "Player 4".to_string(),
                ty: ActorType::Monster(MonsterKind::GoblinGrunt),
                gold: 0,
                inventory: vec![],
                hp: [6; 2],
                ep: [10; 2],
                dir: Direction::Down,
                x: 0 * 16,
                y: 4 * 16,
                move_state: ActorMoveState::Chilling,
            }],
        };
        // let bytes = state.try_to_vec().unwrap();
        // println!("\nSTATE SIZE = {}\n", bytes.len());
        state
    }
}

impl ActorMovement {
    fn move_in_direction(&mut self, direction: Direction) -> bool {
        if self.is_direction_pending() {
            return false;
        }
        let is_undo = self.is_undo(direction);
        let move_dir = MovementDirection::Pending {
            direction,
            timer: Timer::new(24),
            is_undo,
        };
        if is_undo {
            self.directions[0] = move_dir;
            return true;
        }
        if self.directions.len() >= self.range as usize {
            return false;
        }
        self.directions.insert(0, move_dir);
        return true;
    }
    fn is_direction_pending(&self) -> bool {
        self.directions.len() > 0 && self.directions[0].is_pending()
    }
    fn is_undo(&self, next_dir: Direction) -> bool {
        let opposite = match next_dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        if self.directions.is_empty() {
            return false;
        }
        self.directions[0].dir() == opposite
    }
}

impl MovementDirection {
    fn is_pending(&self) -> bool {
        match self {
            Self::Done(_) => false,
            _ => true,
        }
    }
    fn dir(&self) -> Direction {
        match self {
            MovementDirection::Done(direction) => *direction,
            MovementDirection::Pending { direction, .. } => *direction,
        }
    }
}

impl Timer {
    fn new(duration: u32) -> Self {
        Self {
            duration,
            elapsed: 0,
        }
    }
    fn progress(&self) -> f32 {
        self.elapsed as f32 / self.duration as f32
    }
    fn inc(&mut self) {
        self.elapsed += 1;
    }
    fn is_done(&self) -> bool {
        self.duration <= self.elapsed
    }
}

turbo::go! {
    let mut state = GameState::load();

    clear(0x6ecb62ff);

    let mut active_actor_idx = 0;
    for actor in &mut state.actors {
        if actor.is_active {
            break;
        }
        active_actor_idx += 1;
    }

    let gp = gamepad(0);
    let active_actor = &mut state.actors[active_actor_idx];
    match &mut active_actor.move_state {
        ActorMoveState::Chilling => {}
        ActorMoveState::Ready => {
            if let TurnPhase::PlayerAction = state.turn_phase {
                if gp.a.just_pressed() {
                    active_actor.move_state = ActorMoveState::Spinning(Spin {
                        started_at: state.tick,
                        angle: (rand() % 360) as i32,
                        speed: 40.,
                        decel: 1.,
                        value: None,
                    });
                }
            }
        }
        ActorMoveState::Spinning(spin) => {
            if gp.a.just_pressed() || gp.b.just_pressed() {
                if spin.decel == 1. {
                    spin.decel = (97. + (((rand() % 100) / 100) as f32) * 2.9) / 100.; // 97 - 99.9
                }
                if let Some(value) = spin.value {
                    active_actor.move_state = ActorMoveState::Moving(ActorMovement {
                        range: value,
                        origin: (active_actor.x, active_actor.y),
                        directions: vec![]
                    });
                }
            }
        }
        ActorMoveState::Waiting => {
            for item in &mut state.items {
                if let ItemState::Collecting(ref mut t, ref mut did_collect) = item.state {
                    if *t == 120 {
                        match item.kind {
                            ItemKind::Coin => {
                                active_actor.gold += 1;
                                *did_collect = true;
                            }
                            ItemKind::Gem => {
                                active_actor.gold += 10;
                                *did_collect = true;
                            }
                            ItemKind::Diamond => {
                                active_actor.gold += 100;
                                *did_collect = true;
                            }
                            ItemKind::SmallKey | ItemKind::BossKey => {
                                for x in &mut active_actor.inventory {
                                    if x.kind == item.kind {
                                        x.qty += 1;
                                        *did_collect = true;
                                    }
                                }
                                if !*did_collect && active_actor.inventory.len() < 3 {
                                    active_actor.inventory.push(InventoryItem {
                                        kind: item.kind,
                                        qty: 1,
                                    });
                                    *did_collect = true;
                                }
                            }
                            _ => {
                                turbo::println!("TODO: add collected logic for {:?}", item.kind);
                            }
                        }
                    }
                    *t -= 1;
                    if *t == 0 {
                        active_actor.move_state = ActorMoveState::Chilling;
                    }
                }
            }
            // Remove collected items
            state.items.retain_mut(|item| item.state != ItemState::Collecting(0, true));
        }
        ActorMoveState::Moving(ref mut movement) => {
            let is_end_of_range = movement.range as usize == movement.directions.len();
            if is_end_of_range && !movement.is_direction_pending() && gp.a.just_pressed() {
                let mut is_collecting = false;
                for item in &mut state.items {
                    if active_actor.x == item.x && active_actor.y == item.y {
                        match item.kind {
                            ItemKind::Coin | ItemKind::Gem | ItemKind::Diamond => {
                                item.state = ItemState::Collecting(60 * 2, false);
                                is_collecting = true;
                            }
                            k => {
                                let can_stack = active_actor.inventory.iter().any(|item| item.kind == k);
                                let has_room = active_actor.inventory.len() < 3;
                                turbo::println!("{:?} -- can_stack = {}, has_room = {}", item.kind, can_stack, has_room);
                                if can_stack || has_room {
                                    item.state = ItemState::Collecting(60 * 2, false);
                                    is_collecting = true;
                                }
                            }
                        }
                    }
                }
                active_actor.move_state = if is_collecting {
                    ActorMoveState::Waiting
                } else {
                    ActorMoveState::Chilling
                };
            } else {
                let direction = if gp.up.pressed() {
                    Some(Direction::Up)
                } else if gp.down.pressed() {
                    Some(Direction::Down)
                } else if gp.left.pressed() {
                    Some(Direction::Left)
                } else if gp.right.pressed() {
                    Some(Direction::Right)
                } else {
                    None
                };
                if let Some(direction) = direction {
                    let next = match direction {
                        Direction::Up => (active_actor.x, active_actor.y - 16),
                        Direction::Down => (active_actor.x, active_actor.y + 16),
                        Direction::Left => (active_actor.x - 16, active_actor.y),
                        Direction::Right => (active_actor.x + 16, active_actor.y),
                    };
                    let mut item_index = None;
                    for (i, item) in state.items.iter().enumerate() {
                        if item.x == next.0 && item.y == next.1 {
                            item_index = Some(i);
                        }
                    }
                    if let Some(item) = item_index.and_then(|i| state.items.get_mut(i)) {
                        match item.kind {
                            // Collectable items
                            ItemKind::Coin | ItemKind::Gem | ItemKind::Diamond | ItemKind::SmallKey | ItemKind::BossKey => {
                                movement.move_in_direction(direction);
                            }
                            _ => {
                                //
                            }
                        }
                    } else {
                        let mut hit_obj_id = None;
                        let next = ((next.0 / 16) as u8, (next.1 / 16) as u8);
                        for [id, x, y] in OBJECTS {
                            if *x == next.0 && *y == next.1 {
                                hit_obj_id = Some(*id);
                            }
                        }
                        match hit_obj_id {
                            None => {
                                movement.move_in_direction(direction);
                            }
                            // Coin
                            Some(19) => {
                                // TODO: increment $
                                movement.move_in_direction(direction);
                            }
                            // Blue Pot
                            Some(23) => {
                                // TODO: break pot
                            }
                            // Ore
                            Some(28) => {
                                // TODO: mine ore
                            }
                            // Normal Key
                            Some(36) => {
                                // TODO: pick up
                                movement.move_in_direction(direction);
                            }
                            // Boss Key
                            Some(39) => {
                                // TODO: pick up
                                movement.move_in_direction(direction);
                            }
                            // Gem
                            Some(65) => {
                                // TODO: pick up
                                movement.move_in_direction(direction);
                            }
                            // Diamond
                            Some(68) => {
                                // TODO: pick up
                                movement.move_in_direction(direction);
                            }
                            // Shrub
                            Some(87) => {
                                // Todo: ?
                                movement.move_in_direction(direction);
                            }
                            // Tree tops
                            Some(101 | 102 | 103 | 104) => {
                                movement.move_in_direction(direction);
                            }
                            _ => {
                                // noop
                            }
                        }
                    }
                }
            }
        }
    }

    let mut sprite_args = ("player_front_idle", 12, 0, 0);
    for actor in &mut state.actors {
        if actor.is_active {
            match &mut actor.move_state {
                ActorMoveState::Moving(ref mut movement) => {
                    if let Some(MovementDirection::Pending { direction, ref mut timer, is_undo }) = movement.directions.get_mut(0) {
                        let t = timer.progress();
                        let dist = 16;
                        let (key, fps, x, y) = match direction {
                            Direction::Up => (
                                "player_back_walk",
                                fps::FAST,
                                actor.x,
                                terpi(actor.y, actor.y - dist, t, cubic_ease_in)
                            ),
                            Direction::Down => (
                                "player_front_walk",
                                fps::FAST,
                                actor.x,
                                terpi(actor.y, actor.y + dist, t, cubic_ease_in)
                            ),
                            Direction::Left => (
                                "player_left_walk",
                                fps::FAST,
                                terpi(actor.x, actor.x - dist, t, cubic_ease_in),
                                actor.y
                            ),
                            Direction::Right => (
                                "player_right_walk",
                                fps::FAST,
                                terpi(actor.x, actor.x + dist, t, cubic_ease_in),
                                actor.y,
                            ),
                        };

                        if timer.is_done() {
                            actor.x = x;
                            actor.y = y;
                            if *is_undo {
                                movement.directions.remove(0);
                            } else {
                                movement.directions[0] = MovementDirection::Done(*direction);
                            }
                        } else {
                            timer.inc();
                        }
                        sprite_args = (key, fps, x, y);
                    } else {
                        sprite_args.2 = actor.x;
                        sprite_args.3 = actor.y;
                    }
                }
                _ => {
                    sprite_args.2 = actor.x;
                    sprite_args.3 = actor.y;
                }
            }
            let (_key, _fps, x, y) = sprite_args;
            if let TurnPhase::TurnStarting(t, cam) = state.turn_phase {
                let prog = (60. - t as f32) / 60.;
                // let dx = x - cam.0;
                // let dy = y - cam.1;
                // let x = cam.0 + (prog * dx as f32) as i32;
                // let y = cam.1 + (prog * dy as f32) as i32;
                let x = terpi(cam.0, x, prog, ease_in_out_quint);
                let y = terpi(cam.1, y, prog, ease_in_out_quint);
                canvas::set_camera(144/2-x-8, 256/2-y-24-8);
            } else {
                canvas::set_camera(144/2-x-8, 256/2-y-24-8);
            }
            match &actor.move_state {
                ActorMoveState::Moving(ActorMovement { directions, .. }) => match directions.get(0) {
                    None => {},
                    Some(move_dir) => {
                        static mut poof_t: u32 = 0;
                        match move_dir {
                            MovementDirection::Done(_) => poof_t = 0,
                            MovementDirection::Pending { direction, .. } => {
                                let t = poof_t as f32 / 16.;
                                let max_diameter = 4.;
                                let d = if t <= 0.5 {
                                    max_diameter * (t * 2.0)
                                } else {
                                    max_diameter * ((1.0 - t) * 2.0)
                                } as u32;
                                let offset = (d/2) as i32;
                                if *direction == Direction::Right {
                                    circ!(x = x - (poof_t / 2) as i32, y = y + 14 - offset, fill = 0xffffffff, d = d);
                                    circ!(x = x - 3 + (poof_t / 3) as i32, y = y + 14 - offset, fill = 0xffffff44, d = d);
                                }
                                if *direction == Direction::Left {
                                    circ!(x = x + 12 + (poof_t / 2) as i32, y = y + 14 - offset, fill = 0xffffffff, d = d);
                                    circ!(x = x + 15 + (poof_t / 3) as i32, y = y + 14 - offset, fill = 0xffffff44, d = d);
                                }
                                poof_t += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    for [n, x, y] in BACKGROUND {
        turbo::sprite!(&format!("tile_{}", n), x = *x as i32 * 16, y = *y as i32 * 16);
    }


    for actor in &state.actors {
        if let ActorMoveState::Moving(movement) = &actor.move_state {
            let mut prev_x = movement.origin.0;
            let mut prev_y = movement.origin.1;
            for (_i, move_dir) in movement.directions.iter().rev().enumerate() {
                // if actor.motion.len() > 0 {
                //     break;
                // }
                match move_dir.dir() {
                    Direction::Up => {
                        circ!(fill = 0xffffffff, x = prev_x + 6, y = prev_y + 6, d = 4);
                        prev_y -= 16;
                    }
                    Direction::Down => {
                        circ!(fill = 0xffffffff, x = prev_x + 6, y = prev_y + 6, d = 4);
                        prev_y += 16;
                    }
                    Direction::Left => {
                        circ!(fill = 0xffffffff, x = prev_x + 6, y = prev_y + 6, d = 4);
                        prev_x -= 16;
                    }
                    Direction::Right => {
                        circ!(fill = 0xffffffff, x = prev_x + 6, y = prev_y + 6, d = 4);
                        prev_x += 16;
                    }
                }
            }
        }
        if actor.is_active {
            let (key, fps, x, y) = sprite_args;
            let key = match &actor.ty {
                ActorType::Monster(MonsterKind::GoblinGrunt) => "green_goblin_down_idle",
                _ => key
            };
            sprite!(key, x = x, y = y, fps = fps);
        } else {
            let (key, fps, x, y) = ("player_front_idle", 12, actor.x, actor.y);
            let key = match &actor.ty {
                ActorType::Monster(MonsterKind::GoblinGrunt) => "green_goblin_down_idle",
                _ => key
            };
            sprite!(key, x = x, y = y, fps = fps);
        };
    }

    for [n, x, y] in OBJECTS {
        match *n {
            // coin
            19 |
            36 |
            39 |
            65 |
            68 => {
                let speed = 8.;
                let time = state.tick as f32 * 0.016;
                let x = *x as i32 * 16;
                let y = *y as i32 * 16;
                let offset = ((speed * time).cos() * 1.5) as i32;
                circ!(fill = 0x00000055, d = 8 + offset as u32, x = x + 4 + (-offset / 2), y = y + 11);
                let offset = (speed * time).sin() * 2.;
                let y = y + offset as i32;
                sprite!(&format!("tile_{}", n), x = x, y = y);
            }
            _ => {
                let x = *x as i32 * 16;
                let y = *y as i32 * 16;
                if *n == 87 {
                    rect!(fill = 0x6ecb62ff, w = 16, h = 12, x = x, y = y + 4);
                    sprite!(&format!("tile_{}", n), x = x, y = y);
                    for actor in &state.actors {
                        let (key, fps, ax, ay) = if actor.is_active {
                            sprite_args
                        } else {
                            ("player_front_idle", 12, actor.x, actor.y)
                        };
                        if ax == x && ay >= y + 6 && ay <= y + 16 {
                            sprite!(key, x = ax, y = ay, fps = fps);
                        }
                    }
                } else {
                    sprite!(&format!("tile_{}", n), x = x, y = y);
                }
            }
        }
    }

    // Draw player indicator
    let (_key, fps, x, y) = sprite_args;
    let n = (state.tick as i32 / 8) % 4;
    sprite!("indicator_down", x = x, y = y-14 + n, fps = fps);

    let active_actor = &mut state.actors[active_actor_idx];

    for item in &state.items {
        if let ItemState::Collecting(t, _) = &item.state {
            if *t < 60 || state.tick % 4 < 2 {
                let n = ((60. - *t as f32) / 60.) * 2.;
                match item.kind {
                    ItemKind::Coin => {
                        text!("+1", x = active_actor.x + 4, y = active_actor.y - 15 - n as i32, color = 0x00000066);
                        text!("+1", x = active_actor.x + 4, y = active_actor.y - 16 - n as i32);
                    }
                    ItemKind::Gem => {
                        text!("+10", x = active_actor.x, y = active_actor.y - 15 - n as i32, color = 0x00000066);
                        text!("+10", x = active_actor.x, y = active_actor.y - 16 - n as i32);
                    }
                    ItemKind::Diamond => {
                        text!("+100", x = active_actor.x - 4, y = active_actor.y - 15 - n as i32, color = 0x00000066);
                        text!("+100", x = active_actor.x - 4, y = active_actor.y - 16 - n as i32);
                    }
                    _ => {
                        text!("+1", x = active_actor.x + 4, y = active_actor.y - 15 - n as i32, color = 0x00000066);
                        text!("+1", x = active_actor.x + 4, y = active_actor.y - 16 - n as i32);
                    }
                }
            }
        } else {
            let speed = 8.;
            let time = state.tick as f32 * 0.016;
            let x = item.x;
            let y = item.y;
            let p = (speed * time).cos();
            let offset = (p * 1.5) as i32;
            let delta = if offset >= 4 {
                2
            } else {
                0
            };
            circ!(fill = 0x00000055, d = 8 + offset as u32, x = x + 4, y = y + 11);
            let offset = (speed * time).sin() * 2.;
            let y = y + offset as i32;
            sprite!(&format!("tile_{}", item.key), x = x, y = y);
        }
    }

    // for y in 0..8 { // row
    //     for x in 0..10 { // col
    //         if y == 0 {
    //             if x == 0 {
    //                 sprite!("tile_14", x = (x - 1) * 16, y = -16);
    //             } else if x == 9 {
    //                 sprite!("tile_17", x = (x - 1) * 16, y = -16);
    //             } else {
    //                 sprite!("tile_16", x = (x - 1) * 16, y = -16);
    //             }
    //         }
    //     }
    // }

    set_camera(0, 0);

    let active_actor = &mut state.actors[active_actor_idx];

    // Draw top bar / turn indicator
    rect!(fill = 0x000000ff, x = 0, y = 0, w = 256, h = 8);
    let text = &format!("{} | GOLD: ${}.00", active_actor.name, active_actor.gold);
    text!(text, x = 2, y = 2, font = Font::S);

    // Draw health
    sprite!("tile_55", x = 0, y = 8); // full
    sprite!("tile_56", x = 13, y = 8); // half
    sprite!("tile_57", x = 26, y = 8); // empty

    // Draw items
    for i in 0..3 {
        rectv!(fill = 0x000000ff, x = 144 - (21 * (i + 1)), y = 11, w = 18, h = 18, border = Border {
            radius: 2,
            size: 1,
            color: 0xffffffff,
        });
        if let Some(item) = active_actor.inventory.get(i as usize) {
            match item.kind {
                ItemKind::SmallKey => {
                    sprite!("tile_36", x = 144 - (21 * (i + 1)) + 1, y = 12);
                    text!(&format!("{}", item.qty), x = 157 - (21 * (i + 1)), y = 24, font = Font::S, color = 0x000000ff);
                    text!(&format!("{}", item.qty), x = 156 - (21 * (i + 1)), y = 23, font = Font::S);
                }
                ItemKind::BossKey => {
                    sprite!("tile_39", x = 144 - (21 * (i + 1)) + 1, y = 12);
                    text!(&format!("{}", item.qty), x = 157 - (21 * (i + 1)), y = 24, font = Font::S, color = 0x000000ff);
                    text!(&format!("{}", item.qty), x = 156 - (21 * (i + 1)), y = 23, font = Font::S);
                }
                _ => {
                    sprite!("tile_20", x = 144 - (21 * (i + 1)) + 1, y = 12);
                    text!(&format!("{}", item.qty), x = 157 - (21 * (i + 1)), y = 24, font = Font::S, color = 0x000000ff);
                    text!(&format!("{}", item.qty), x = 156 - (21 * (i + 1)), y = 23, font = Font::S);
                }
            }
        }
    }

    // Debug
    // text!(&format!("{:#?}", active_actor.inventory), x = 4, y = 140, font = Font::S);
    // rect!(fill = 0x000000ff, y = 32, w = 144, h = 1);


    if let ActorMoveState::Ready = active_actor.move_state {
        // Draw bottom panel
        let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
        let text = &format!("{}, it's your turn :D\nPress [A] to spin!", active_actor.name);
        text!(text, x = x + 2, y = y + 2, font = Font::M);
    }

    if let ActorMoveState::Moving(ref mut movement) = active_actor.move_state {
        let max = movement.range;
        let curr = movement.directions.len();
        for i in 0..6 {
            circ!(fill = 0x000000ff, d = 6, x = 2 + (i as i32 * 9), y = 17 + 6);
            let n = max as usize - curr;
            if i < n as u32 {
                circ!(fill = 0x5fcde4ff, d = 4, x = 3 + (i as i32 * 9), y = 18 + 6);
            } else {
                circ!(fill = 0x444444ff, d = 4, x = 3 + (i as i32 * 9), y = 18 + 6);
            }
        }
        if max as usize == curr && !movement.is_direction_pending() {
            // Draw bottom panel
            let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
            text!("You can't move any farther.\nPress [A] to end your turn.", x = x + 2, y = y + 2, font = Font::M);
        }
    } else {
        for i in 0..6 {
            circ!(fill = 0x000000ff, d = 6, x = 2 + (i as i32 * 9), y = 17 + 6);
            circ!(fill = 0x444444ff, d = 4, x = 3 + (i as i32 * 9), y = 18 + 6);
        }
    }

    // Draw spinner
    if let ActorMoveState::Spinning(ref mut spin)  = &mut active_actor.move_state {
        let diameter = 64;
        let half_diameter = (diameter / 2) as i32;
        // circ!(x = (144/2) - half_diameter, y = ((256-48)/2) - half_diameter, fill = 0xa23000ff, d = diameter);
        circ!(x = (144/2) - half_diameter, y = ((256-48)/2) - half_diameter, fill = 0x000000ff, d = diameter);
        circ!(x = (144/2) - half_diameter, y = 2 + ((256-48)/2) - half_diameter, fill = 0x000000aa, d = diameter);
        let diameter = 62;
        let half_diameter = (diameter / 2) as i32;
        circ!(x = (144/2) - half_diameter, y = ((256-48)/2) - half_diameter, fill = 0xc56129ff, d = diameter);
        let diameter = 56;
        let half_diameter = (diameter / 2) as i32;
        circ!(x = (144/2) - half_diameter, y = ((256-48)/2) - half_diameter, fill = 0xffce60ff, d = diameter);
        let diameter = 52;
        let half_diameter = (diameter / 2) as i32;
        let fill = if spin.speed <= 1. {
            0x4181c5ff
        } else {
            shift_hue(0x293c8bff, (state.tick - spin.started_at) * 40)
        };
        circ!(x = (144/2) - half_diameter, y = ((256-48)/2) - half_diameter, fill = fill, d = diameter);

        for i in 0..6 {
            let i = i as f32;
            let (x, y) = rotate_position((144/2) as f32, ((256-48)/2) as f32, 19., (i * 60.) + 60.);
            text!(&format!("{}", i + 1.), x = x - 3, y = y - 3, font = Font::L);
        }

        // sprite!("tile_39", x = (144/2)-8, y = ((256-48)/2)-8, deg = spin.angle);
        sprite!("spinner_arrow", x = (144/2)-8, y = ((256-48)/2)-8, deg = spin.angle);
        spin.angle += spin.speed as i32;
        spin.speed *= spin.decel;
        if spin.speed <= 1. && spin.value.is_none() {
            let angle = (spin.angle - 30) % 360;
            let n = (angle / 60) + 1;
            spin.value = Some(n as u32);
        }
        // Draw bottom panel
        if let Some(value) = spin.value {
            let (key, text) = match value {
                6 => ("pepe_happy", "6 energy?!\nLFG you're THAT guy!"),
                5 => ("pepe_happy", "You got 5 energy!"),
                4 => ("pepe_happy", "You got 4 energy!"),
                3 => ("pepe_happy", "You got 3 energy!"),
                2 => ("pepe_happy", "You got 2 energy."),
                1 => ("pepe_sad", "You got 1 energy (tT-Tt)"),
                _ => ("pepe_happy", "How is that even possible?!")
            };
            let (x, y) = draw_bottom_panel(BottomPanelOptions { key });
            text!(text, x = x + 2, y = y + 2, font = Font::M);
        }
    }

    for item in &state.items {
        match item {
            Item { state: ItemState::Collecting(_, true), kind: ItemKind::Coin, .. } => {
                let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
                text!("Get that bread, son.", x = x + 2, y = y + 2, font = Font::M);
            }
            Item { state: ItemState::Collecting(_, true), kind: ItemKind::Gem, .. } => {
                let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
                text!("Bussin' fr fr", x = x + 2, y = y + 2, font = Font::M);
            }
            Item { state: ItemState::Collecting(_, true), kind: ItemKind::Diamond, .. } => {
                let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
                text!("GYAT", x = x + 2, y = y + 2, font = Font::M);
            }
            Item { state: ItemState::Collecting(_, true), kind: ItemKind::SmallKey, .. } => {
                let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
                text!("Let's gooooo!", x = x + 2, y = y + 2, font = Font::M);
            }
            Item { state: ItemState::Collecting(_, true), kind: ItemKind::BossKey, .. } => {
                let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
                text!("POGGERS! O_O", x = x + 2, y = y + 2, font = Font::M);
            }
            _ => {}
        }
    }

    match &mut state.turn_phase {
        TurnPhase::TurnEnding(t) => {
            // let (x, y) = draw_bottom_panel();
            // let text = &format!("{}'s turn is ending...", active_actor.name);
            // text!(text, x = x + 2, y = y + 2, font = Font::M);
            *t -= 1;
            if *t == 0 {
                state.turn_phase = TurnPhase::TurnStarting(60, (active_actor.x, active_actor.y));
                active_actor.is_active = false;
                active_actor_idx = (active_actor_idx + 1) % state.actors.len();
                state.actors[active_actor_idx].is_active = true;
                state.actors[active_actor_idx].move_state = ActorMoveState::Ready;
            }
        }
        TurnPhase::TurnStarting(t, _prev_cam) => {
            let (x, y) = draw_bottom_panel(BottomPanelOptions { key: "pepe_happy" });
            let text = &format!("{}, get ready...", active_actor.name);
            text!(text, x = x + 2, y = y + 2, font = Font::M);
            *t -= 1;
            if *t == 0 {
                state.turn_phase = TurnPhase::PlayerAction;
            }
        }
        TurnPhase::PlayerAction => {
            // Next player
            if active_actor.move_state == ActorMoveState::Chilling {
                state.turn_phase = TurnPhase::TurnEnding(30);
            }
        }
        TurnPhase::SpecialEvent(_) => {
            // TODO
        }
    }

    state.tick += 1;
    state.save();
}

const BACKGROUND: &[[u8; 3]] = &[
    //
    [11, 3, 1],  // bricks 1
    [11, 3, 2],  // bricks 1
    [11, 3, 3],  // bricks 1
    [11, 4, 3],  // bricks 1
    [11, 5, 3],  // bricks 1
    [11, 5, 4],  // bricks 1
    [11, 5, 5],  // bricks 1
    [11, 5, 6],  // bricks 1
    [11, 5, 7],  // bricks 1
    [11, 5, 8],  // bricks 1
    [11, 4, 8],  // bricks 1
    [11, 3, 8],  // bricks 1
    [11, 3, 9],  // bricks 1
    [11, 2, 9],  // bricks 1
    [11, 1, 9],  // bricks 1
    [11, 1, 8],  // bricks 1
    [11, 0, 8],  // bricks 1
    [11, 0, 7],  // bricks 1
    [11, 0, 6],  // bricks 1
    [11, 0, 5],  // bricks 1
    [11, 0, 4],  // bricks 1
    [11, 1, 3],  // bricks 1
    [11, 2, 3],  // bricks 1
    [162, 0, 1], // down path
    [162, 0, 2], // down path
    [29, 1, 0],  // gravel
    [42, 5, 0],  // shrooms
    [42, 7, 3],  // shrooms
    [9, 5, 3],   // flowers
    [106, 2, 4], // grass
    [106, 5, 2], // grass
    [106, 7, 1], // grass
    [105, 6, 3], // thickgrass
    [135, 5, 1], // grass 3
    [119, 6, 1], // semi-thickgrass
    [120, 7, 2], // grass 2
    [146, 2, 2], // grass 4
    [146, 7, 5], // grass 4
    [25, 7, 6],  // flower
    [25, 0, 3],  // flower
];

const OBJECTS: &[[u8; 3]] = &[
    //
    [19, 8, 1],
    // [82, 0, 3],
    // [81, 2, 3],
    // [83, 1, 3],
    [84, 1, 4], // stump
    // [162, 0, 1],
    // [162, 0, 2],
    [23, 1, 1], // blue pot
    // [19, 1, 2], // coin
    [28, 4, 2], // ore
    // [36, 5, 2], // normal key
    // [39, 5, 8], // boss key
    [43, 6, 7], // big shroom
    [40, 2, 8], // normie grave
    [41, 6, 8], // baddy grave
    [61, 3, 0], // big chest
    [71, 6, 2], // small chest
    // [65, 5, 5], // gem
    // [68, 0, 7], // diamond
    [87, 2, 5], // shrub
    [87, 3, 5], // shrub
    [87, 4, 5], // shrub
    [24, 4, 4], // sign
    // treeeeeees
    [101, 1, 6],
    [102, 2, 6],
    [103, 3, 6],
    [104, 4, 6],
    [115, 1, 7],
    [116, 2, 7],
    [117, 3, 7],
    [118, 4, 7],
];

// struct Sprite;
// impl Sprite {
//     fn draw_coin(x: i32, y: i32) {
//         turbo::sprite!("tile_19", x = x, y = y)
//     }
//     fn draw_red_pot(x: i32, y: i32) {
//         turbo::sprite!("tile_20", x = x, y = y);
//     }
// }

fn terpi(start: i32, end: i32, t: f32, f: fn(t: f32) -> f32) -> i32 {
    if t <= 0. {
        return start;
    }
    if t >= 1. {
        return end;
    }
    let t = f(t);
    let delta = end - start;
    let n = start as f32 + delta as f32 * t;
    n.round() as i32
}

fn cubic_ease_in(t: f32) -> f32 {
    t * t * t
}

fn ease_in_out_quint(x: f32) -> f32 {
    if x < 0.5 {
        16.0 * x.powi(5)
    } else {
        1.0 - ((-2.0 * x + 2.0).powi(5) / 2.0)
    }
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * (std::f32::consts::PI / 180.0)
}

fn rotate_position(x: f32, y: f32, dst: f32, angle: f32) -> (i32, i32) {
    let angle_in_radians = degrees_to_radians(angle - 90.);
    let x = x + dst * angle_in_radians.cos();
    let y = y + dst * angle_in_radians.sin();
    (x as i32, y as i32)
}

// fn shift_hue(color: u32, step: u32) -> u32 {
//     let r = (color >> 16) & 0xFF;
//     let g = (color >> 8) & 0xFF;
//     let b = color & 0xFF;

//     let new_r = (r + step) % 256;
//     let new_g = (g + step) % 256;
//     let new_b = (b + step) % 256;

//     return (new_r << 16) | (new_g << 8) | new_b;
// }

enum ColorState {
    RtoG,
    GtoB,
    BtoR,
}
static mut COLOR_STATE: ColorState = ColorState::BtoR;

unsafe fn shift_hue(color: u32, step: u32) -> u32 {
    let mut r = ((color >> 16) & 0xFF) as u8;
    let mut g = ((color >> 8) & 0xFF) as u8;
    let mut b = (color & 0xFF) as u8;

    match COLOR_STATE {
        ColorState::RtoG => {
            g = g.saturating_add((step % 256) as u8);
            if g >= 255 {
                COLOR_STATE = ColorState::GtoB;
            }
        }
        ColorState::GtoB => {
            r = r.saturating_sub((step % 256) as u8);
            if r <= 0 {
                COLOR_STATE = ColorState::BtoR;
            }
        }
        ColorState::BtoR => {
            b = b.saturating_add((step % 256) as u8);
            if b >= 255 {
                COLOR_STATE = ColorState::RtoG;
            }
        }
    }

    return ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
}

struct BottomPanelOptions<'a> {
    key: &'a str,
}

fn draw_bottom_panel(opts: BottomPanelOptions) -> (i32, i32) {
    // Draw bottom panel
    let mut x = 0;
    let mut h = 20;
    let mut y = 256 - h as i32;
    let mut w = 144;
    rect!(fill = 0x00000099, x = x, y = y - 1, w = w, h = 1);
    rect!(fill = 0x000000ff, x = x, y = y, w = w, h = h);
    y += 1;
    h -= 1;
    // rect!(fill = 0xffffffff, x = x, y = y, w = w, h = h);
    rectv!(
        fill = 0x000000ff,
        x = x,
        y = y,
        w = w,
        h = h,
        border = Border {
            radius: 2,
            size: 1,
            color: 0xffffffff,
        }
    );
    let pad = 2;
    x += pad / 2;
    w -= pad as u32;
    // y += pad / 2;
    // h -= pad as u32;
    // rect!(fill = 0x000000ff, x = x, y = y, w = w, h = h);
    sprite!(opts.key, x = x - 1, y = y - 32);
    (x, y)
}
