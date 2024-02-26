use lumberjack::constants::MAX_WOOD_PER_TREE;
use lumberjack::instructions::GameData;
use lumberjack::state::player_data::PlayerData;
use turbo::borsh::BorshDeserialize;
use turbo::solana::{
    self,
    anchor::Program,
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, system_program},
};

turbo::cfg! {r#"
    name = "Solana Lumberjack"
    [settings]
    resolution = [256, 256]
    [solana]
    http-rpc-url = "http://localhost:8899"
    ws-rpc-url = "ws://localhost:8900"
"#}

turbo::go! {
    clear!(0x95bea1ff);

    let level = "Forest";

    // Load  Data
    match (get_player_data(), get_game_data(level)) {
        // No player data or no game data
        // - Allow player to initialize
        (None, _) => start_game_screen(level, "No Player Data..."),
        (Some(Ok(_)), None) => start_game_screen(level, "No Game Data..."),

        // Player and game data exist
        // - Allow player to chop wood
        (Some(Ok(player_data)), Some(Ok(game_data))) => chopping_screen(level, &player_data, &game_data),

        // Data fetching errors
        // - NGMI (fix your code or RPC)
        (Some(Err(err)),  _) => error_screen(level, "Player data fetch error", err),
        (_, Some(Err(err))) => error_screen(level, "Game data fetch error", err),
    };
}

fn start_game_screen(level: &str, desc: &str) {
    // Draw background image
    sprite!("title_screen");

    // Blinking start message
    if tick() / 30 % 2 == 0 {
        text!(
            "PRESS START",
            x = 84,
            y = 232,
            font = Font::L,
            color = 0x000000ff
        );
    }

    // Start game
    if gamepad(0).start.just_pressed() {
        init_player_and_game(level);
    }

    // Logs debug information
    if gamepad(0).select.just_pressed() {
        let signer_pubkey = solana::signer();
        log!("-----------------------------------------------------");
        log!("DEBUG");
        log!("-----------------------------------------------------");
        log!("Signer = {signer_pubkey}");
        log!("Level  = {level}");
        log!("Status = {desc}");
        log!("-----------------------------------------------------");
    }
}

fn chopping_screen(level: &str, player_data: &PlayerData, game_data: &GameData) {
    sprite!("forest", x = 0);
    let mut x = 0;
    let mut y = 0;
    set_cam!(x = 0, y = 0);
    rect!(w = 128, h = 16, color = 0x555533ff);
    rect!(w = 128, h = 16, color = 0x335555ff, x = 128);
    text!(
        &format!("LEVEL: {level}",),
        font = Font::L,
        x = 4,
        y = 4,
        color = 0xffffffff
    );
    let chopped = (game_data.total_wood_collected as f32 / MAX_WOOD_PER_TREE as f32) * 100.;
    text!(
        &format!("CHOPPED: {chopped:.2}%",),
        font = Font::L,
        x = 132,
        y = 4,
        color = 0xffffffff
    );

    y += 4;
    y += 128;
    if gamepad(0).start == Button::Released {
        set_cam!(x = 0, y = 0);
        let n = if tick() / 20 % 2 == 0 { 3 } else { 0 };
        sprite!("lumberjack_swing1", x = 156, y = y + n, h = 100 - n as u32);
    } else if player_data.energy == 0 {
        sprite!("lumberjack_sweat", x = 156, y = y, h = 100);
    } else {
        set_cam!(
            x = (tick() as i32 % 6) as i32 - 3,
            y = (tick() as i32 % 6) as i32 - 3,
        );
        sprite!("lumberjack_swing2", x = 156, y = y, h = 100);
    }
    // x = 132;
    // y += 8;
    x = 64;
    y = 20;
    text!(
        &format!("Energy: {}/100", player_data.energy),
        font = Font::L,
        x = x,
        y = y,
        color = 0x000000ff
    );
    y += 10;
    stat_bar((player_data.energy, 100), x, y, 128, 0x008899ff, 0x000000ff);
    y += 16;
    let wood_msg = &format!("Wood: {}", player_data.wood);
    let x = 128 - (wood_msg.chars().count() * 4) as i32;
    text!(wood_msg, x = x, y = y, font = Font::L, color = 0x000000ff);
    y += 8;

    // Chop the tree
    if gamepad(0).start.just_pressed() {
        chop_tree(level);
    }

    // Logs debug information
    if gamepad(0).select.just_pressed() {
        log!("-----------------------------------------------------");
        log!("DEBUG");
        log!("-----------------------------------------------------");
        log!("{player_data:#?}");
        log!("{game_data:#?}");
        log!("-----------------------------------------------------");
    }
}

fn stat_bar(value: (u64, u64), x: i32, y: i32, w: u32, color: u32, border_color: u32) {
    rect!(
        x = x,
        y = y,
        w = w,
        h = 12,
        color = 0x000000ff,
        border_radius = 2
    );
    rect!(
        x = x,
        y = y + 2,
        w = (w as f32 * (value.0 as f32 / value.1 as f32)) as u32,
        h = 8,
        color = color,
        border_radius = 2
    );
    rect!(
        x = x,
        y = y,
        w = w,
        h = 12,
        color = 0x00000000,
        border_radius = 2,
        border_width = 2,
        border_color = border_color
    );
}

fn error_screen(level: &str, label: &str, err: std::io::Error) {
    let y = 0;
    let msg = &format!("Level {} - {}: {:#?}", level, label, err);
    text!(msg, y = y, font = Font::S, color = 0xff0000ff);
}

fn get_player_pubkey() -> Pubkey {
    let signer_pubkey = solana::signer();
    Pubkey::find_program_address(&[b"player", signer_pubkey.as_ref()], &lumberjack::ID).0
}

fn get_game_pubkey(level_seed: &str) -> Pubkey {
    Pubkey::find_program_address(&[level_seed.as_ref()], &lumberjack::ID).0
}

fn get_player_data() -> Option<Result<PlayerData, std::io::Error>> {
    let player_pubkey = get_player_pubkey();
    let player_data_account = solana::rpc::get_account(player_pubkey);
    if let Some(ref account_info) = player_data_account.value {
        return match PlayerData::deserialize(&mut account_info.data.get(8..).unwrap_or(&[])) {
            // Success
            Ok(data) => Some(Ok(data)),
            // Error
            Err(err) => Some(Err(err)),
        };
    }
    // Loading or doesn't exist
    return None;
}

fn get_game_data(level: &str) -> Option<Result<GameData, std::io::Error>> {
    let game_pubkey = get_game_pubkey(level);
    let game_data_account = solana::rpc::get_account(game_pubkey);
    if let Some(ref account_info) = game_data_account.value {
        return match GameData::deserialize(&mut account_info.data.get(8..).unwrap_or(&[])) {
            // Success
            Ok(data) => Some(Ok(data)),
            // Error
            Err(err) => Some(Err(err)),
        };
    }
    // Loading or doesn't exist
    return None;
}

fn init_player_and_game(level: &str) {
    let instruction_name = "init_player";
    let player_pubkey = get_player_pubkey();
    let level_pubkey = get_game_pubkey(level);
    let signer_pubkey = solana::signer();
    let accounts = vec![
        AccountMeta::new(player_pubkey, false),
        AccountMeta::new(level_pubkey, false),
        AccountMeta::new(signer_pubkey, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let args = lumberjack::instruction::InitPlayer {
        _level_seed: level.to_string(),
    };
    let tx = Program::new(lumberjack::ID)
        .instruction(instruction_name)
        .accounts(accounts)
        .args(args)
        .transaction();
    log!("{:#?}", tx);
    solana::rpc::sign_and_send_transaction(&tx);
}

fn chop_tree(level: &str) {
    let instruction_name = "chop_tree";
    let player_pubkey = get_player_pubkey();
    let level_pubkey = get_game_pubkey(level);
    let signer_pubkey = solana::signer();
    let accounts = vec![
        AccountMeta::new(player_pubkey, false),
        AccountMeta::new(level_pubkey, false),
        AccountMeta::new(signer_pubkey, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let args = lumberjack::instruction::ChopTree {
        _level_seed: level.to_string(),
        counter: rand() as u16,
    };
    let tx = Program::new(lumberjack::ID)
        .instruction(instruction_name)
        .accounts(accounts)
        .args(args)
        .transaction();
    // log!("{:#?}", tx);
    solana::rpc::sign_and_send_transaction(&tx);
}
