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
    [solana]
    http-rpc-url = "http://localhost:8899"
    ws-rpc-url = "ws://localhost:8900"
"#}

turbo::go! {
    let level = "1";

    // Load  Data
    match (get_player_data(), get_game_data(level)) {
        // Player and game data exist
        // - Allow player to chop wood
        (Some(Ok(player_data)), Some(Ok(game_data))) => chopping_screen(level, &player_data, &game_data),

        // No player data or no game data
        // - Allow player to initialize
        (None, _) => start_game_screen(level, "No Player Data..."),
        (Some(Ok(_)), None) => start_game_screen(level, "No Game Data..."),

        // Data fetching errors
        // - NGMI (fix your code or RPC)
        (Some(Err(err)),  _) => error_screen(level, "Player data fetch error", err),
        (_, Some(Err(err))) => error_screen(level, "Game data fetch error", err),
    };
}

fn start_game_screen(level: &str, desc: &str) {
    let mut y = 0;

    let signer_pubkey = solana::signer();
    let msg = &format!("Signer = {}", signer_pubkey);
    text!(msg, y = y);
    y += 8;

    let msg = &format!("{} Start level {}", desc, level);
    text!(msg, y = y);

    if gamepad(0).start.just_pressed() {
        init_player_and_game(level);
    }
}

fn chopping_screen(level: &str, player_data: &PlayerData, game_data: &GameData) {
    let mut y = 0;

    let msg = &format!("Level = {}", level);
    text!(msg, y = y);
    y += 8;

    let msg = &format!("{:#?}\n{:#?}", player_data, game_data);
    text!(msg, y = y, font = Font::S);
    let height = msg.lines().count() * 5;
    y += height as i32;

    let msg = &format!(
        "Total Wood Available: {}",
        MAX_WOOD_PER_TREE - game_data.total_wood_collected
    );
    text!(msg, y = y, font = Font::S);

    if gamepad(0).start.just_pressed() {
        chop_tree(level);
    }
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
