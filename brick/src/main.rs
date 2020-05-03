const USAGE: &'static str = "
Brick

Usage:
  brick --bot
  brick --bot-test
  brick --simulate

Options:
  -h --help     Show this screen.
  --version     Show version.
  --bot         Run regular bot in a match.
  --bot-test    Run test bot during dev in an empty match.
  --simulate    Run bot in a simulation of RL with visualization.
";

extern crate brain;
extern crate docopt;
extern crate kiss3d;
extern crate nalgebra as na;
extern crate passthrough;
extern crate ratelimit;
extern crate rlbot;
extern crate state;

#[macro_use]
extern crate lazy_static;

use docopt::Docopt;
use std::error::Error;
use std::f32;
use std::f32::consts::PI;
use std::panic;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, RwLock};
use std::thread;
use std::time::Duration;

use passthrough::{human_input, update_gamepad, Gamepad, Gilrs};
use state::*;

use kiss3d::light::Light;
use kiss3d::resource::MeshManager;
use kiss3d::window::Window;
use na::{Point3, Rotation3, Translation3, Unit, UnitQuaternion, Vector3};

pub const TICK: f32 = 1.0 / 120.0; // FIXME import from predict

#[derive(Default)]
struct BotIoConfig<'a> {
    manipulator: Option<fn(&rlbot::RLBot, &GameState, &BotState)>,
    print_turn_errors: bool,
    start_match: bool,
    match_settings: Option<rlbot::MatchSettings<'a>>,
}

lazy_static! {
    static ref GAME_STATE: RwLock<GameState> = {
        RwLock::new(GameState::default())
    };

    static ref LINES: RwLock<Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>> = {
        RwLock::new(vec![])
    };

    static ref POINTS: RwLock<Vec<(Point3<f32>, Point3<f32>)>> = {
        RwLock::new(vec![])
    };
}

fn run_visualization(){
    let mut window = Window::new("Rocket League Visualization");

    // we're dividing everything by 1000 until we can set the camera up to be more zoomed out
    let mut sphere = window.add_sphere(BALL_COLLISION_RADIUS / 1000.0);
    let mut car = window.add_cube(
        CAR_DIMENSIONS.x / 1000.0,
        CAR_DIMENSIONS.y / 1000.0,
        CAR_DIMENSIONS.z / 1000.0,
    );

    let arena_mesh = MeshManager::load_obj(
        Path::new("./assets/arena.obj"),
        Path::new("./assets/"),
        "arena",
    )
    .expect("Can't load arena obj file")
    .pop()
    .expect("Missing arena mesh")
    .1
    .clone();
    let mut arena = window.add_mesh(arena_mesh, Vector3::new(0.0, 0.0, 0.0));
    arena.set_surface_rendering_activation(false);
    arena.set_points_size(0.1);
    arena.set_lines_width(0.1);
    arena.set_local_scale(0.001, 0.001, 0.001);

    sphere.set_color(0.8, 0.8, 0.8);
    car.set_color(0.1, 0.4, 1.0);

    window.set_light(Light::StickToCamera);

    let mut ratelimiter = ratelimit::Builder::new()
        .interval(Duration::from_millis(1000 / 60)) // rendering limited to 60 fps
        .build();

    while window.render() {
        ratelimiter.wait();

        let game_state = &GAME_STATE.read().unwrap();
        let lines = &LINES.read().unwrap();
        let points = &POINTS.read().unwrap();

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        sphere.set_local_translation(Translation3::from(
            game_state.ball.position.map(|c| c / 1000.0),
        ));

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        let hitbox_position = game_state.player.hitbox_center().map(|c| c / 1000.0);
        car.set_local_translation(Translation3::from(hitbox_position));
        car.set_local_rotation(game_state.player.rotation);

        // grid for debugging
        //for x in (-160..160) {
        //    window.draw_line(
        //        &Point3::new((x as f32) * 25.0 / 1000.0, -10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new((x as f32) * 25.0 / 1000.0,  10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.15, 0.15, 0.15)
        //    );
        //}
        //for x in (-40..40) {
        //    window.draw_line(
        //        &Point3::new((x as f32) * 100.0 / 1000.0, -10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new((x as f32) * 100.0 / 1000.0,  10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.3, 0.3, 0.3)
        //    );
        //}
        //for y in (-160..160) {
        //    window.draw_line(
        //        &Point3::new(-10_000.0 / 1000.0, (y as f32) * 25.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new( 10_000.0 / 1000.0, (y as f32) * 25.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.15, 0.15, 0.15)
        //    );
        //}
        //for y in (-40..40) {
        //    window.draw_line(
        //        &Point3::new(-10_000.0 / 1000.0, (y as f32) * 100.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new( 10_000.0 / 1000.0, (y as f32) * 100.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.3, 0.3, 0.3)
        //    );
        //}

        for l in lines.iter() {
            window.draw_line(
                &Point3::new(l.0.x / 1000.0, l.0.y / 1000.0, l.0.z / 1000.0),
                &Point3::new(l.1.x / 1000.0, l.1.y / 1000.0, l.1.z / 1000.0),
                &l.2,
            );
        }

        for p in points.iter() {
            window.draw_point(
                &Point3::new(p.0.x / 1000.0, p.0.y / 1000.0, p.0.z / 1000.0),
                &p.1,
            );
        }
    }
}

/// main bot playing loop
/// this is the entry point for custom logic for this specific bot
fn run_bot() {
    let (state_sender, state_receiver): (
        Sender<(GameState, BotState)>,
        Receiver<(GameState, BotState)>,
    ) = mpsc::channel();
    let (plan_sender, plan_receiver): (Sender<PlanResult>, Receiver<PlanResult>) = mpsc::channel();
    thread::spawn(move || {
        bot_io_loop(state_sender, plan_receiver, BotIoConfig::default());
    });
    bot_logic_loop(plan_sender, state_receiver);
}

fn run_bot_test() {
    let (state_sender, state_receiver): (
        Sender<(GameState, BotState)>,
        Receiver<(GameState, BotState)>,
    ) = mpsc::channel();
    let (plan_sender, plan_receiver): (Sender<PlanResult>, Receiver<PlanResult>) = mpsc::channel();
    thread::spawn(move || {
        let _batmobile = rlbot::PlayerLoadout::new().car_id(803);
        let fennec = rlbot::PlayerLoadout::new().car_id(4284);

        let mut match_settings =
            rlbot::MatchSettings::new().player_configurations(vec![rlbot::PlayerConfiguration::new(
                rlbot::PlayerClass::RLBotPlayer,
                "Brick Test",
                0,
            )
            .loadout(fennec)]);

        match_settings.mutator_settings =
            rlbot::MutatorSettings::new().
            match_length(rlbot::MatchLength::Unlimited).
            boost_option(rlbot::BoostOption::Unlimited_Boost);

        let bot_io_config = BotIoConfig {
            manipulator: Some(bot_test_manipulator),
            print_turn_errors: true,
            start_match: true,
            match_settings: Some(match_settings),
        };

        bot_io_loop(state_sender, plan_receiver, bot_io_config);
    });
    bot_logic_loop_test(plan_sender, state_receiver);
}

fn bot_test_manipulator(rlbot: &rlbot::RLBot, game_state: &GameState, bot: &BotState) {
    // TODO implement
}

fn bot_logic_loop(sender: Sender<PlanResult>, receiver: Receiver<(GameState, BotState)>) {
    let mut model = brain::get_model();
    loop {
        let (mut game, mut bot) = receiver.recv().expect("Couldn't receive game state");

        // make sure we have the latest, drop earlier states
        while let Ok((g, b)) = receiver.try_recv() {
            game = g;
            bot = b;
        }

        let plan_result = brain::play::play(&mut model, &game, &mut bot);
        sender
            .send(plan_result)
            .expect("Failed to send plan result");
    }
}

fn bot_test_plan<H: brain::HeuristicModel>(model: &mut H, game: &GameState, bot: &mut BotState) -> PlanResult {
    ////let plan = offset_forward_plan(&game.player);
    //let plan = square_plan(&game.player);
    //PlanResult {
    //    plan: plan,
    //    desired: DesiredContact::default(),
    //    visualization_lines: vec![],
    //    visualization_points: vec![],
    //})
    brain::play::play(model, &game, bot)
}

fn bot_logic_loop_test(sender: Sender<PlanResult>, receiver: Receiver<(GameState, BotState)>) {
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();
    let mut model = brain::get_model();
    let mut ratelimiter = ratelimit::Builder::new()
        .interval(Duration::from_millis(1000 / 120)) // bot limited to 120 fps
        .build();

    loop {
        let (mut game, mut bot) = receiver.recv().expect("Couldn't receive game state");

        // make sure we have the latest, drop earlier states
        while let Ok((g, b)) = receiver.try_recv() {
            game = g;
            bot = b;
        }

        update_gamepad(&mut gilrs, &mut gamepad);
        if !gamepad.select_toggled {
            ratelimiter.wait();
            continue;
        }

        sender
            .send(bot_test_plan(&mut model, &game, &mut bot))
            .expect("Failed to send plan result");

        ratelimiter.wait();

        // below is for figuring out good PD control parameters, with a canned plan
        //
        // let mut square_errors = vec![];
        // loop {
        //     let (game, _bot) = receiver.recv().expect("Coudln't receive game state");

        //     let closest_index = brain::play::closest_plan_index(&game.player, &plan);
        //     plan = plan.split_off(closest_index);

        //     let square_error = (plan[0].0.position - &game.player.position)
        //         .norm()
        //         .powf(2.0);
        //     square_errors.push(square_error);

        //     if plan.len() <= 2 {
        //         break;
        //     }

        //     update_gamepad(&mut gilrs, &mut gamepad);
        //     if !gamepad.select_toggled {
        //         break;
        //     }

        //     let square_error = (plan[0].0.position - &game.player.position)
        //         .norm()
        //         .powf(2.0);
        //     square_errors.push(square_error);

        //     ratelimiter.wait();
        // }
        // println!("========================================");
        // println!("Steps: {}", square_errors.len());
        // println!(
        //     "RMS Error: {}",
        //     (square_errors.iter().sum::<f32>() / (square_errors.len() as f32)).sqrt()
        // );
        // println!("========================================");
    }
}

fn bot_io_loop(sender: Sender<(GameState, BotState)>, receiver: Receiver<PlanResult>, bot_io_config: BotIoConfig) {
    let mut bot = BotState::default();
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();
    let rlbot = rlbot::init().expect("rlbot init failed");
    let mut physicist = rlbot.physicist();

    let mut ratelimiter = ratelimit::Builder::new()
        .interval(Duration::from_millis(1000 / 120)) // bot io limited to 120 fps
        .build();

    if bot_io_config.start_match {
        if let Some(match_settings) = bot_io_config.match_settings {
            rlbot.start_match(&match_settings).expect("Failed to start match");
            rlbot.wait_for_match_start().expect("Failed waiting for match start");
            println!("Started Match!");
        } else {
            eprintln!("WARNING: Trying to start a match, but no match settings given. Ignoring start_match option.");
        }
    }

    loop {
        // FIXME find out the new method for setting the correct player index since the tcp server
        // thing is gone afaik. just a command line argument now perhaps?
        let player_index = 0;

        while let Ok(tick) = physicist.next_flat() {
            update_game_state(&mut GAME_STATE.write().unwrap(), &tick, player_index);

            send_to_bot_logic(&sender, &bot);
            ratelimiter.wait();

            // make sure we have the latest results in case there are multiple, though note we may save
            // the plan from an earlier run if it happens to be the best one
            while let Ok(plan_result) = receiver.try_recv() {
                update_bot_state(&GAME_STATE.read().unwrap(), &mut bot, &plan_result);
                match update_in_game_visualization(&rlbot, &bot, &plan_result) {
                    Ok(_) => {},
                    Err(e) => eprintln!("Failed rendering to rlbot: {}", e),
                };
            }

            // remove part of plan that is no longer relevant since we've already passed it
            if let Some(ref mut plan) = bot.plan {
                let closest_index = brain::play::closest_plan_index(&GAME_STATE.read().unwrap().player, &plan);
                //println!("closest index: {}, plan len: {}", closest_index, plan.len());
                *plan = plan.split_off(closest_index);
            } else {
                //println!("no plan");
            }

            // the difference between these is the frame lag
            let input_frame = GAME_STATE.read().unwrap().input_frame;
            let frame = GAME_STATE.read().unwrap().frame;

            update_gamepad(&mut gilrs, &mut gamepad);
            let mut input = if gamepad.select_toggled {
                brain::play::next_input(&GAME_STATE.read().unwrap().player, &mut bot)
            } else {
                human_input(&gamepad)
            };

            // allows tracking the frame lag using a side-channel in the player inputs
            {
                let game = GAME_STATE.read().unwrap();
                set_frame_metadata(game.frame, &mut input);
            }

            bot.controller_history.push_back((frame, (&input).into()));
            if bot.controller_history.len() > 100 {
                // keep last 10
                bot.controller_history = bot.controller_history.split_off(90);
            }

            if bot_io_config.print_turn_errors {
                if bot.turn_errors.len() % 20 == 0 && bot.turn_errors.len() >= 20 {
                    let errors = bot.turn_errors.iter().take(20).cloned().collect::<Vec<_>>();
                    //let sum = errors.iter().map(f32::abs).sum::<f32>();
                    //let avg = sum / 20.0;
                    let squared_sum = errors.iter().map(|x| x.abs() * x.abs()).sum::<f32>();
                    let rms = (squared_sum / 20.0).powf(0.5);
                    let max = errors.iter().cloned().fold(-1.0f32/0.0 /* -inf */, f32::max);
                    let min = errors.iter().cloned().fold(1.0f32/0.0 /* inf */, f32::min);
                    println!("rms: {}, min: {}, max: {}", rms, min, max);
                }
            }

            rlbot
                .update_player_input(player_index as i32, &input)
                .expect("update_player_input failed");

            // let's some kind of testing mode update the game state
            if let Some(manipulator) = bot_io_config.manipulator {
                manipulator(&rlbot, &GAME_STATE.read().unwrap(), &bot);
            }
        }
    }
}

fn plan_is_valid(game: &GameState, plan: &Plan) -> bool {
    let closest_index = brain::play::closest_plan_index(&game.player, &plan);
    if let Some((player, _, _)) = plan.get(closest_index) {
        // TODO tune
        (player.position - game.player.position).norm() < 30.0 && (player.velocity - game.player.velocity).norm() < 200.0
    } else {
        false
    }
}

fn update_bot_state(game: &GameState, bot: &mut BotState, plan_result: &PlanResult) {
    if let Some(ref new_plan) = plan_result.plan {
        if let Some(ref existing_plan) = bot.plan {
            let new_plan_cost = new_plan.iter().map(|(_, _, cost)| cost).sum::<f32>();

            let closest_index = brain::play::closest_plan_index(&game.player, &existing_plan);
            let existing_plan_cost = existing_plan.iter().enumerate().filter(|(index, _val)| {
                *index > closest_index
            }).map(|(_index, (_, _, cost))| cost).sum::<f32>();

            // // bail, we got a worse plan!
            // if new_plan_cost >= existing_plan_cost && plan_is_valid(&game, &existing_plan) {
            //     println!("bailing longer plan! existing_plan_cost: {}, new_plan_cost: {}", existing_plan_cost, new_plan_cost);
            //     return;
            // }

            let existing_diff = bot.cost_diff.abs();
            let new_diff = plan_result.cost_diff.abs();
            if new_diff > existing_diff && plan_is_valid(&game, &existing_plan) {
                //println!("bailing less accurate plan! existing_diff: {}, new_diff: {}", existing_diff, new_diff);
                return;
            } else if new_diff == existing_diff && new_plan_cost >= existing_plan_cost && plan_is_valid(&game, &existing_plan) {
                //println!("bailing longer plan! existing_plan_cost: {}, new_plan_cost: {}", existing_plan_cost, new_plan_cost);
                return;
            }
        }

        //let cost = new_plan.iter().map(|(_, _, cost)| cost).sum::<f32>();
        //println!("new best plan! cost: {}", cost);
        bot.plan = Some(new_plan.clone());
        bot.cost_diff = plan_result.cost_diff;
        bot.turn_errors.clear();
    }
}

fn plan_lines(plan: &Plan, color: Point3<f32>) -> Vec<(Point3<f32>, Point3<f32>, Point3<f32>)> {
    let mut lines = Vec::with_capacity(plan.len());
    let pos = plan
        .get(0)
        .map(|(p, _, _)| p.position)
        .unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));
    let mut last_point = Point3::new(pos.x, pos.y, pos.z);
    for (ps, _, _) in plan {
        let point = Point3::new(ps.position.x, ps.position.y, ps.position.z + 0.1);
        lines.push((last_point.clone(), point.clone(), color));
        last_point = point;
    }
    lines
}

fn update_simulation_visualization(bot: &BotState, plan_result: &PlanResult) {
    let PlanResult {
        plan,
        visualization_lines: lines,
        visualization_points: points,
        ..
    } = plan_result;

    let mut visualization_lines = LINES.write().unwrap();
    visualization_lines.clear();

    // lines directly from plan result
    visualization_lines.append(&mut lines.clone());

    // white line showing best planned path
    if let Some(ref plan) = bot.plan {
        visualization_lines.append(&mut plan_lines(&plan, Point3::new(1.0, 1.0, 1.0)));
    }

    // blue line showing most recently calculated path
    if let Some(plan) = plan {
        visualization_lines.append(&mut plan_lines(&plan, Point3::new(0.0, 1.0, 1.0)));
    }

    let mut visualization_points = POINTS.write().unwrap();
    visualization_points.clear();
    visualization_points.append(&mut points.clone());
}

const VISUALIZATION_GROUP_ID: i32 = 7323; // trying to not overlap with other bots, though idk if they CAN overlap

fn draw_lines(rlbot: &rlbot::RLBot, lines: &Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>, chunk_num: &mut i32) -> Result<(), Box<dyn Error>> {
    for chunk in lines.chunks(200) {
        // TODO in case of multiple bricks, add player index * 1000 to the group id
        let mut group = rlbot.begin_render_group(VISUALIZATION_GROUP_ID + *chunk_num);

        for &l in chunk.iter() {
            let p1 = l.0;
            let p2 = l.1;
            let p3 = l.2;
            let color = group.color_rgb((255.0 * p3.x) as u8, (255.0 * p3.y) as u8, (255.0 * p3.z) as u8);
            group.draw_line_3d((-p1.x, p1.y, p1.z), (-p2.x, p2.y, p2.z), color);
        }

        group.render()?;

        *chunk_num += 1;
    }

    Ok(())
}

fn update_in_game_visualization(rlbot: &rlbot::RLBot, bot: &BotState, plan_result: &PlanResult) -> Result<(), Box<dyn Error>> {
    let PlanResult { plan, visualization_lines, ..  } = plan_result;
    let mut chunk_num = 0;

    // white line showing best planned path
    if let Some(ref plan) = bot.plan {
        draw_lines(&rlbot, &plan_lines(&plan, Point3::new(1.0, 1.0, 1.0)), &mut chunk_num)?;
    }

    if let Some(plan) = plan {
        // turquoise line showing most recently calculated path
        draw_lines(&rlbot, &plan_lines(&plan, Point3::new(0.0, 1.0, 1.0)), &mut chunk_num)?;

        // visualization of work done to find this path
        // XXX maybe this is too many lines, cos it doesn't render much of this
        // draw_lines(&rlbot, &visualization_lines, &mut chunk_num)?;
    }

    Ok(())
}

fn send_to_bot_logic(sender: &Sender<(GameState, BotState)>, bot: &BotState) {
    let game = (*GAME_STATE.read().unwrap()).clone();
    let bot = bot.clone();
    sender
        .send((game, bot))
        .expect("Sending to bot logic failed");
}

fn turn_plan(current: &PlayerState, angle: f32) -> Plan {
    let mut plan = vec![];
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let desired_heading = Rotation3::from_euler_angles(0.0, 0.0, angle) * current_heading;
    let mut turn_controller = BrickControllerState::new();
    turn_controller.throttle = Throttle::Forward;
    turn_controller.steer = if angle < 0.0 {
        Steer::Right
    } else {
        Steer::Left
    };

    let mut straight_controller = BrickControllerState::new();
    straight_controller.throttle = Throttle::Forward;

    const TURN_DURATION: f32 = 2.0 * TICK;
    // straighten out for zero angular velocity at end, hopefully 16 ticks is enough?
    const STRAIGHT_DURATION: f32 = 16.0 * TICK;

    // iterate till dot product is maximized (ie we match the desired heading)
    let mut last_dot = std::f32::MIN;
    let mut player = current.clone();
    loop {
        let turn_player = brain::predict::player::next_player_state(&player, &turn_controller, TURN_DURATION).unwrap();
        let turn_heading = turn_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let turn_dot = na::Matrix::dot(&turn_heading, &desired_heading);

        // straight duration is much longer than turn duration
        let long_turn_player = brain::predict::player::next_player_state(&turn_player, &turn_controller, STRAIGHT_DURATION).unwrap();
        let long_turn_heading = long_turn_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let long_turn_dot = na::Matrix::dot(&long_turn_heading, &desired_heading);

        let turn_then_straight_player = brain::predict::player::next_player_state(&turn_player, &straight_controller, STRAIGHT_DURATION).unwrap();
        let turn_then_straight_heading = turn_then_straight_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let turn_then_straight_dot = na::Matrix::dot(&turn_then_straight_heading, &desired_heading);

        let straight_player = brain::predict::player::next_player_state(&player, &straight_controller, STRAIGHT_DURATION).unwrap();
        let straight_heading = straight_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let straight_dot = na::Matrix::dot(&straight_heading, &desired_heading);

        // if we aren't overshooting, add a turn
        if turn_dot > last_dot && long_turn_dot > turn_then_straight_dot {
            plan.push((turn_player, turn_controller, TURN_DURATION));
            player = turn_player;
            last_dot = turn_dot;
        } else if turn_then_straight_dot > last_dot && turn_then_straight_dot > straight_dot {
            plan.push((turn_player, turn_controller, TURN_DURATION));
            plan.push((turn_then_straight_player, straight_controller, STRAIGHT_DURATION));
            player = turn_then_straight_player;
            last_dot = turn_then_straight_dot;
        } else if straight_dot > last_dot + 0.001 {
            plan.push((straight_player, straight_controller, STRAIGHT_DURATION));
            player = straight_player;
            last_dot = straight_dot;
        } else {
            break;
        }
    }

    plan
}

fn forward_plan(current: &PlayerState, distance: f32) -> Plan {
    let mut plan = vec![];

    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;

    let mut player = current.clone();
    while (player.position - current.position).norm() < distance {
        player = brain::predict::player::next_player_state(&player, &controller, 16.0 * TICK).unwrap(); // FIXME step_duration input
        plan.push((player, controller, 16.0 * TICK));
    }
    plan
}

fn square_plan(current: &PlayerState) -> Plan {
    let mut plan = vec![];
    let mut player = current.clone();
    let max_throttle_speed = 1545.0; // FIXME put in common lib
    player.velocity = max_throttle_speed * Unit::new_normalize(player.velocity).into_inner();
    plan.push((player, BrickControllerState::new(), 0.0));
    for _ in 0..4 {
        let mut plan_part = forward_plan(&plan[plan.len() - 1].0, 1000.0);
        plan.append(&mut plan_part);
        let mut plan_part = turn_plan(&plan[plan.len() - 1].0, -PI / 2.0);
        plan.append(&mut plan_part);
    }
    plan
}

fn offset_forward_plan(current: &PlayerState) -> Plan {
    let mut offset_player = current.clone();
    let heading = offset_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, -PI / 2.0);
    let right = clockwise_90_rotation * heading;
    offset_player.position += 200.0 * right;

    forward_plan(&offset_player, 4000.0)
}

fn simulate_over_time() {
    thread::sleep(Duration::from_millis(5000));
    let initial_game_state: GameState;
    let mut bot = BotState::default();
    let mut model = brain::get_model();

    let mut ratelimiter = ratelimit::Builder::new()
        .interval(Duration::from_millis(1000 / 120)) // simulation limited to 120 fps
        .build();

    {
        let mut game_state = GAME_STATE.write().unwrap();
        game_state.ball.position = Vector3::new(0.0, 0.0, BALL_COLLISION_RADIUS);
        game_state.player.position = Vector3::new(0.0, 4000.0, 0.0);
        game_state.player.velocity = Vector3::new(0.0, 0.0, 0.0);

        // left
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        // up
        game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI / 2.0);
        // down
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        // right
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI);

        initial_game_state = game_state.clone();
    }

    loop {
        {
            let game_state = GAME_STATE.read().unwrap();
            let plan_result = brain::play::play(&mut model, &game_state, &mut bot);
            update_bot_state(&game_state, &mut bot, &plan_result);
            update_simulation_visualization(&bot, &plan_result);
            // this pauses the simulation forever when no plan is found
            // if plan_result.plan.is_none() {
            //     thread::sleep(Duration::from_millis(5000)));
            //     continue;
            // }
        }

        if let Some(plan) = bot.plan.clone() {
            let mut game_state = GAME_STATE.write().unwrap();
            let i = brain::play::closest_plan_index(&game_state.player, &plan);
            if plan.len() >= i + 2 {
                game_state.player = plan[i + 1].0;
                // TODO move the ball too. ball velocity is zero for now
            } else {
                // we're at the goal, so start over
                *game_state = initial_game_state.clone();
                bot.plan = None;
            }
        } else {
            // no plan, just try again with a slightly different position
            let mut game_state = GAME_STATE.write().unwrap();
            game_state.player.position += Vector3::new(20.0, 20.0, 0.0);
        }

        ratelimiter.wait();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.parse())
        .unwrap_or_else(|e| e.exit());

    let test_bot = args.get_bool("--bot-test");
    if args.get_bool("--bot") || test_bot {
        thread::spawn(move || loop {
            let t = thread::spawn(move || {
                if test_bot {
                    panic::catch_unwind(run_bot_test).expect("Panic catch unwind failed");
                } else {
                    panic::catch_unwind(run_bot).expect("Panic catch unwind failed");
                }
            });
            t.join()
                .expect_err("The bot thread should only end if panic, but it didn't panic.");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

            thread::sleep(Duration::from_millis(1000));
        });
    } else if args.get_bool("--simulate") {
        thread::spawn(simulate_over_time);
    } else {
        panic!("Must provide --bot, --bot-test or --simulate");
    }

    if args.get_bool("--simulate") {
        run_visualization();
    } else {
        thread::park();
    }

    Ok(())
}
