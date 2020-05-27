use heuristic::HeuristicModel;
use na::{Point3, Unit, Vector3};
use predict;
use state::*;
use std::cmp::Ordering;
use std::error::Error;
use std::f32::consts::PI;

use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;
use itertools;
use std::collections::BinaryHeap;
use std::mem;
use std::usize;

use fnv::FnvHasher;
use std::hash::BuildHasherDefault;
type MyHasher = BuildHasherDefault<FnvHasher>;

lazy_static! {
    pub static ref GROUND_CONTROL_BRANCHES: Vec<BrickControllerState> = {
        // boost
        let mut left_boost = BrickControllerState::new();
        left_boost.boost = true;
        left_boost.steer = Steer::Left;

        let mut right_boost = BrickControllerState::new();
        right_boost.boost = true;
        right_boost.steer = Steer::Right;

        let mut straight_boost = BrickControllerState::new();
        straight_boost.boost = true;

        // throttle
        let mut left_throttle = BrickControllerState::new();
        left_throttle.throttle = Throttle::Forward;
        left_throttle.steer = Steer::Left;

        let mut right_throttle = BrickControllerState::new();
        right_throttle.throttle = Throttle::Forward;
        right_throttle.steer = Steer::Right;

        let mut straight_throttle = BrickControllerState::new();
        straight_throttle.throttle = Throttle::Forward;

        // drift + boost
        let mut left_drift_boost = left_boost.clone();
        left_drift_boost.handbrake = true;

        let mut right_drift_boost = right_boost.clone();
        right_drift_boost.handbrake = true;

        let mut straight_drift_boost = straight_boost.clone();
        straight_drift_boost.handbrake = true;

        // drift + throttle
        let mut left_drift_throttle = left_throttle.clone();
        left_drift_throttle.handbrake = true;

        let mut right_drift_throttle = right_throttle.clone();
        right_drift_throttle.handbrake = true;

        let mut straight_drift_throttle = straight_throttle.clone();
        straight_drift_throttle.handbrake = true;

        // idle
        let mut left_idle = BrickControllerState::new();
        left_idle.steer = Steer::Left;

        let mut right_idle = BrickControllerState::new();
        right_idle.steer = Steer::Right;

        let _straight_idle = BrickControllerState::new();

        // reverse
        let mut left_reverse = BrickControllerState::new();
        left_reverse.throttle = Throttle::Reverse;
        left_reverse.steer = Steer::Left;

        let mut right_reverse = BrickControllerState::new();
        right_reverse.throttle = Throttle::Reverse;
        right_reverse.steer = Steer::Right;

        let mut straight_reverse = BrickControllerState::new();
        straight_reverse.throttle = Throttle::Reverse;

        vec![
            left_boost,
            right_boost,
            straight_boost,

            //left_drift_boost,
            //right_drift_boost,
            //straight_drift_boost,

            left_throttle,
            right_throttle,
            straight_throttle,

            //left_drift_throttle,
            //right_drift_throttle,
            //straight_drift_throttle,

            //left_idle,
            //right_idle,
            //straight_idle,

            //left_drift_idle,
            //right_drift_idle,
            //straight_drift_idle,

            //left_reverse,
            //right_reverse,
            //straight_reverse,

            //left_drift_reverse,
            //right_drift_reverse,
            //straight_drift_reverse,
        ]
    };
}

const TICKS_PER_STEP: i32 = 1;
const EXPLODED_STEP_DURATION: f32 = TICKS_PER_STEP as f32 * TICK;

/// wrapper around hybrid_a_star for convenience and some extra smarts. meant to be used by the
/// live bot or bot simulation only, as it configures the serch paramters to favor speed over
/// accuracy/optimality.
// TODO maybe we should take the entire gamestate instead. we also need a history component, ie BotState
pub extern "C" fn plan<H: HeuristicModel>(
    model: &mut H,
    player: &PlayerState,
    ball_trajectory: &[BallState],
    initial_ball_trajectory_index: usize,
    desired: &DesiredContact,
    cost_to_strive_for: f32,
    _last_plan: Option<&Plan>,
) -> PlanResult {
    let mut config = SearchConfig::default();

    // speed over optimality
    config.scale_heuristic = 10.0;
    config.max_iterations = 500;

    // if we have a perfectly good plan, we can use it as benchmark of when to stop looking
    // further, since if we get a worse plan now we'll ignore it.
    // TODO let's check if the last plan is still valid before doing this
    // XXX this can potentially block finding a higher quality (ie hits the ball accurately) plan,
    // so disabling for now
    // if let Some(last_plan) = last_plan {
    //     config.max_cost = (20.0 + last_plan.len() as f32) * EXPLODED_STEP_DURATION;
    // }

    let mut plan_result = hybrid_a_star(model, player, ball_trajectory, initial_ball_trajectory_index, desired, cost_to_strive_for, &config);

    match explode_plan(&plan_result.plan) {
        Ok(exploded) => plan_result.plan = exploded,
        Err(e) => {
            eprintln!("Exploding plan failed: {}", e);
            plan_result.plan = None;
        }
    };

    plan_result
}

/// modifies the plan to use finer-grained steps
pub fn explode_plan(plan: &Option<Plan>) -> Result<Option<Plan>, Box<dyn Error>> {
    if let Some(ref plan) = plan {
        if plan.get(0).is_none() {
            return Ok(None)
        }
        let mut exploded_plan = Vec::with_capacity(plan.len()); // will be at least this long
        exploded_plan.push(plan[0]);

        // for every plan segment, we expand within that segment. using small ticks repeated causes
        // the errors to accumulate rapidly, so we start over with each plan segment starting with
        // a more accurate base value
        for i in 1..plan.len() {
            let num_ticks = (plan[i].2 / TICK).round() as i32;
            let num_steps = num_ticks / TICKS_PER_STEP;
            let remaining_ticks = num_ticks % TICKS_PER_STEP;
            let mut last_player = plan[i - 1].0;
            let controller = plan[i].1;

            for j in 1..=num_steps {
                // when stepping by single ticks, still use 2-tick calculations when possible for better accuracy
                let (step, last) = if EXPLODED_STEP_DURATION == TICK && j % 2 == 0 {
                    if j == 2 {
                        // use the more accurate value as a base to start expansion from
                        (TICK * 2.0, &plan[i - 1].0)
                    } else {
                        (TICK * 2.0, &exploded_plan[exploded_plan.len() - 2].0)
                    }
                } else {
                    (EXPLODED_STEP_DURATION, &last_player)
                };
                let next_player = predict::player::next_player_state(
                    last,
                    &controller,
                    step,
                )?;
                exploded_plan.push((next_player, controller, step));
                last_player = next_player;
            }

            // if we are exploding into 2-tick steps, if there is a leftover tick we still want to
            // include/exlode it
            if remaining_ticks > 0 {
                for _j in 1..=remaining_ticks {
                    let next_player = predict::player::next_player_state(
                        &last_player,
                        &controller,
                        TICK,
                    )?;
                    exploded_plan.push((next_player, controller, TICK));
                    last_player = next_player;
                }
            }
        }

        //println!("===================================");
        //println!("PLAN: {:?}", plan.iter().map(|(p, c, s)| {
        //    (p.position.x, p.position.y, p.local_velocity().x, p.local_velocity().y, p.rotation.euler_angles().2, p.angular_velocity.z, c.steer, c.boost, s)
        //}).collect::<Vec<_>>());
        //println!("-----------------------------------");
        //println!("exploded: {:?}", exploded_plan.iter().map(|(p, c, s)| {
        //    (p.position.x, p.position.y, p.velocity.x, p.velocity.y, p.rotation.euler_angles().2, p.angular_velocity.z, c.steer, s)
        //}).collect::<Vec<_>>());
        //println!("===================================");

        Ok(Some(exploded_plan))
    } else {
        Ok(None)
    }
}

#[derive(Clone, Debug)]
struct PlayerVertex {
    cost_so_far: f32,
    player: PlayerState,
    /// the controller state in previous step that lead to this player vertex
    prev_controller: BrickControllerState,
    ball_trajectory_index: usize,
    step_duration: f32,
    parent_index: usize,
    parent_is_secondary: bool,
}

#[derive(Debug)]
struct SmallestCostHolder {
    estimated_cost: f32,
    cost_so_far: f32,
    index: usize,
    is_secondary: bool,
}

impl PartialEq for SmallestCostHolder {
    fn eq(&self, other: &SmallestCostHolder) -> bool {
        self.cost_so_far == other.cost_so_far
    }
}
impl Eq for SmallestCostHolder {}

impl Ord for SmallestCostHolder {
    fn cmp(&self, other: &SmallestCostHolder) -> Ordering {
        // we flip the ordering here, to make the heap a min-heap
        if self.estimated_cost == other.estimated_cost {
            Ordering::Equal
        } else if self.estimated_cost < other.estimated_cost {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl<'a> PartialOrd for SmallestCostHolder {
    fn partial_cmp(&self, other: &SmallestCostHolder) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// XXX is the use of i16 here actually helping?
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct RoundedPlayerState {
    x: i16,
    y: i16,
    z: i16,
    //vx: i16,
    //vy: i16,
    //vz: i16,
    //roll: i16,
    //pitch: i16,
    yaw: i16,
}

type ParentsMap = IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher>;

pub fn hybrid_a_star<H: HeuristicModel>(
    model: &mut H,
    current: &PlayerState,
    ball_trajectory: &[BallState],
    initial_ball_trajectory_index: usize,
    desired: &DesiredContact,
    cost_to_strive_for: f32,
    config: &SearchConfig,
) -> PlanResult {
    // TODO take this fn as an argument, so different actions can have different goal_reached evaluation functions
    let is_ball_hit_towards_goal = |ball_trajectory: &[BallState], player: &PlayerState, next_vertex: &PlayerVertex, controller: &BrickControllerState, time_step: f32| -> Option<(PlayerState, BallState, f32)> {
        let index = ((next_vertex.cost_so_far - next_vertex.step_duration) / TICK).round() as usize;
        if let Some((colliding_player, colliding_ball, collision_point, collision_time)) = predict::player::get_collision(&ball_trajectory[index..], player, controller, time_step) {
            match predict::ball::calculate_hit(&colliding_ball, &colliding_player, &collision_point) {
                Ok(next_ball) => {
                    if predict::ball::trajectory_enters_soccar_goal(&next_ball) {
                        // println!("collision point: {:?}, ball velocity: {:?}", collision_point, next_ball.velocity);
                        Some((colliding_player, colliding_ball, collision_time))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    eprintln!("Error calculating ball hit: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    let mut visualization_lines = vec![];

    #[allow(unused_mut)]
    let mut visualization_points = vec![];

    // we can't fly yet :(
    if ball_trajectory[initial_ball_trajectory_index].position.z - BALL_COLLISION_RADIUS > CAR_DIMENSIONS.z + CAR_OFFSET.z {
        return PlanResult::default();
    }

    // sets up the model for this particular prediction. it can do some calculations upfront here
    // instead of over and over again for each prediction.
    model.configure(&desired, config.scale_heuristic);

    let mut to_see: BinaryHeap<SmallestCostHolder> = BinaryHeap::new();
    let mut parents: ParentsMap = IndexMap::default();

    // buffers to avoid re-allocating in a loop
    let mut new_vertices = vec![];
    let mut new_players = vec![];
    let mut cur_heuristic_costs: Vec<f32> = vec![];
    let mut prev_heuristic_costs: Vec<f32> = vec![];
    let mut next_heuristic_costs: Vec<f32> = vec![];
    let mut single_heuristic_cost: Vec<f32> = vec![0.0];

    model
        .heuristic(&[current.clone()], &mut single_heuristic_cost[0..1])
        .expect("Heuristic failed initial!");

    to_see.push(SmallestCostHolder {
        estimated_cost: single_heuristic_cost[0],
        cost_so_far: 0.0,
        index: 0,
        is_secondary: false,
    });

    let start = PlayerVertex {
        player: current.clone(),
        cost_so_far: 0.0,
        prev_controller: BrickControllerState::new(),
        ball_trajectory_index: initial_ball_trajectory_index,
        step_duration: 0.0,
        parent_index: usize::MAX,
        parent_is_secondary: false,
    };

    parents.insert(
        round_player_state(&current, config.step_duration, current.velocity.norm()),
        (start, None),
    );

    let mut num_iterations = 0;

    while let Some(SmallestCostHolder {
        estimated_cost,
        cost_so_far,
        index,
        is_secondary,
        ..
    }) = to_see.pop()
    {
        // avoid an infinite graph search
        if cost_so_far > config.max_cost {
            //println!("short circuit, hit max cost!");
            break;
        }

        // HACK avoid very large searches completely
        num_iterations += 1;
        if num_iterations > config.max_iterations {
            //println!("short circuit, too many iterations!");
            break;
        }

        let dur = if estimated_cost - cost_so_far > 2.0 {
            // if we're really far... yeah just make it super coarse to make it tractable, and the
            // search config has no control over this for now
            32.0 * TICK
        } else {
            config.step_duration
        };

        let line_start;
        {
            let (_, (v1, maybe_v2)) = parents
                .get_index(index)
                .expect("missing index in parents, shouldn't be possible");
            let vertex = if is_secondary {
                maybe_v2.as_ref().unwrap()
            } else {
                v1
            };
            line_start = vertex.player.position;

            let mut parent_player;
            if let Some((_, (parent_v1, maybe_parent_v2))) = parents.get_index(vertex.parent_index)
            {
                let parent_vertex = if vertex.parent_is_secondary {
                    maybe_parent_v2.as_ref().unwrap()
                } else {
                    parent_v1
                };
                parent_player = parent_vertex.player;
            } else {
                // no parent, this can only happen on first expansion. we need to construct a fake
                // one just so that goal detection works
                parent_player = vertex.player.clone();

                // avoid divide by zero during direction vector inversion
                parent_player.position.x += 0.1;
                parent_player.position.y += 0.1;
                parent_player.position.z += 0.1;
            }

            if let Some((player, ball, cost)) = player_goal_reached(&vertex, &parent_player, ball_trajectory, &vertex.prev_controller, config.step_duration, is_ball_hit_towards_goal) {
                let plan = reverse_path(&parents, index, is_secondary, &player, cost);

                let total_cost = plan.iter().map(|(_, _, cost)| cost).sum::<f32>();
                // println!(
                //     "omg reached! step size: {} | expansions: {} | cost: {}",
                //     config.step_duration * 120.0,
                //     visualization_lines.len(),
                //     total_cost,
                // );
                return PlanResult {
                    plan: Some(plan),
                    planned_ball: Some(ball),
                    source_frame: 0, // caller sets it
                    cost_diff: total_cost - cost_to_strive_for,
                    ball_trajectory: ball_trajectory.iter().cloned().collect::<Vec<_>>(),
                    visualization_lines,
                    visualization_points,
                };
            } else if coarse_collision(&vertex, &parent_player, &ball_trajectory[vertex.ball_trajectory_index]) {
                // if we hit the ball but we didn't reach the goal, we skip instead of expanding
                // this vertex
                let index = ((vertex.cost_so_far - vertex.step_duration) / TICK).round() as usize;
                if predict::player::get_collision(&ball_trajectory[index..], &parent_player, &vertex.prev_controller, config.step_duration).is_some() {
                    continue
                }
            }

            // We may have inserted a node several times into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            //
            // NOTE: this also achieves the same thing as checking if we are in the closed set
            if cost_so_far > vertex.cost_so_far {
                continue;
            }

            expand_vertex(index, is_secondary, &vertex, &mut new_vertices, dur, config.custom_filter)
        };

        new_players.clear();
        new_players.extend(new_vertices.iter().map(|v| v.player));

        if let Some(first_new_vertex) = new_vertices.get(0) {
            let index = first_new_vertex.ball_trajectory_index;
            set_heuristic_costs(model, &new_players, &mut cur_heuristic_costs, &ball_trajectory, index);
            set_heuristic_costs(model, &new_players, &mut prev_heuristic_costs, &ball_trajectory, index.wrapping_sub(1));
            set_heuristic_costs(model, &new_players, &mut next_heuristic_costs, &ball_trajectory, index + 1);
        }

        let mut heuristic_index = 0;
        for mut new_vertex in new_vertices.drain(0..) {
            let new_vertex_rounded =
                round_player_state(&new_vertex.player, dur, new_vertex.player.velocity.norm());
            let new_cost_so_far = new_vertex.cost_so_far;
            let new_index;
            let mut new_is_secondary = false;
            let line_end = new_vertex.player.position;
            let mut new_estimated_cost = 0.0;
            let i = heuristic_index;
            heuristic_index += 1;

            let cur_diff = (cur_heuristic_costs[i] - TICK * new_vertex.ball_trajectory_index as f32).abs();
            let prev_diff = (prev_heuristic_costs[i] - TICK * (new_vertex.ball_trajectory_index as f32 - 1.0)).abs();
            let next_diff = (next_heuristic_costs[i] - TICK * (new_vertex.ball_trajectory_index as f32 + 1.0)).abs();
            let heuristic_cost =
                if cur_diff < prev_diff && cur_diff <= next_diff {
                    cur_heuristic_costs[i]
                } else if prev_diff <= next_diff {
                    new_vertex.ball_trajectory_index = new_vertex.ball_trajectory_index.wrapping_sub(1);
                    prev_heuristic_costs[i]
                } else {
                    new_vertex.ball_trajectory_index += 1;
                    next_heuristic_costs[i]
                };

            match parents.entry(new_vertex_rounded) {
                Vacant(e) => {
                    new_index = e.index();
                    new_estimated_cost = new_cost_so_far + heuristic_cost;
                    e.insert((new_vertex, None));
                }
                Occupied(mut e) => {
                    new_index = e.index();
                    new_is_secondary = true;
                    let insertable;
                    match e.get() {
                        (existing_vertex, None) => {
                            // basically just like the vacant case
                            new_estimated_cost = new_cost_so_far + heuristic_cost;
                            // TODO-perf avoid the clone here. nll? worst-case, can use mem::replace with an enum
                            insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                        }
                        (existing_vertex, Some(existing_secondary_vertex)) => {
                            // FIXME we seem to only be checking/replacing the secondary vertex..
                            // what about the primary?
                            let mut new_cost_is_lower =
                                existing_secondary_vertex.cost_so_far > new_vertex.cost_so_far;
                            if new_cost_is_lower {
                                // turns out that the index invalidation is a problem in general
                                // with the IndexMap approach when replacing occupied entries: we
                                // end up with some nonsensical plan jumps otherwise. instead,
                                // let's limit replacements to siblings. this does mean we may miss
                                // out on the best path, but at least doesn't give us corrupted
                                // paths! if this causes us to lose too many good paths, we'll need
                                // to reconsider the use of the IndexMap
                                if existing_secondary_vertex.parent_index != new_vertex.parent_index
                                {
                                    continue;
                                }

                                new_estimated_cost = new_cost_so_far + heuristic_cost;
                            } else if e.index() == new_vertex.parent_index
                                || new_vertex.parent_index == existing_secondary_vertex.parent_index
                            {
                                // same cell expansion. due to the consistent nature of the heuristic, a new
                                // vertex that is closer to the goal will have a higher cost-so-far than its
                                // parent, even if in the same cell. so this is the workaround from
                                // Karl Kurzer's masters thesis.
                                //
                                // extending that idea since we are using indexes in the indexmap for
                                // performance, which makes things more complicated. replacing any value in
                                // the indexmap actually invalidates the index that any smallest cost
                                // holders may be referring to. in the case of a child overwriting its
                                // parent, it actually leads to a cycle, which makes reversing the path an
                                // infinite loop!
                                //
                                // the secondary workaround then is to have a .. secondary value in each
                                // cell. thus we allow same-cell expansion only 1 time. expanding the child
                                // a second time and getting to the same grid cell will result in ignoring
                                // the grandchild. we could generalize this to n expansions and store an
                                // array instead, but it doesn't seem necessary.

                                // if we're expanding in the same cell as the parent, but. the
                                // parent was already a secondary, we want to ignore as there's no
                                // spot to put ourselves in
                                if e.index() == new_vertex.parent_index
                                    && new_vertex.parent_is_secondary
                                {
                                    continue;
                                }

                                // turns out that the index invalidation is a problem in general,
                                // since we end up with some nonsensical plan jumps otherwise,
                                // though rarely.  instead, let's limit this extra logic to
                                // same-cell expansion.  however, this prunes paths that may be the
                                // path we want
                                if e.index() != new_vertex.parent_index {
                                    continue;
                                }

                                // now check if we are better than the secondary using the
                                // whole estimate, given this scenario is mostly made for children
                                // of the same parent, and thus pretty similar if not exactly the
                                // same cost so far. we don't want a tie-breaker like Karl's
                                // version had since we are not comparing against a parent
                                // directly, but a sibling!
                                new_estimated_cost = new_cost_so_far + heuristic_cost;

                                set_heuristic_costs(model, &[existing_secondary_vertex.player], &mut single_heuristic_cost, &ball_trajectory, existing_secondary_vertex.ball_trajectory_index);
                                let existing_secondary_estimated_cost = existing_secondary_vertex
                                    .cost_so_far
                                    + single_heuristic_cost[0];

                                if new_estimated_cost < existing_secondary_estimated_cost {
                                    new_cost_is_lower = true;
                                }
                            }

                            if new_cost_is_lower {
                                // TODO-perf avoid the clone here. can use mem::replace with an enum
                                insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                            } else {
                                //visualization_points.push((
                                //    Point3::new(line_end.x, line_end.y, line_end.z),
                                //    Point3::new(0.4, 0.0, 0.0),
                                //));
                                visualization_lines.push((
                                    Point3::new(line_start.x, line_start.y, line_start.z),
                                    Point3::new(
                                        0.1 * (line_end.x - line_start.x) + line_start.x,
                                        0.1 * (line_end.y - line_start.y) + line_start.y,
                                        0.1 * (line_end.z - line_start.z) + line_start.z,
                                    ),
                                    Point3::new(0.4, 0.0, 0.0),
                                ));
                                continue;
                            }
                        }
                    }

                    if let Some(v) = insertable {
                        e.insert(v);
                    }
                }
            }

            //visualization_points.push((
            //    Point3::new(line_end.x, line_end.y, line_end.z),
            //    Point3::new(0.6, 0.6, 0.6),
            //));
            visualization_lines.push((
                Point3::new(line_start.x, line_start.y, line_start.z),
                Point3::new(
                    0.1 * (line_end.x - line_start.x) + line_start.x,
                    0.1 * (line_end.y - line_start.y) + line_start.y,
                    0.1 * (line_end.z - line_start.z) + line_start.z,
                ),
                Point3::new(0.6, 0.6, 0.6),
            ));

            // rustc is forcing us to initialize this every time because it doesn't understand data
            // flow, so let's manually make sure we aren't in the initial state which is never right
            assert!(new_estimated_cost != 0.0);

            // NOTE this insertion into to_see is a replacement for the open set decreaseKey
            // operation. there is also a check for multiple insertions earlier.
            to_see.push(SmallestCostHolder {
                estimated_cost: new_estimated_cost,
                cost_so_far: new_cost_so_far,
                index: new_index,
                is_secondary: new_is_secondary,
            });
        }
    }

    // println!(
    //     "omg failed! step size: {} | expansions: {} | left: {}",
    //     config.step_duration * 120.0,
    //     visualization_lines.len(),
    //     to_see.len()
    // );

    PlanResult {
        visualization_lines,
        visualization_points,
        ..PlanResult::default()
    }
}

fn set_heuristic_costs<H: HeuristicModel>(model: &mut H, new_players: &[PlayerState], costs: &mut Vec<f32>, ball_trajectory: &[BallState], ball_trajectory_index: usize) {
    while costs.len() < new_players.len() {
        costs.push(0.0)
    }

    let goal = Vector3::new(0.0, BACK_WALL_DISTANCE, BALL_COLLISION_RADIUS); // FIXME don't hard-code
    if let Some(ball) = ball_trajectory.get(ball_trajectory_index) {
        model.ball_configure(&ball, &goal); // FIXME adjust this for ball velocity
    } else {
        // just make it a high cost as the ball doesn't exist in this offset as far as we know
        for cost in costs.iter_mut() { *cost = 1000.0 }
        return
    }

    model
        .heuristic(&new_players, &mut costs[0..new_players.len()])
        .expect("Heuristic failed!");
}

fn coarse_collision(candidate_vertex: &PlayerVertex, previous_player: &PlayerState, ball: &BallState) -> bool {
    // the bounding box size includes the car dimensions because we use the center of the car's
    // position to create the line for the coarse collision check
    let size = BALL_COLLISION_RADIUS + CAR_DIMENSIONS.norm() / 2.0;

    let coarse_box = BoundingBox::new(&ball.position, size);
    // XXX note if using large time steps, this will be especially  inaccurate as we assume prev to
    // current is a straight line but it may be curved. this is offset by the fact that we use the
    // maximium car dimension to extend the bounding box though, so we may have fewer false
    // negatives than otherwise
    line_collides_bounding_box(&coarse_box, previous_player.hitbox_center(), candidate_vertex.player.hitbox_center())
}

fn player_goal_reached(
    candidate_vertex: &PlayerVertex,
    previous_player: &PlayerState,
    ball_trajectory: &[BallState],
    controller: &BrickControllerState,
    time_step: f32,
    evaluator: fn(&[BallState], &PlayerState, &PlayerVertex, &BrickControllerState, f32) -> Option<(PlayerState, BallState, f32)>, // consider using Fn trait + generics to make this inlinable
) -> Option<(PlayerState, BallState, f32)> {
    let coarse_collision = coarse_collision(candidate_vertex, previous_player, &ball_trajectory[candidate_vertex.ball_trajectory_index]);
    if !coarse_collision {
        return None;
    };

    evaluator(ball_trajectory, previous_player, candidate_vertex, controller, time_step)
}

fn reverse_path(parents: &ParentsMap, initial_index: usize, initial_is_secondary: bool, initial_player: &PlayerState, initial_cost: f32) -> Plan {
    let path = itertools::unfold((initial_index, initial_is_secondary), |vals| {
        let index = (*vals).0;
        let is_secondary = (*vals).1;
        parents.get_index(index).map(|(_rounded, (v1, maybe_v2))| {
            let vertex = if is_secondary {
                maybe_v2.as_ref().unwrap()
            } else {
                v1
            };
            (*vals).0 = vertex.parent_index;
            (*vals).1 = vertex.parent_is_secondary;
            let player = if index == initial_index { initial_player.clone() } else { vertex.player };
            let cost = if index == initial_index { initial_cost } else { vertex.step_duration };
            (player, vertex.prev_controller, cost)
        })
    })
    .collect::<Vec<_>>();

    path.into_iter().rev().collect()
}

/// not the opponent's goal. this is the goal for our a* search!
#[derive(Copy, Clone, Debug)]
pub struct Goal {
    bounding_box: BoundingBox,
    heading: Unit<Vector3<f32>>,
    min_dot: f32, // desired precision in heading
}

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl BoundingBox {
    fn new(pos: &Vector3<f32>, slop: f32) -> BoundingBox {
        let mut bb = BoundingBox {
            min_x: pos.x - slop,
            max_x: pos.x + slop,
            min_y: pos.y - slop,
            max_y: pos.y + slop,
            min_z: pos.z - slop,
            max_z: pos.z + slop,
        };
        // FIXME hack to extend bounds down to the ground, where the car can reach them
        if bb.max_z - bb.min_z < BALL_COLLISION_RADIUS * 2.0 {
            bb.min_z = (bb.min_z + bb.max_z) / 2.0 - BALL_COLLISION_RADIUS
        }
        bb
    }
}

// https://bheisler.github.io/post/writing-gpu-accelerated-path-tracer-part-3/
// https://gamedev.stackexchange.com/a/18459/4929
pub fn ray_collides_bounding_box(
    bounding_box: &BoundingBox,
    start: Vector3<f32>,
    end: Vector3<f32>,
) -> bool {
    let dir = end - start;
    let dir_inv = Vector3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);

    let mut txmin = (bounding_box.min_x - start.x) * dir_inv.x;
    let mut txmax = (bounding_box.max_x - start.x) * dir_inv.x;

    if txmin > txmax {
        mem::swap(&mut txmin, &mut txmax);
    }

    let mut tymin = (bounding_box.min_y - start.y) * dir_inv.y;
    let mut tymax = (bounding_box.max_y - start.y) * dir_inv.y;

    if tymin > tymax {
        mem::swap(&mut tymin, &mut tymax);
    }

    if txmin > tymax || tymin > txmax {
        return false;
    }

    let mut tzmin = (bounding_box.min_z - start.z) * dir_inv.z;
    let mut tzmax = (bounding_box.max_z - start.z) * dir_inv.z;

    if tzmin > tzmax {
        mem::swap(&mut tzmin, &mut tzmax);
    }

    let tmin = txmin.max(tymin).max(tzmin);
    let tmax = txmax.min(tymax).min(tzmax);

    // AABB is behind!
    if tmax < 0.0 {
        return false;
    }

    // no intersection
    if tmin > tmax {
        return false;
    }

    true
}

//// since the ray collision algorithm doesn't allow for the ray ending early, but does account
//// for point of origin, we just test it in both directions for a complete line test. minimize
//// the overhead by using the previous position as the ray origin first, assuming we'll
//// mostly be moving *towards* the desired position for most expanded a* paths
pub fn line_collides_bounding_box(
    bounding_box: &BoundingBox,
    start: Vector3<f32>,
    end: Vector3<f32>,
) -> bool {
    ray_collides_bounding_box(&bounding_box, start, end)
        && ray_collides_bounding_box(&bounding_box, end, start)
}

fn round_player_state(player: &PlayerState, step_duration: f32, speed: f32) -> RoundedPlayerState {
    // we're using the rounded speed to determine the grid size. we want a good bit of tolerance for
    // this, if we relax the rounded velocity equality check. or some other logic that will ensure
    // same grid for different player states that we want to match
    let rounding_factor = 1.0; // TODO tune. for both correctness AND speed!
    let mut rounded_speed = (speed / rounding_factor).round();
    if rounded_speed == 0.0 {
        rounded_speed = 0.5;
    }
    let rounded_speed = rounded_speed * rounding_factor;

    let grid_size = step_duration * rounded_speed;
    //let velocity_margin = 250.0; // TODO tune
    let (_roll, _pitch, yaw) = player.rotation.euler_angles();

    RoundedPlayerState {
        // TODO we could have individual grid sizes for x/y/z based on vx/vy/vz. not sure it's
        // worth it.
        x: (grid_size * (player.position.x / grid_size).round()) as i16,
        y: (grid_size * (player.position.y / grid_size).round()) as i16,
        z: 0i16, // FIXME // (grid_size * (player.position.z / grid_size).round()) as i16,

        //   // XXX including velocity in the search space might just be too much. but let's give it
        //   // a shot sometime later.
        //   vx: 0, // TODO // (player.velocity.x / velocity_margin).floor() as i16,
        //   vy: 0, // TODO // (player.velocity.y / velocity_margin).floor() as i16,
        //   vz: 0, // TODO // (player.velocity.z / velocity_margin).floor() as i16,

        //   // XXX is this the best way to round a rotation matrix? do discontinuities in euler angles
        //   // cause problems here?
        //   // TODO use angular velocity to determine margin of rounding.
        //   // XXX including rotation in search space also seems like too much for now
        //   roll: 0, //(roll * 10.0).floor() as i16,
        //   pitch: 0, //(pitch * 10.0).floor() as i16,
        yaw: (yaw / (PI / 8.0)).round() as i16, // round to nearest pi/4 angle
    }
}

fn control_branches(player: &PlayerState) -> &'static Vec<BrickControllerState> {
    match predict::player::find_prediction_category(&player) {
        // TODO if we ran out of boost, then need a boost-less version
        predict::player::PredictionCategory::Ground => &GROUND_CONTROL_BRANCHES
        //PredictionCategory::Ground2 => TODO,
        //PredictionCategory::Wall => TODO,
        //PredictionCategory::Ceiling => TODO,
        //PredictionCategory::CurveWall => TODO,
        //PredictionCategory::Air => TODO,
    }
}

// the margin is to allow hitting on the curve or from inside the goal. this is only needed until
// we get prediction for driving on the curves/walls
// TODO well we should model the goal area as drivable too
const BOUNDS_MARGIN: f32 = 200.0;
fn out_of_bounds(player: &PlayerState) -> bool {
    let pos = player.position;
    pos.x.abs() > SIDE_CURVE_DISTANCE + BOUNDS_MARGIN
        || pos.y.abs() > BACK_CURVE_DISTANCE + BOUNDS_MARGIN
}

fn expand_vertex(
    index: usize,
    is_secondary: bool,
    vertex: &PlayerVertex,
    new_vertices: &mut Vec<PlayerVertex>,
    step_duration: f32,
    custom_filter: Option<fn(&PlayerState) -> bool>,
) {
    let iterator = control_branches(&vertex.player)
        .iter()
        .map(|&controller: &BrickControllerState| -> Result<PlayerVertex, String> {
            let next_player = predict::player::next_player_state(&vertex.player, &controller, step_duration);
            if next_player.is_err() {
                // print to stderr now since we're swallowing these errors right after this
                eprintln!("Warning: failed to expand vertex: {}", next_player.as_ref().unwrap_err());
            }

            Ok(PlayerVertex {
                player: next_player?,
                cost_so_far: vertex.cost_so_far + step_duration,
                prev_controller: controller,
                ball_trajectory_index: vertex.ball_trajectory_index,
                step_duration: step_duration,
                parent_index: index,
                parent_is_secondary: is_secondary,
            })
        })
        .filter_map(Result::ok)
        .filter(|new_vertex| {
            if let Some(filter_func) = custom_filter {
                filter_func(&new_vertex.player)
            } else {
                // if parent (ie vertex) is already out of bounds, allow going out of bounds since we need
                // to be able to move back in if we start planning from out of bounds (eg player inside
                // goal). we're  only allowing if we're getting *less* out of bounds than before
                !out_of_bounds(&new_vertex.player) || out_of_bounds(&vertex.player)
            }
        });
    new_vertices.extend(iterator);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const FINE_STEP: f32 = 8.0 * TICK;
    const MEDIUM_STEP: f32 = 16.0 * TICK;
    const COARSE_STEP: f32 = 16.0 * TICK;
    const VERY_COARSE_STEP: f32 = 16.0 * TICK;

    fn resting_position() -> Vector3<f32> {
        Vector3::new(0.0, 0.0, 0.0)
    }
    fn resting_velocity() -> Vector3<f32> {
        Vector3::new(0.0, 0.0, 0.0)
    }
    fn resting_rotation() -> UnitQuaternion<f32> {
        UnitQuaternion::from_euler_angles(0.0, 0.0, -PI / 2.0)
    }

    fn resting_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            angular_velocity: resting_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    fn test_ball() -> BallState {
        BallState::default()
    }

    fn test_desired_contact() -> DesiredContact {
        DesiredContact {
            position: resting_position(),
            heading: Vector3::new(0.0, 1.0, 0.0),
        }
    }

    #[test]
    fn line_doesnt_collide_even_though_ray_does() {
        let slop = 1.0;
        let bounding_box = BoundingBox {
            min_x: 0.0 - slop,
            max_x: 0.0 + slop,
            min_y: 0.0 - slop,
            max_y: 0.0 + slop,
            min_z: 0.0 - slop,
            max_z: 0.0 + slop,
        };
        let start = Vector3::new(2.0, 0.0, 0.0);
        let end = Vector3::new(3.0, 0.0, 0.0);
        assert!(!ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, 2.0, 0.0);
        let end = Vector3::new(0.0, 3.0, 0.0);
        assert!(!ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, 0.0, 2.0);
        let end = Vector3::new(0.0, 0.0, 3.0);
        assert!(!ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(-2.0, 0.0, 0.0);
        let end = Vector3::new(-3.0, 0.0, 0.0);
        assert!(!ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, -2.0, 0.0);
        let end = Vector3::new(0.0, -3.0, 0.0);
        assert!(!ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, 0.0, -2.0);
        let end = Vector3::new(0.0, 0.0, -3.0);
        assert!(!ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));
    }

    #[test]
    fn line_collides() {
        let slop = 1.0;
        let bounding_box = BoundingBox {
            min_x: 0.0 - slop,
            max_x: 0.0 + slop,
            min_y: 0.0 - slop,
            max_y: 0.0 + slop,
            min_z: 0.0 - slop,
            max_z: 0.0 + slop,
        };
        let start = Vector3::new(0.5, 0.0, 0.0);
        let end = Vector3::new(1.5, 0.0, 0.0);
        assert!(ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, 0.5, 0.0);
        let end = Vector3::new(0.0, 1.5, 0.0);
        assert!(ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, 0.0, 0.5);
        let end = Vector3::new(0.0, 0.0, 1.5);
        assert!(ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(-0.5, 0.0, 0.0);
        let end = Vector3::new(-1.5, 0.0, 0.0);
        assert!(ray_collides_bounding_box(&bounding_box, start, end));
        assert!(ray_collides_bounding_box(&bounding_box, end, start));

        let start = Vector3::new(0.0, -0.5, 0.0);
        let end = Vector3::new(0.0, -1.5, 0.0);
        assert!(ray_collides_bounding_box(&bounding_box, end, start));
        assert!(ray_collides_bounding_box(&bounding_box, start, end));

        let start = Vector3::new(0.0, 0.0, -0.5);
        let end = Vector3::new(0.0, 0.0, -1.5);
        assert!(ray_collides_bounding_box(&bounding_box, end, start));
        assert!(ray_collides_bounding_box(&bounding_box, start, end));
    }

    #[test]
    fn just_drive_straight() {
        let mut count = 0;
        let mut failures = vec![];
        let mut current = resting_player_state();
        current.position.y = -1000.0;
        let desired = test_desired_contact();
        let PlanResult { mut plan, .. } = hybrid_a_star(&current, test_ball(), &desired, 0.5);
        //assert!(plan.is_some());
        if plan.is_some() {
            count += 1
        } else {
            failures.push(true)
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!(
            "FAIL PERCENT {}%",
            100.0 * failures.len() as f32 / count as f32
        );
        println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_fuzz1() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -500..0 {
            for &step_duration in [0.5].iter() {
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = test_desired_contact();
                let PlanResult { mut plan, .. } = hybrid_a_star(&current, test_ball(), &desired, step_duration);
                //assert!(plan.is_some());
                if plan.is_some() {
                    count += 1
                } else {
                    failures.push((step_duration, distance))
                }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!(
            "FAIL PERCENT {}%",
            100.0 * failures.len() as f32 / count as f32
        );
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    // #[test]
    // fn just_drive_straight_fuzz2() {
    //     let mut count = 0;
    //     let mut failures = vec![];
    //     for distance in -1000..-500 {
    //         for &step_duration in [0.5].iter() {
    //             let mut current = resting_player_state();
    //             current.position.y = distance as f32;
    //             let desired = test_desired_contact();
    //             let PlanResult { mut plan, .. } = hybrid_a_star(&current, test_ball(), &desired, step_duration);
    //             //assert!(plan.is_some());
    //             if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
    //         }
    //     }
    //     println!("WORKED {} TIMES", count);
    //     println!("FAILED {} TIMES", failures.len());
    //     println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
    //     //println!("FAILURES: {:?}", failures);
    //     assert!(failures.len() == 0);
    // }

    // #[test]
    // fn just_drive_straight_fuzz3() {
    //     let mut count = 0;
    //     let mut failures = vec![];
    //     //for distance in -2000..-1000 {
    //     for distance in -2000..-1900 {
    //         for &step_duration in [0.5].iter() {
    //         //for &step_duration in [COARSE_STEP].iter() {
    //             let mut current = resting_player_state();
    //             current.position.y = distance as f32;
    //             let desired = test_desired_contact();
    //             let PlanResult { mut plan, .. } = hybrid_a_star(&current, test_ball(), &desired, step_duration);
    //             //assert!(plan.is_some());
    //             if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
    //         }
    //     }
    //     println!("WORKED {} TIMES", count);
    //     println!("FAILED {} TIMES", failures.len());
    //     println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
    //     //println!("FAILURES: {:?}", failures);
    //     assert!(failures.len() == 0);
    // }

    // #[test]
    // fn just_drive_straight_fuzz4() {
    //     let mut count = 0;
    //     let mut failures = vec![];
    //     for distance in -4000..-2000 {
    //         for &step_duration in [0.5].iter() {
    //             let mut current = resting_player_state();
    //             current.position.y = distance as f32;
    //             let desired = test_desired_contact();
    //             let PlanResult { mut plan, .. } = hybrid_a_star(&current, test_ball(), &desired, step_duration);
    //             //assert!(plan.is_some());
    //             if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
    //         }
    //     }
    //     println!("WORKED {} TIMES", count);
    //     println!("FAILED {} TIMES", failures.len());
    //     println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
    //     //println!("FAILURES: {:?}", failures);
    //     assert!(failures.len() == 0);
    // }

    #[test]
    fn unreachable() {
        let mut count = 0;
        let distance = -10_000;
        for &step_duration in [FINE_STEP, MEDIUM_STEP, COARSE_STEP, VERY_COARSE_STEP].iter() {
            let mut current = resting_player_state();
            current.position.y = distance as f32;
            let desired = test_desired_contact();
            let PlanResult { mut plan, .. } = hybrid_a_star(&current, test_ball(), &desired, step_duration);
            assert!(plan.is_none());
        }
    }
}
