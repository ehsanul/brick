use state::*;
use predict;
use na::{ self, Unit, Vector3, Point3, UnitQuaternion };
use std::cmp::Ordering;
use std::f32::consts::PI;

use std::collections::BinaryHeap;
use std::mem;
use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;
use std::usize;
use itertools;
use itertools::Itertools;

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

            left_throttle,
            right_throttle,
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

const FINE_STEP: f32 = 4.0 / predict::FPS;
const MEDIUM_STEP: f32 = 10.0 / predict::FPS;
const COARSE_STEP: f32 = 20.0 / predict::FPS;
const VERY_COARSE_STEP: f32 = 40.0 / predict::FPS;
pub(crate) fn appropriate_step(current: &PlayerState, desired: &PlayerState) -> f32 {
    let speed = current.velocity.norm();
    let delta = desired.position - current.position;
    let distance = delta.norm();
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let dot = na::dot(&current_heading, &Unit::new_normalize(delta).unwrap());

    if distance < 500.0 {
        // XXX this was attempted to be tuned per speed, but turns out that with lower speed, we
        // don't go as far along any turning curves, and thus we end up in the same angle, just
        // earlier
        let min_dot = 0.9;

        // check if the desired state within a cone around our heading, whose angle is determined by the speed
        if dot > min_dot {
            FINE_STEP
        } else {
            // we'll probably have to go the long way around as we aren't facing the right way for
            // this, so use a coarse step
            COARSE_STEP
        }
    } else if distance < 1000.0 {

        let min_dot = if speed < 400.0 {
            // XXX in this measurement, it only managed to go 400uu total, so... we probably need
            // to do something smarter here long-term
            0.68
        } else {
            // fastest, so it's the tighest angle
            0.75
        };

        // check if the desired state within a cone around our heading, whose angle is determined by the speed
        if dot > min_dot {
            MEDIUM_STEP
        } else {
            // we'll probably have to go the long way around as we aren't facing the right way for
            // this, so use a coarse step
            COARSE_STEP
        }
    } else {
        COARSE_STEP
    }
}

/// given the current state and a desired state, return one frame of input that will take us to the
/// desired state, if any is possible.
// TODO maybe we should take the entire gamestate instead. we also need a history component
#[no_mangle]
pub extern fn plan(player: &PlayerState, ball: &BallState, desired_state: &DesiredState) -> PlanResult {
    // TODO figure out the right function to call rather than expect/unwrap
    //let ref desired_player: &PlayerState = &desired.player.expect("desired player is required for now");
    if let Some(ref desired_player) = desired_state.player {
        //println!("{}", desired_player.position);
    } else {
        panic!("desired player is required for now");
    }

    // this might be supported by translating into a set of possible desired player state at
    // possible impact points with the ball..
    if desired_state.ball.is_some() { panic!("desired ball not supported for now"); }

    let mut controller = BrickControllerState::new();

    let step_duration = appropriate_step(&player, &desired_state.player.unwrap());

    hybrid_a_star(player, &desired_state.player.unwrap(), step_duration)
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
    //vx: i16,
    //vy: i16,
    //vz: i16,
    //roll: i16,
    //pitch: i16,
    yaw: i16,
}


// it's a lot more expensive to do the search with small step durations, so we impose a low max
// cost in that case, which represents the amount of time we can simulate into the future. it is
// expected that the caller specifies a low step duration only when the search space is supposed to
// be quite small, and will fit in the below max cost
fn max_cost(step_duration: f32) -> f32 {
    match step_duration {
        FINE_STEP => 0.4,
        MEDIUM_STEP => 1.0,
        COARSE_STEP | VERY_COARSE_STEP => 5.0,
        _ => unimplemented!("max_cost step_duration") // we have only tuned for the values above, not allowing others for now
    }
}

#[no_mangle]
pub extern fn hybrid_a_star(current: &PlayerState, desired: &PlayerState, step_duration: f32) -> PlanResult {
    let mut to_see: BinaryHeap<SmallestCostHolder> = BinaryHeap::new();
    let mut parents: IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher> = IndexMap::default();
    let mut visualization_lines = vec![];
    let mut visualization_points = vec![];

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

    parents.insert(round_player_state(&current, step_duration, current.velocity.norm()), (start, None));

    let slop = match step_duration {
        FINE_STEP => 1.0, // TODO tune
        MEDIUM_STEP => 20.0, // TODO tune
        COARSE_STEP => 30.0, // TODO tune
        VERY_COARSE_STEP => 200.0, // TODO tune
        _ => unimplemented!("slop"),
    };

    let desired_box = BoundingBox {
        min_x: desired.position.x - slop,
        max_x: desired.position.x + slop,
        min_y: desired.position.y - slop,
        max_y: desired.position.y + slop,
        min_z: desired.position.z - slop,
        max_z: desired.position.z + slop,
    };
    visualization_lines.append(&mut desired_box.lines());

    let (_desired_roll, _desired_pitch, desired_yaw) = desired.rotation.to_euler_angles();

    let max_cost = max_cost(step_duration);
    while let Some(SmallestCostHolder { estimated_cost, cost_so_far, index, is_secondary, .. }) = to_see.pop() {

        // avoid an infinite graph search
        if cost_so_far > max_cost {
            break;
        }

        // FIXME super hack for debugging
        //if visualization_lines.len() > 20 {
        //    break;
        //}


        let line_start;
        let new_vertices = {
            let (_, (v1, maybe_v2)) = parents.get_index(index).expect("missing index in parents, shouldn't be possible");
            let vertex = if is_secondary { maybe_v2.as_ref().unwrap() }  else { v1 };
            line_start = vertex.player.position;


            let mut parent_player;
            if let Some((_, (parent_v1, maybe_parent_v2))) = parents.get_index(vertex.parent_index) {
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

            if player_goal_reached(&desired_box, desired_yaw, &vertex.player, &parent_player) {
                //println!("omg reached {}", visualization_points.len());
                return PlanResult {
                    plan: Some(reverse_path(&parents, index, is_secondary)),
                    desired: DesiredState { player: Some(desired.clone()), ball: None },
                    visualization_lines,
                    visualization_points,
                };
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
            let new_vertex_rounded = round_player_state(&new_vertex.player, step_duration, new_vertex.player.velocity.norm());
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
                            } else if e.index() == new_vertex.parent_index || new_vertex.parent_index == existing_secondary_vertex.parent_index {
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
                                if e.index() == new_vertex.parent_index && new_vertex.parent_is_secondary {
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
                                visualization_points.push((
                                    Point3::new(line_end.x, line_end.y, line_end.z),
                                    Point3::new(0.4, 0.0, 0.0),
                                ));
                                continue;
                            }
                        }
                    }

                    if let Some(v) = insertable { e.insert(v); }


                }
            }

            visualization_points.push((
                Point3::new(line_end.x, line_end.y, line_end.z),
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

    //println!("omg failed {}", visualization_points.len());
    PlanResult { plan: None, desired: DesiredState { player: Some(desired.clone()), ball: None }, visualization_lines, visualization_points }
}

fn player_goal_reached(desired_box: &BoundingBox, desired_yaw: f32, candidate: &PlayerState, previous: &PlayerState) -> bool {
    // since the ray collision algorithm doesn't allow for the ray ending early, but does account
    // for point of origin, we just test it in both directions for a complete line test. minimize
    // the overhead by using the previous position as the ray origin first, assuming we'll
    // mostly be moving *towards* the desired position for most expanded a* paths
    //
    let (_roll, _pitch, mut yaw) = candidate.rotation.to_euler_angles();
    let (_roll, _pitch, mut prev_yaw) = previous.rotation.to_euler_angles();

    // remove discontinuity, so we can average these or do 1d itersection, whichever
    if yaw < 0.0 { yaw = 2.0*PI + yaw; }
    if prev_yaw < 0.0 { prev_yaw = 2.0*PI + prev_yaw; }
    let mut desired_yaw = desired_yaw;
    if desired_yaw < 0.0 { desired_yaw = 2.0*PI + desired_yaw; }

    let division = (PI/4.0); // FIXME should not be hard-coded like this
    let rounded_yaw = ((yaw/2.0 + prev_yaw/2.0) / division).round();

    rounded_yaw == (desired_yaw / division).round() &&
        ray_collides_bounding_box(&desired_box, previous.position, candidate.position) &&
        ray_collides_bounding_box(&desired_box, candidate.position, previous.position)
}

fn reverse_path(parents: &IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher>, initial_index: usize, initial_is_secondary: bool) -> Vec<(PlayerState, BrickControllerState)> {
    let path = itertools::unfold((initial_index, initial_is_secondary), |vals| {
        let index = (*vals).0;
        let is_secondary = (*vals).1;
        parents.get_index(index).map(|(_rounded, (v1, maybe_v2))| {
            let vertex = if is_secondary { maybe_v2.as_ref().unwrap() }  else { v1 };
            (*vals).0 = vertex.parent_index;
            (*vals).1 = vertex.parent_is_secondary;
            (vertex.player, vertex.prev_controller)
        })
    }).collect::<Vec<_>>();

    path.into_iter().rev().collect()
}

fn grid_factor(step_duration: f32, speed: f32) -> f32 {
    match step_duration {
        FINE_STEP => {
            if speed < 400.0 {
                0.4
            } else if speed < 1000.0 {
                0.16 // TODO tune
            } else {
                0.08
            }
        }
        MEDIUM_STEP => 1.0, // TODO tune
        COARSE_STEP | VERY_COARSE_STEP => 1.0, // TODO tune
        _ => unimplemented!("grid factor") // we have only tuned for the values above, not allowing others for now
    }
}

pub struct BoundingBox {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl BoundingBox {
    fn lines(&self) -> Vec<(Point3<f32>, Point3<f32>, Point3<f32>)> {
        let mut corners = vec![];
        for &x in [self.min_x, self.max_x].iter() {
            for &y in [self.min_y, self.max_y].iter() {
                for &z in [self.min_z, self.max_z].iter() {
                    corners.push(Point3::new(x, y, z));
                }
            }
        }

        corners.clone().iter().flat_map(|c1: &Point3<f32>| {
            corners.iter().map(|c2: &Point3<f32>| {
                (Point3::new(c1.x, c1.y, c1.z), Point3::new(c2.x, c2.y, c2.z), Point3::new(0.0f32, 1.0f32, 0.3f32))
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>()
    }
}


// https://bheisler.github.io/post/writing-gpu-accelerated-path-tracer-part-3/
// https://gamedev.stackexchange.com/a/18459/4929
pub fn ray_collides_bounding_box(bounding_box: &BoundingBox, start: Vector3<f32>, end: Vector3<f32>) -> bool {
    let dir = end - start;
    let dir_inv = Vector3::new(1.0/dir.x, 1.0/dir.y, 1.0/dir.z);

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

    let mut grid_size = step_duration * rounded_speed * grid_factor(step_duration, speed);
    let velocity_margin = 250.0; // TODO tune
    let (roll, pitch, yaw) = player.rotation.to_euler_angles();

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
        yaw: (yaw / (PI/4.0)).round() as i16, // round to nearest pi/4 angle
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

    let current_heading = candidate.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let desired_heading = desired.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0); // FIXME cache
    let rotation_match_factor = 1.0 - na::dot(&current_heading, &desired_heading); // 0 for perfect match, -2 for exactly backwards, -1 for orthogonal
    let distance_factor = (1.0 - distance/400.0).max(0.0); // linearly reduce rotation cost the further we are away. 0 at max distance, ie yaw mismatch is complete ignored if far enough (TODO tune)
    let rotation_cost = 1.1 * distance_factor * rotation_match_factor; // constant factor here is arbitrary (TODO tune)
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
    // FIXME acceleration_time_cost causing problems, removing temporarily. bring it back when
    // round player state includes velocity again.
    movement_time_cost + rotation_cost
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
        for &step_duration in [COARSE_STEP, VERY_COARSE_STEP].iter() {
            let mut current = resting_player_state();
            current.position.y = -1000.0;
            let desired = resting_player_state();
            let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
            //assert!(plan.is_some());
            if plan.is_some(){ count += 1 } else { failures.push(step_duration) }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_fuzz1() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -500..0 {
            for &step_duration in [COARSE_STEP, VERY_COARSE_STEP].iter() {
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
                //assert!(plan.is_some());
                if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_fuzz2() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -1000..-500 {
            for &step_duration in [COARSE_STEP, VERY_COARSE_STEP].iter() {
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
                //assert!(plan.is_some());
                if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_fuzz3() {
        let mut count = 0;
        let mut failures = vec![];
        //for distance in -2000..-1000 {
        for distance in -2000..-1900 {
            for &step_duration in [COARSE_STEP, VERY_COARSE_STEP].iter() {
            //for &step_duration in [COARSE_STEP].iter() {
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
                //assert!(plan.is_some());
                if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    #[test]
    fn just_drive_straight_fuzz4() {
        let mut count = 0;
        let mut failures = vec![];
        for distance in -4000..-2000 {
            for &step_duration in [COARSE_STEP, VERY_COARSE_STEP].iter() {
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = resting_player_state();
                let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
                //assert!(plan.is_some());
                if plan.is_some(){ count += 1 } else { failures.push((step_duration, distance)) }
            }
        }
        println!("WORKED {} TIMES", count);
        println!("FAILED {} TIMES", failures.len());
        println!("FAIL PERCENT {}%", 100.0 * failures.len() as f32 / count as f32);
        //println!("FAILURES: {:?}", failures);
        assert!(failures.len() == 0);
    }

    // #[test]
    // fn unreachable() {
    //     let mut count = 0;
    //     let distance = -10_000;
    //     for &step_duration in [FINE_STEP, MEDIUM_STEP, COARSE_STEP, VERY_COARSE_STEP].iter() {
    //         let mut current = resting_player_state();
    //         current.position.y = distance as f32;
    //         let desired = resting_player_state();
    //         let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
    //         assert!(plan.is_none());
    //     }
    // }
}
