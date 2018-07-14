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
}

struct SmallestCostHolder {
    estimated_cost: f32,
    cost_so_far: f32,
    index: usize,
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
    let mut parents: IndexMap<RoundedPlayerState, PlayerVertex, MyHasher> = IndexMap::default();
    let mut visualization_lines = vec![];

    to_see.push(SmallestCostHolder {
        estimated_cost: heuristic_cost(&current, &desired),
        cost_so_far: 0.0,
        index: 0,
    });

    let start = PlayerVertex {
        player: current.clone(),
        cost_so_far: 0.0,
        prev_controller: BrickControllerState::new(),
        parent_index: usize::MAX,
    };

    parents.insert(round_player_state(&current, step_duration, true), start);

    let mut i = 0.0f32;
    while let Some(SmallestCostHolder { estimated_cost, cost_so_far, index, .. }) = to_see.pop() {
        println!("PARENTS\n====================\n {:?}", parents);
        // hack to avoid a infinite graph search
        if cost_so_far > 5.0 {
            break;
        }

        // FIXME super hack for debugging
        if visualization_lines.len() > 20 {
            break;
        }


        let line_start;
        let new_vertices = {
            let (_rounded, vertex) = parents.get_index(index).expect("missing index in parents, shouldn't be possible");
            line_start = vertex.player.position;

            if player_goal_reached(&vertex.player, &desired, step_duration) {
                println!("omg reached {}", visualization_lines.len());
                return ( Some(reverse_path(&parents, index)), visualization_lines );
            }

            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            //
            // NOTE: this also achieves the same thing as checking if we are in the closed set
            if cost_so_far > vertex.cost_so_far {
                continue;
            }

            expand_vertex(index, &vertex, step_duration)
        };

        for new_vertex in new_vertices {
            let new_vertex_rounded = round_player_state(&new_vertex.player, step_duration, true);
            let new_cost_so_far = new_vertex.cost_so_far;
            let new_index;
            let line_end = new_vertex.player.position;
            let mut new_estimated_cost = 0.0;

            println!("new_vertex.position: {:?}\nnew_cost_s_far: {}\nheuristic_cost: {}\n\n", new_vertex.player.position, new_cost_so_far, heuristic_cost(&new_vertex.player, &desired));
            match parents.entry(new_vertex_rounded) {
                Vacant(e) => {
                    new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                    new_index = e.index();
                    e.insert(new_vertex);

                }
                Occupied(mut e) => {
                    let mut new_cost_is_lower = e.get().cost_so_far > new_vertex.cost_so_far;

                    // same cell expansion. due to the consistent nature of the heuristic, a new
                    // vertex that is closer to the goal will have a higher cost-so-far than its
                    // parent, even if in the same cell. so this is the workaround from
                    // Karl Kurzer's masters thesis.
                    if new_cost_is_lower {
                        new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                    } else if e.index() == new_vertex.parent_index {
                        new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);

                        // NOTE `estimated_cost` is related to the parent vertex, ie e.get(), but we
                        // have it already from smallest cost holder so we avoid re-calculating it
                        if new_estimated_cost < estimated_cost + step_duration {
                            new_cost_is_lower = true;
                        }
                    }

                    if new_cost_is_lower {
                        new_index = e.index();
                        e.insert(new_vertex); // XXX this doesn't seem right. the parent is replaced, but can't other nodes point at the parent??
                    } else {
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
            });
        }
    }
    println!("omg failed {}", visualization_lines.len());

    (None, visualization_lines)
}

/// the step duration defines the margin of error we allow. larger steps means we allow a larger
/// margin. eg with a 1-second step, it's unlikely we can ever find that the goal has been reached
/// unless we allow a margin that is quite big. note that the velocity and angular velocity will
/// also figure into the margin for position and rotation.
fn player_goal_reached(candidate: &PlayerState, desired: &PlayerState, step_duration: f32) -> bool {
    println!("is player goal reached\ncandidate: {:?}\ndesired: {:?}", candidate, desired);
    let candidate = round_player_state(&candidate, step_duration, false);
    let desired = round_player_state(&desired, step_duration, false); // TODO we could memoize this one. or avoid it by enforcing rounding at the beginning
    println!("is player goal reached\nrounded candidate: {:?}\nrounded desired: {:?}", candidate, desired);

    candidate == desired
}

fn reverse_path(parents: &IndexMap<RoundedPlayerState, PlayerVertex, MyHasher>, start: usize) -> Vec<(PlayerState, BrickControllerState)> {
    let path = itertools::unfold(start, |i| {
        parents.get_index(*i).map(|(_rounded, vertex)| {
            *i = vertex.parent_index;
            //println!("xxxxxxxxxxxxxxxxxx: {} | {:?}", i, vertex.player.position);
            (vertex.player, vertex.prev_controller)
        })
    }).collect::<Vec<_>>();

    path.into_iter().rev().collect()
}

fn round_player_state(player: &PlayerState, step_duration: f32, pruning: bool) -> RoundedPlayerState {
    // we're using the rounded speed to determine the grid size. we want a good bit of tolerance for
    // this, if we relax the rounded velocity equality check. or some other logic that will ensure
    // same grid for different player states that we want to match
    //
    // NOTE actually this may make sense for the goal state check. but for pruning, we want
    // to round more aggressively. that is, if we are going really slow, we should still have a min
    // grid size so that we prune similar states as aggressively as possible
    //
    // FIXME the discontinuity in grid size going from a rounded speed of 400 to 800 is not
    // a problem since these are divisible. but 800 to 1200 is an issue. So we should instead
    // double the grid size, to at least help with the case where velocities are around 950 to
    // 1050, resulting in completely non-overlapping grid cells. with doubling, we will at least
    // overlap in half of the boundaries, and thus prune more branches in this case
    //
    let mut rounded_speed = (player.velocity.norm() / 400.0).round(); // TODO tune
    if rounded_speed == 0.0 { rounded_speed = 0.5 }
    let rounded_speed = rounded_speed * 400.0;

    let mut grid_size = step_duration * rounded_speed;
    let velocity_margin = 250.0; // TODO tune
    let (roll, pitch, yaw) = player.rotation.to_euler_angles();

    // TODO tune min grid size
    if pruning && grid_size < 50.0 {
        grid_size = 50.0;
    }

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

fn expand_vertex(index: usize, vertex: &PlayerVertex, step_duration: f32) -> Vec<PlayerVertex> {
    control_branches(&vertex.player).iter().map(|&controller| {
        let x = PlayerVertex {
            player: predict::player::next_player_state(&vertex.player, &controller, step_duration),
            // TODO incorporate small boost usage penalty
            cost_so_far: vertex.cost_so_far + step_duration,
            prev_controller: controller,
            parent_index: index,
        };
        println!("control branch: {:?}", x);
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

    // we can't add these times as they may be overlapping (eg imagine we just need to go straight
    // from resting), and we must have an admissable heuristic. so it's just whichever one is
    // bigger.
    if movement_time_cost > acceleration_time_cost {
        movement_time_cost
    } else {
        acceleration_time_cost
    }
}
