extern crate state;
extern crate brain;

use state::*;
use brain::plan;

fn main() {
    println!("Hello, world!");

    let mut game_state = GameState::default();
    let mut bot = BotState::default();
    let mut desired_contact = DesiredContact::default();

    loop {
        let plan_result = plan::hybrid_a_star(&game_state.player, &desired_contact, &config);

        if let Some(plan) = plan_result.plan {
        } else {
            println!("Failed: {:?}", game_state.player);
        }
    }
}
