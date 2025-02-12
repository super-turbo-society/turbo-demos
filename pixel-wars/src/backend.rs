use crate::*;

#[export_name = "turbo/reset_choice_data"]
unsafe extern "C" fn reset_choice_data_os() -> usize {
    let file_path = format!("global_choice_counter");
    let choices = TeamChoiceCounter {
        team_0: 0,
        team_1: 0,
    };
    let data = borsh::to_vec(&choices).unwrap();
    let Ok(_) = os::server::write_file(&file_path, &data) else {
        return os::server::CANCEL;
    };
    return os::server::COMMIT;
}

#[export_name = "turbo/choose_team"]
unsafe extern "C" fn choose_team_os() -> usize {
    //write to a file that you chose that team
    let choice_data = os::server::command!(TeamChoiceCounter);
    let userid = os::server::get_user_id();
    let file_path = format!("global_choice_counter");
    let mut all_choices = os::server::read_or!(
        TeamChoiceCounter,
        &file_path,
        TeamChoiceCounter {
            team_0: 0,
            team_1: 0
        }
    );
    all_choices.team_0 += choice_data.team_0;
    all_choices.team_1 += choice_data.team_1;
    let data = borsh::to_vec(&all_choices).unwrap();
    let Ok(_) = os::server::write_file(&file_path, &data) else {
        return os::server::CANCEL;
    };
    //TODO: turn the file paths into constants
    let battle = os::server::read!(Battle, "current_battle");
    let seed = battle.team_seed;
    let file_path = format!("users/{}/choice/{}", userid, seed);
    let mut num: i32 = 0;
    if choice_data.team_1 == 1 {
        num = 1;
    }
    let data = num.to_le_bytes();
    let Ok(_) = os::server::write_file(&file_path, &data) else {
        return os::server::CANCEL;
    };
    os::server::log!("Choice: {:?}", all_choices);
    return os::server::COMMIT;
}

#[export_name = "turbo/commit_points"]
unsafe extern "C" fn commit_points() -> usize {
    //watch file to get sim result
    let sim_result = os::server::read!(SimulationResult, "current_result");

    let winning_team_index = sim_result.winning_team;
    let seed = sim_result.seed;
    //watch file to get the user choice
    let userid = os::server::get_user_id();
    let file_path = format!("users/{}/choice/{}", userid, seed);
    let choice = os::server::read_file(&file_path)
        .ok()
        .and_then(|file| <u8>::try_from_slice(&file).ok());

    let mut is_win = false;
    if choice.is_some() {
        if choice == winning_team_index {
            is_win = true;
        }

        let file_path = format!("users/{}/stats", userid);
        //read from the file
        let mut stats = os::server::read_or!(UserStats, &file_path, UserStats { points: 100 });
        //check if it matches the winning team
        if is_win {
            stats.points += 10;
        } else {
            stats.points -= 10;
        }
        let Ok(_) = os::server::write!(&file_path, stats) else {
            return os::server::CANCEL;
        };
        os::server::log!("Points: {}", stats.points);
    }
    os::server::log!("Choice {:?}", choice);
    return os::server::COMMIT;
}

#[export_name = "turbo/generate_teams"]
unsafe extern "C" fn generate_teams_os() -> usize {
    let seed: u32 = os::server::random_number();
    let mut team_rng = RNG::new(seed);
    let Ok(data_store) = UnitDataStore::load_from_csv(UNIT_DATA_CSV) else {
        return os::server::CANCEL;
    };
    let team_0 = generate_team(&data_store, &mut team_rng, None, "Pixel Peeps".to_string());
    let team_1 = generate_team(
        &data_store,
        &mut team_rng,
        Some(&team_0),
        "Battle Bois".to_string(),
    );
    let battle = Battle {
        team_0,
        team_1,
        team_seed: seed,
        battle_seed: None,
    };
    let bytes = battle.try_to_vec().unwrap();
    let Ok(_) = os::server::write_file("current_battle", &bytes) else {
        return os::server::CANCEL;
    };
    let t = battle.team_0;
    let num = t.units.len();
    os::server::log!("SEED: {}", seed);
    os::server::log!("Team_0: {:?}", num);
    return os::server::COMMIT;
}

#[export_name = "turbo/simulate_battle"]
unsafe extern "C" fn simulate_battle_os() -> usize {
    let seed: u32 = os::server::random_number();
    let mut simulation_rng = RNG::new(seed);
    let Ok(data_store) = UnitDataStore::load_from_csv(UNIT_DATA_CSV) else {
        return os::server::CANCEL;
    };
    //add the battle seed to the current battle
    let mut battle = os::server::read!(Battle, "current_battle");
    battle.battle_seed = Some(seed);
    let bytes = battle.try_to_vec().unwrap();
    let Ok(_) = os::server::write_file("current_battle", &bytes) else {
        return os::server::CANCEL;
    };

    let team_0 = battle.team_0;
    let team_1 = battle.team_1;
    let mut teams = vec![team_0, team_1];
    let mut units = create_units_for_all_teams(&mut teams, &mut simulation_rng, &data_store);
    let mut attacks = Vec::new();
    let mut traps = Vec::new();
    let mut craters = Vec::new();
    let mut explosions = Vec::new();
    let mut i: u32 = 0;
    let winning_team_index = loop {
        step_through_battle(
            &mut units,
            &mut attacks,
            &mut traps,
            &mut explosions,
            &mut craters,
            &mut simulation_rng,
            &mut Vec::new(),
            true,
        );
        i += 1;
        if let Some(winner_idx) = has_some_team_won(&units) {
            break winner_idx;
        }
    };
    //write a simulation result to a file
    let living_units = all_living_units(&units);
    let sim_result = SimulationResult {
        seed: battle.team_seed,
        living_units,
        winning_team: Some(winning_team_index),
        num_frames: i,
    };
    let bytes = sim_result.try_to_vec().unwrap();
    let Ok(_) = os::server::write_file("current_result", &bytes) else {
        return os::server::CANCEL;
    };
    os::server::log!("Battle Seed: {}", seed);
    os::server::log!("Frames: {}", i);
    os::server::alert!("Result: {:?}", sim_result);
    os::server::alert!("Winning Team: {:?}", winning_team_index);

    os::server::COMMIT
}

//TODO: use this function in other places where we are using this
//TODO: make this return something if it can't find the file instead of crashing
pub fn get_seed_from_turbo_os() -> u32 {
    let battle = os::client::watch_file("pixel-wars", "current_battle")
        .data
        .and_then(|file| Battle::try_from_slice(&file.contents).ok());
    let seed = battle.unwrap().team_seed;
    seed
}
