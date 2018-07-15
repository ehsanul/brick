use state::*;
use predict;
use na::{ Vector3, Point3, UnitQuaternion };
use std::cmp::Ordering;

use std::collections::BinaryHeap;
use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;
use std::usize;
use itertools;

use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
type MyHasher = BuildHasherDefault<FnvHasher>;



lazy_static! {
    pub static ref GROUND_CONTROL_BRANCHES: Vec<BrickControllerState> = {
        let mut left_boost = BrickControllerState::new();
        left_boost.boost = true;
        left_boost.steer = Steer::Left;

        let mut right_boost = BrickControllerState::new();
        right_boost.boost = true;
        right_boost.steer = Steer::Right;

        let mut straight_boost = BrickControllerState::new();
        straight_boost.boost = true;

        let mut left_throttle = BrickControllerState::new();
        left_throttle.throttle = Throttle::Forward;
        left_throttle.steer = Steer::Left;

        let mut right_throttle = BrickControllerState::new();
        right_throttle.throttle = Throttle::Forward;
        right_throttle.steer = Steer::Right;

        let mut straight_throttle = BrickControllerState::new();
        straight_throttle.throttle = Throttle::Forward;

        let mut left_idle = BrickControllerState::new();
        left_idle.steer = Steer::Left;

        let mut right_idle = BrickControllerState::new();
        right_idle.steer = Steer::Right;

        let mut straight_idle = BrickControllerState::new();

        let mut left_brake = BrickControllerState::new();
        left_brake.throttle = Throttle::Reverse;
        left_brake.steer = Steer::Left;

        let mut right_brake = BrickControllerState::new();
        right_brake.throttle = Throttle::Reverse;
        right_brake.steer = Steer::Right;

        let mut straight_brake = BrickControllerState::new();
        straight_brake.throttle = Throttle::Reverse;

        vec![
            //left_boost,
            //right_boost,
            //straight_boost,

            //left_throttle,
            //right_throttle,
            straight_throttle,

            //left_idle,
            //right_idle,
            //straight_idle,

            //left_brake,
            //right_brake,
            //straight_brake,
        ]
    };
}


/// given the current state and a desired state, return one frame of input that will take us to the
/// desired state, if any is possible.
// TODO maybe we should take the entire gamestate instead. we also need a history component
#[no_mangle]
pub extern fn plan(player: &PlayerState, ball: &BallState, desired_state: &DesiredState) -> Option<BrickControllerState> {
    // TODO figure out the right function to call rather than expect/unwrap
    //let ref desired_player: &PlayerState = &desired.player.expect("desired player is required for now");
    if let Some(ref desired_player) = desired_state.player {
        println!("{}", desired_player.position);
    } else {
        panic!("desired player is required for now");
    }

    // this might be supported by translating into a set of possible desired player state at
    // possible impact points with the ball..
    if desired_state.ball.is_some() { panic!("desired ball not supported for now"); }

    let mut controller = BrickControllerState::new();

    let step_duration = 1.0; // FIXME how to set this?
    let (path, visualization_lines) = hybrid_a_star(player, &desired_state.player.unwrap(), step_duration);
    match path {
        Some(p) => Some(p.first().unwrap().1),
        None => None,
    }
}

#[derive(Clone, Debug)]
struct PlayerVertex {
    cost_so_far: f32,
    player: PlayerState,
    /// the controller state in previous step that lead to this player vertex
    prev_controller: BrickControllerState,
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
    vx: i16,
    vy: i16,
    vz: i16,
    roll: i16,
    pitch: i16,
    yaw: i16,
}


#[no_mangle]
pub extern fn hybrid_a_star(current: &PlayerState, desired: &PlayerState, step_duration: f32) -> (Option<Vec<(PlayerState, BrickControllerState)>>, Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>) {
    let mut to_see: BinaryHeap<SmallestCostHolder> = BinaryHeap::new();
    let mut parents: IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher> = IndexMap::default();
    let mut visualization_lines = vec![];

    to_see.push(SmallestCostHolder {
        estimated_cost: heuristic_cost(&current, &desired),
        cost_so_far: 0.0,
        index: 0,
        is_secondary: false,
    });

    let start = PlayerVertex {
        player: current.clone(),
        cost_so_far: 0.0,
        prev_controller: BrickControllerState::new(),
        parent_index: usize::MAX,
        parent_is_secondary: false,
    };

    parents.insert(round_player_state(&current, step_duration), (start, None));

    let mut i = 0.0f32;
    while let Some(SmallestCostHolder { estimated_cost, cost_so_far, index, is_secondary, .. }) = to_see.pop() {
        // DEBUG // println!("estimated_cost: {}, cost_so_far: {}, index: {}, is_secondary: {}", estimated_cost, cost_so_far, index, is_secondary);
        // DEBUG // println!("PARENTS\n====================");
        // DEBUG // for (k, v) in parents.iter() {
        // DEBUG //     println!("{:?}\n   -   ->{:?}\n------------", k, v);
        // DEBUG // }
        // DEBUG // println!("to_see\n===========================");
        // DEBUG // for x in to_see.iter() {
        // DEBUG //     println!("---> {:?}", x);
        // DEBUG // }
        // DEBUG // println!("===========================");

        // hack to avoid a infinite graph search
        if cost_so_far > 5.0 {
            break;
        }

        // FIXME super hack for debugging
        //if visualization_lines.len() > 20 {
        //    break;
        //}


        let line_start;
        let new_vertices = {
            let (_rounded, (v1, maybe_v2)) = parents.get_index(index).expect("missing index in parents, shouldn't be possible");
            let vertex = if is_secondary { maybe_v2.as_ref().unwrap() }  else { v1 };
            line_start = vertex.player.position;

            if player_goal_reached(&vertex.player, &desired, step_duration) {
                // DEBUG // println!("");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("");
                // DEBUG // println!("omg reached {}", visualization_lines.len());
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||");
                // DEBUG // println!("");
                return ( Some(reverse_path(&parents, index, is_secondary)), visualization_lines );
            }

            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            //
            // NOTE: this also achieves the same thing as checking if we are in the closed set
            if cost_so_far > vertex.cost_so_far {
                continue;
            }

            expand_vertex(index, is_secondary, &vertex, step_duration)
        };

        for new_vertex in new_vertices {
            // DEBUG // println!(" - - - - - - - - - - - - - - - - -");
            let new_vertex_rounded = round_player_state(&new_vertex.player, step_duration);
            let new_cost_so_far = new_vertex.cost_so_far;
            let new_index;
            let mut new_is_secondary = false;
            let line_end = new_vertex.player.position;
            let mut new_estimated_cost = 0.0;

            // DEBUG // println!("new_vertex.position: {:?}\nnew_cost_s_far: {}\nheuristic_cost: {}", new_vertex.player.position, new_cost_so_far, heuristic_cost(&new_vertex.player, &desired));
            // DEBUG // println!("new_vertex_rounded: {:?}", new_vertex_rounded);
            match parents.entry(new_vertex_rounded) {
                Vacant(e) => {
                    // DEBUG // println!("VACANT");
                    new_index = e.index();
                    new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                    e.insert((new_vertex, None));

                }
                Occupied(mut e) => {
                    new_index = e.index();
                    new_is_secondary = true;
                    let mut insertable = None; // make borrowck happy
                    match e.get() {
                        (existing_vertex, None) => {
                            // DEBUG // println!("KINDA OCCUPIED");
                            // basically just like the vacant case
                            new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                            insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                        },
                        (existing_vertex, Some(existing_secondary_vertex)) => {
                            // DEBUG // println!("REALLY OCCUPIED");
                            let mut new_cost_is_lower = existing_secondary_vertex.cost_so_far > new_vertex.cost_so_far;
                            if new_cost_is_lower {
                                new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                            } else if e.index() == new_vertex.parent_index {
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

                                // as at this point, we've determined that our parent is from here. if the
                                // parent was a secondary though, we want to ignore as there's no spot to
                                // put ourselves in
                                if new_vertex.parent_is_secondary {
                                    // DEBUG // println!("parent is secondary, so i can't insert and leaving it be");
                                    continue;
                                }

                                // now check if we are better than the secondary using the
                                // whole estimate, given this scenario is mostly made for children
                                // of the same parent, and thus pretty similar if not exactly the
                                // same cost so far. we don't want a tie-breaker like Karl's
                                // version had since we are not comparing against a parent
                                // directly, but a sibling!
                                new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                                let existing_secondary_estimated_cost = existing_secondary_vertex.cost_so_far + heuristic_cost(&existing_secondary_vertex.player, &desired);

                                if new_estimated_cost < existing_secondary_estimated_cost {
                                    new_cost_is_lower = true;
                                } else {
                                    // DEBUG // println!("didn't replace existing secondary");
                                    // DEBUG // println!("new - so far: {} | h: {} | est: {}", new_cost_so_far, heuristic_cost(&new_vertex.player, &desired), new_estimated_cost);
                                    // DEBUG // println!("old - so far: {} | h: {} | est: {}", existing_secondary_vertex.cost_so_far, heuristic_cost(&existing_secondary_vertex.player, &desired), estimated_cost);
                                }
                            } else {
                                // DEBUG // println!("well, our cost is larger and we're not doing same-cell expension, rip");
                            }

                            if new_cost_is_lower {
                                // DEBUG // println!("new cost is lower");
                                insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                            } else {
                                // DEBUG // println!("new cost is NOT lower");
                                // show pruned as grey
                                visualization_lines.push((
                                    Point3::new(line_start.x, line_start.y, line_start.z),
                                    Point3::new(line_end.x, line_end.y, line_end.z),
                                    Point3::new(0.3, 0.3, 0.3),
                                ));
                                continue;
                            }
                        }
                    }

                    if let Some(v) = insertable { e.insert(v); }


                }
            }

            // show expanded as colored 
            visualization_lines.push((
                Point3::new(line_start.x, line_start.y, line_start.z),
                Point3::new(line_end.x, line_end.y, line_end.z),
                Point3::new(0.5 + 0.5*i.sin(), 0.5 + 0.5*(i/7.0).sin(), 0.5 + 0.5*(i/23.0).sin()),
            ));
            i += 93.0;

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
    // DEBUG // println!("omg failed {}", visualization_lines.len());

    (None, visualization_lines)
}

/// the step duration defines the margin of error we allow. larger steps means we allow a larger
/// margin. eg with a 1-second step, it's unlikely we can ever find that the goal has been reached
/// unless we allow a margin that is quite big. note that the velocity and angular velocity will
/// also figure into the margin for position and rotation.
//
//
// we're having trouble with this naive implementation when given really large step sizes. the
// problem is that the step sizes can get big enough that we overshoot the goal completely and go
// through ti, but don't register that the state hits the goal.
//
// part of the problem is when the goal is at a corner of a grid cell. that means when a vertex
// that's really close to it, but outside the grid, we miss it. then the next step takes it really
// far away.
//  
// so we want to change this by making the grid be positioned such that the goal is in the midpoint
// of a grid cell.
//
// even if we fix this, we will still end up in the situation where a big enough step could pass
// right over the grid containing the goal. solutions to this include:
//
// 1. tune the grid size so this is not possible.
// 2. use a line intersection between candidate and it's parent node, and if that line touches the
//    grid at any point we can consider that good
// 3. try the midpoint instead of a full line intersection. or several points. same idea though.
//
// however, we want this to be really cheap! and ideally it is not used at all for small step
// durations, we should have good enough control over the grid there to avoid this completely.
//
// XXX: nvm, looks like tuning the rounded_speed used for the grid size made a pretty big impact on
// how often we find a goal state no matter the step duration etc, at least for the really basic
// drive straight case. we'll probably have to revisit this as we add way more ways to move..
fn player_goal_reached(candidate: &PlayerState, desired: &PlayerState, step_duration: f32) -> bool {
    //println!("exact player goal reached\ncandidate: {:?}\ndesired: {:?}", candidate, desired);
    let candidate = round_player_state(&candidate, step_duration);
    let desired = round_player_state(&desired, step_duration); // TODO we could memoize this one. or avoid it by enforcing rounding at the beginning
    //println!("rounded player goal reached\nrounded candidate: {:?}\nrounded desired: {:?}", candidate, desired);

    candidate == desired
}

fn reverse_path(parents: &IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher>, index: usize, is_secondary: bool) -> Vec<(PlayerState, BrickControllerState)> {
    let path = itertools::unfold((index, is_secondary), |i| {
        parents.get_index((*i).0).map(|(_rounded, (v1, maybe_v2))| {
            let vertex = if is_secondary { maybe_v2.as_ref().unwrap() }  else { v1 };
            (*i).0 = vertex.parent_index;
            (*i).1 = vertex.parent_is_secondary;
            //println!("xxxxxxxxxxxxxxxxxx: {} | {:?}", i, vertex.player.position);
            (vertex.player, vertex.prev_controller)
        })
    }).collect::<Vec<_>>();

    path.into_iter().rev().collect()
}

fn round_player_state(player: &PlayerState, step_duration: f32) -> RoundedPlayerState {
    // we're using the rounded speed to determine the grid size. we want a good bit of tolerance for
    // this, if we relax the rounded velocity equality check. or some other logic that will ensure
    // same grid for different player states that we want to match
    let speed = player.velocity.norm();
    let mut rounded_speed = (speed / 10.0).round(); // TODO tune. for both correctness AND speed!
    if rounded_speed == 0.0 {
        rounded_speed = 0.5;
    }
    let rounded_speed = rounded_speed * 10.0;

    let mut grid_size = step_duration * rounded_speed;
    let velocity_margin = 250.0; // TODO tune
    let (roll, pitch, yaw) = player.rotation.to_euler_angles();

    RoundedPlayerState {
        // TODO we could have individual grid sizes for x/y/z based on vx/vy/vz. not sure it's
        // worth it.
        x: (player.position.x / grid_size).floor() as i16,
        y: (player.position.y / grid_size).floor() as i16,
        z: (player.position.z / grid_size).floor() as i16,

        // XXX including velocity in the search space might just be too much. but let's give it
        // a shot sometime later.
        vx: 0, // TODO // (player.velocity.x / velocity_margin).floor() as i16,
        vy: 0, // TODO // (player.velocity.y / velocity_margin).floor() as i16,
        vz: 0, // TODO // (player.velocity.z / velocity_margin).floor() as i16,

        // XXX is this the best way to round a rotation matrix? do discontinuities in euler angles
        // cause problems here?
        // TODO use angular velocity to determine margin of rounding. again with minimum if we are
        // in pruning mode.
        roll: (roll * 10.0).floor() as i16,
        pitch: (pitch * 10.0).floor() as i16,
        yaw: (yaw * 10.0).floor() as i16,
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

fn expand_vertex(index: usize, is_secondary: bool, vertex: &PlayerVertex, step_duration: f32) -> Vec<PlayerVertex> {
    control_branches(&vertex.player).iter().map(|&controller| {
        let x = PlayerVertex {
            player: predict::player::next_player_state(&vertex.player, &controller, step_duration),
            // TODO incorporate small boost usage penalty
            cost_so_far: vertex.cost_so_far + step_duration,
            prev_controller: controller,
            parent_index: index,
            parent_is_secondary: is_secondary,
        };
        //println!("control branch: {:?}", x);
        x
    }).collect::<Vec<PlayerVertex>>()
}

fn heuristic_cost(candidate: &PlayerState, desired: &PlayerState) -> f32 {
    // basic heuristic cost is a lower-bound for how long it would take, given max boost, to reach
    // the desired position and velocity. and we need to do rotation too.
    //
    // NOTE for now we ignore the fact that we are not starting at the max boost velocity pointed
    // directly at the desired position. the heuristic just needs to be a lower bound, until we
    // want to get it more accurate and thus ignore irrelevant branches more efficiently.
    let distance = (desired.position - candidate.position).norm();
    let movement_time_cost = distance / predict::player::MAX_BOOST_SPEED;

    // NOTE this is weird since we are not taking into account that some of the time cost here may
    // overlap with the distance cost. what we really want to do is find the boost vector that
    // would minimize time to reach the target, but seems complicated and we do need this heuristic
    // function to be really cheap.
    //
    // so instead we do something dumb and have basically a second heuristic. we can take the max
    // of the two heuristics
    //
    // FIXME this also assumes boosting is the highest acceleration action. braking or backflipping
    // while at max speed may be higher, making this strictly not admissable
    let relative_velocity = (desired.velocity - candidate.velocity).norm();
    let acceleration_time_cost = relative_velocity / predict::player::BOOST_ACCELERATION_FACTOR;

    // TODO rotation cost
    //let normalization_rotation = candidate.rotation.to_rotation_matrix() * na::inverse(&desired.rotation.to_rotation_matrix());
    //let rotation_cost = normalization_rotation.norm();
    //let rotation_cost = 100.0 * rotation_cost / distance.ln(); // the closer we are, the higher the rotation_cost. TODO make exponential

    // TODO angular velocity cost
    // the distance between orientation R1 and R2 is:
    //
    //     || logm(dot(R1, transpose(R2))) ||
    //
    // where || . || is the Frobenius norm
    // and logm is the matrix logarithm

    //    // we can't add these times as they may be overlapping (eg imagine we just need to go straight
    //    // from resting), and we must have an admissable heuristic. so it's just whichever one is
    //    // bigger.
    //    if movement_time_cost > acceleration_time_cost {
    //        movement_time_cost
    //    } else {
    //        acceleration_time_cost
    //    }
    // FIXME acceleration_time_cost causing problems, removing temporarily
    movement_time_cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn resting_position() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_velocity() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_rotation() -> UnitQuaternion<f32> { UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0) }

    fn resting_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    #[test]
    fn just_drive_straight() {
        let mut count = 0;
        let mut failures = vec![];
        for tick_portion in 1..121 {
            let step_duration = (tick_portion as f32) / predict::FPS;
            let mut current = resting_player_state();
            current.position.y = -1000.0;
            let desired = resting_player_state();
            let (mut path, mut lines) = hybrid_a_star(&current, &desired, step_duration);
            //assert!(path.is_some());
            if path.is_some(){ count += 1 } else { failures.push(tick_portion) }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_more1() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -500..0 {
            for tick_portion in 1..121 {
                let step_duration = (tick_portion as f32) / predict::FPS;
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let (mut path, mut lines) = hybrid_a_star(&current, &desired, step_duration);
                //assert!(path.is_some());
                if path.is_some(){ count += 1 } else { failures.push((tick_portion, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_more2() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -1000..-500 {
            for tick_portion in 1..121 {
                let step_duration = (tick_portion as f32) / predict::FPS;
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let (mut path, mut lines) = hybrid_a_star(&current, &desired, step_duration);
                //assert!(path.is_some());
                if path.is_some(){ count += 1 } else { failures.push((tick_portion, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_more3() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -2000..-1000 {
            for tick_portion in 1..121 {
                let step_duration = (tick_portion as f32) / predict::FPS;
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let (mut path, mut lines) = hybrid_a_star(&current, &desired, step_duration);
                //assert!(path.is_some());
                if path.is_some(){ count += 1 } else { failures.push((tick_portion, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

     #[test]
     fn just_drive_straight_more4() {
         let mut count = 0;
         let mut failures = vec![];
         for distance in -4000..-2000 {
             for tick_portion in 1..121 {
                 let step_duration = (tick_portion as f32) / predict::FPS;
                 let mut current = resting_player_state();
                 current.position.y = distance as f32;
                 let desired = resting_player_state();
                 let (mut path, mut lines) = hybrid_a_star(&current, &desired, step_duration);
                 //assert!(path.is_some());
                 if path.is_some(){ count += 1 } else { failures.push((tick_portion, distance)) }
             }
         }
         println!("WORKED {} TIMES", count);
         println!("FAILURES: {:?}", failures);
         println!("FAILED {} TIMES", failures.len());
         println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
         assert!(failures.len() == 0);
     }

    #[test]
    fn unreachable() {
        let mut count = 0;
        let distance = -10_000;
        for tick_portion in 1..121 {
            let step_duration = (tick_portion as f32) / predict::FPS;
            let mut current = resting_player_state();
            current.position.y = distance as f32;
            let desired = resting_player_state();
            let (mut path, mut lines) = hybrid_a_star(&current, &desired, step_duration);
            assert!(path.is_none());
        }
    }
}
