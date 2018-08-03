use state::*;
use predict;
use predict::arena::*;
use na::{ self, Unit, Vector3, Point3, Rotation3, UnitQuaternion };
use std::cmp::Ordering;
use std::f32::consts::PI;
use std;

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
const VERY_COARSE_STEP: f32 = 60.0 / predict::FPS;
pub(crate) fn appropriate_step(current: &PlayerState, desired: &DesiredContact) -> f32 {
    let speed = current.velocity.norm();
    let delta = desired.position - current.position;
    let distance = delta.norm();
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let dot = na::dot(&current_heading, &Unit::new_normalize(delta).unwrap());
    if distance < -200.0 {
        // XXX this was attempted to be tuned per speed, but turns out that with lower speed, we
        // don't go as far along any turning curves, and thus we end up in the same angle, just
        // earlier
        let min_dot = 0.95;

        // check if the desired state within a cone around our heading, whose angle is determined by the speed
        if dot > min_dot {
            FINE_STEP
        } else {
            // we'll probably have to go the long way around as we aren't facing the right way for
            // this, so use a coarse step
            COARSE_STEP
        }
    } else if distance < -200.0 {

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
    } else if distance < 2000.0 && distance > 500.0 && dot > 0.5  {
        COARSE_STEP
    } else {
        VERY_COARSE_STEP
    }
}

/// given the current state and a desired state, return one frame of input that will take us to the
/// desired state, if any is possible.
// TODO maybe we should take the entire gamestate instead. we also need a history component
#[no_mangle]
pub extern fn plan(player: &PlayerState, ball: &BallState, desired_contact: &DesiredContact) -> PlanResult {
    let mut controller = BrickControllerState::new();

    let step_duration = appropriate_step(&player, &desired_contact);

    let mut plan_result = hybrid_a_star(player, &desired_contact, step_duration);
    explode_plan(&mut plan_result, step_duration);
    plan_result
}

/// changes the plan so that it covers 120fps ticks, in case it was created with larger steps
fn explode_plan(plan_result: &mut PlanResult, step_duration: f32) {
    // we would get slightly off results with the method below if we don't have an exact multiple
    let ticks_per_step = (step_duration / predict::TICK).round() as usize;
    assert!(120 % ticks_per_step == 0);

    if let Some(ref mut plan) = plan_result.plan {
        if plan.get(0).is_none() { return }
        let exploded_length = (plan.len() - 1) * ticks_per_step; // first item in plan is current position
        let mut exploded_plan = vec![];

        let mut last_index = 0;
        let mut last_player = plan[0].0;
        let mut last_controller = plan[0].1;
        for i in 0..exploded_length {
            let t = i as f32 * predict::TICK;

            // index of the controller vlue we want to apply
            // again, first item is current position, we need controller on next item
            let original_index = 1 + (t / step_duration) as usize;
            if original_index > last_index {
                last_index = original_index;

                // player is from iteration before. by setting it here, we are re-calibrating
                // to the coarse path. this could help in case there is somehow a divergence in
                // how we calculate at finer time steps (which wouldn't be good at all, but
                // might happen regardless).
                last_player = plan[original_index - 1].0;
            }
            let controller = plan[original_index].1;
            let next_player = predict::player::next_player_state(&last_player, &controller, predict::TICK);
            exploded_plan.push((next_player, controller));
            last_player = next_player;

            // XXX HACK ALERT start turning a few frames early since there's a delay or something. but don't end turning early? tbd.
            if i >= 4 && controller.steer != last_controller.steer {
                exploded_plan[i-1].1.steer = controller.steer;
                exploded_plan[i-2].1.steer = controller.steer;
                if controller.steer != Steer::Straight  {
                    exploded_plan[i-3].1.steer = controller.steer;
                    exploded_plan[i-4].1.steer = controller.steer;
                }
            }
            last_controller = controller;
        }
        plan.clear();
        plan.append(&mut exploded_plan);
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
        FINE_STEP => 0.6,
        MEDIUM_STEP => 1.0,
        COARSE_STEP | VERY_COARSE_STEP => 10.0,
        _ => unimplemented!("max_cost step_duration") // we have only tuned for the values above, not allowing others for now
    }
}

fn setup_goals(desired_contact: &Vector3<f32>, desired_hit_direction: &Unit<Vector3<f32>>, slop: f32) -> Vec<Goal> {
    let contact_to_car = -(CAR_DIMENSIONS.x/2.0) * desired_hit_direction.as_ref();
    let car_corner_distance = (CAR_DIMENSIONS.x.powf(2.0) + CAR_DIMENSIONS.y.powf(2.0)).sqrt();
    let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, PI/2.0);
    let anti_clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, -PI/2.0);

    let mut goals = vec![];

    // straight on basic hit
    goals.push(Goal {
        bounding_box: BoundingBox::new(&(desired_contact + contact_to_car), slop),
        heading: -Unit::new_normalize(contact_to_car.clone()),
        min_dot: (PI/8.0).cos(), // FIXME seems this should depend on the step size!
    });

    // slightly off but straight hits
    let mut offset = slop * 2.0;
    while offset <= 21.0 {
        let offset_right = offset * (clockwise_90_rotation * Unit::new_normalize(contact_to_car.clone()).unwrap());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + contact_to_car + offset_right), slop),
            heading: Unit::new_normalize(-contact_to_car - 2.5 * offset_right),
            min_dot: (PI/8.0).cos(), // FIXME seems this should depend on the step size!
        });

        let offset_left = -offset * (clockwise_90_rotation * Unit::new_normalize(contact_to_car.clone()).unwrap());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + contact_to_car + offset_left), slop),
            heading: Unit::new_normalize(-contact_to_car - 2.5 * offset_left),
            min_dot: (PI/8.0).cos(), // FIXME seems this should depend on the step size!
        });

        offset += slop * 2.0;
    }
    let num_straight_goals = goals.len();

    // corner hits from right side of the ball (left side of the car)
    let mut angle = -PI/2.0 + PI/8.0;
    let mut i = 0;
    while angle < -PI/8.0 {
        // largest arc allowed for bounding box furthest point, to ensure coverage of the entire
        // bounding arc. XXX for larger slops, we may want to still have a smaller max arc, to create
        // more goals with slightly different angles
        // FIXME this weird log thingy is just a hack to get better coverage in the arc. we don't
        // have good coverage because we are iterating over angles rotating about the wrong point!
        // we need to rotate about the corner, not about front center
        let max_arc_length = i as f32 * slop * 1.2 / (1.1 + (i as f32)).log(2.0);
        i += 1;

        // FIXME we should be rotating about the corner too instead of the front center
        angle += max_arc_length / car_corner_distance;
        let rotation = Rotation3::from_euler_angles(0.0, 0.0, angle);
        let rotated_contact_to_car = rotation * contact_to_car;
        // corner offset from front center
        let offset = (CAR_DIMENSIONS.y/2.0) * (clockwise_90_rotation * Unit::new_normalize(rotated_contact_to_car.clone()).unwrap());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + rotated_contact_to_car + offset), slop),
            heading: -Unit::new_normalize(rotated_contact_to_car.clone()),
            min_dot: 0.0, // set later
        });
    }

    // corner hits from left side of the ball (right side of car)
    let mut angle = PI/2.0 - PI/8.0;
    let mut i = 0;
    while angle > PI/8.0 {
        // largest arc allowed for bounding box furthest point, to ensure coverage of the entire
        // bounding arc. XXX for larger slops, we may want to still have a smaller max arc, to create
        // more goals with slightly different angles
        // FIXME this weird log thingy is just a hack to get better coverage in the arc. we don't
        // have good coverage because we are iterating over angles rotating about the wrong point!
        // we need to rotate about the corner, not about front center
        let max_arc_length = i as f32 * slop * 1.2 / (1.1 + (i as f32)).log(2.0);
        i += 1;

        // FIXME we should be rotating about the corner too instead of the front center
        angle -= max_arc_length / car_corner_distance;
        let rotation = Rotation3::from_euler_angles(0.0, 0.0, angle);
        let rotated_contact_to_car = rotation * contact_to_car;
        // corner offset from front center
        let offset = -(CAR_DIMENSIONS.y/2.0) * (clockwise_90_rotation * Unit::new_normalize(rotated_contact_to_car.clone()).unwrap());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + rotated_contact_to_car + offset), slop),
            heading: -Unit::new_normalize(rotated_contact_to_car.clone()),
            min_dot: 0.0, // set later
        });
    }

    // set the desired accuracy based on how many different angles we're checking for. the desired
    // accuracy goes up as we check more angles
    let num_angle_goals = goals.len() - num_straight_goals;
    for goal in goals.iter_mut() {
        if goal.min_dot == 0.0 {
            (*goal).min_dot = ( (PI - PI/8.0 - PI/8.0) / num_angle_goals as f32 ).cos();
        }
    }

    goals
}

fn known_unreachable(current: &PlayerState, desired: &DesiredContact) -> bool {
    // we can't fly yet :(
    desired.position.z > BALL_RADIUS + CAR_DIMENSIONS.z
}

type ParentsMap = IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher>;

#[no_mangle]
pub extern fn hybrid_a_star(current: &PlayerState, desired: &DesiredContact, step_duration: f32) -> PlanResult {
    let mut visualization_lines = vec![];
    let mut visualization_points = vec![];

    if known_unreachable(&current, &desired) {
        return PlanResult { plan: None, desired: desired.clone(), visualization_lines, visualization_points }
    }

    let mut to_see: BinaryHeap<SmallestCostHolder> = BinaryHeap::new();
    let mut parents: ParentsMap = IndexMap::default();

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
        VERY_COARSE_STEP => 100.0, // TODO tune
        _ => unimplemented!("slop"),
    };

    let desired_contact = desired.position;
    let desired_hit_direction = Unit::new_normalize(desired.heading.clone());
    let goals = setup_goals(&desired_contact, &desired_hit_direction, slop);

    let coarse_box = BoundingBox::from_boxes(&(goals.iter().map(|g| g.bounding_box.clone())).collect());
    let coarse_goal = Goal {
        bounding_box: coarse_box,
        heading: desired_hit_direction,
        min_dot: (PI/2.0 - PI/8.0).cos(),
    };


    for goal in goals.iter() {
        let hit_pos = goal.bounding_box.center() + (slop + CAR_DIMENSIONS.x/2.0)*goal.heading.as_ref();
        visualization_lines.append(&mut goal.bounding_box.lines());
        for (l, ..) in goal.bounding_box.lines() {
            visualization_lines.push((
                Point3::new(l.x, l.y, l.z),
                Point3::new(hit_pos.x, hit_pos.y, hit_pos.z),
                Point3::new(0.0, 0.0, 1.0),
            ));
        }
    }

    let max_cost = max_cost(step_duration);
    let mut num_iterations = 0;
    let max_iterations = match step_duration {
        VERY_COARSE_STEP => 4000,
        _ => 10_000,
    };
    while let Some(SmallestCostHolder { estimated_cost, cost_so_far, index, is_secondary, .. }) = to_see.pop() {

        // avoid an infinite graph search
        if cost_so_far > max_cost {
            println!("short circuit, hit max cost!");
            break;
        }

        // HACK avoid very large searches completely
        num_iterations += 1;
        if num_iterations > max_iterations {
            println!("short circuit, too many iterations!");
            break;
        }

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

            if player_goal_reached(&coarse_goal, &goals, &vertex.player, &parent_player) {
                println!("omg reached! step size: {} | expensions: {}", step_duration * 120.0, visualization_points.len());
                return PlanResult {
                    plan: Some(reverse_path(&parents, index, is_secondary)),
                    desired: desired.clone(),
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

            expand_vertex(index, is_secondary, &vertex, step_duration, |_| true)
        };

        for new_vertex in new_vertices {
            let new_vertex_rounded = round_player_state(&new_vertex.player, step_duration, new_vertex.player.velocity.norm());
            let new_cost_so_far = new_vertex.cost_so_far;
            let new_index;
            let mut new_is_secondary = false;
            let line_end = new_vertex.player.position;
            let mut new_estimated_cost = 0.0;

            match parents.entry(new_vertex_rounded) {
                Vacant(e) => {
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
                            // basically just like the vacant case
                            new_estimated_cost = new_cost_so_far + heuristic_cost(&new_vertex.player, &desired);
                            insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                        },
                        (existing_vertex, Some(existing_secondary_vertex)) => {
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
                                }
                            }

                            if new_cost_is_lower {
                                insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                            } else {
                                visualization_points.push((
                                    Point3::new(line_end.x, line_end.y, line_end.z),
                                    Point3::new(0.4, 0.0, 0.0),
                                ));
                                //visualization_lines.push((
                                //    Point3::new(line_start.x, line_start.y, line_start.z),
                                //    Point3::new(line_end.x, line_end.y, line_end.z),
                                //    Point3::new(0.4, 0.0, 0.0),
                                //));
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
            //visualization_lines.push((
            //    Point3::new(line_start.x, line_start.y, line_start.z),
            //    Point3::new(line_end.x, line_end.y, line_end.z),
            //    Point3::new(0.6, 0.6, 0.6),
            //));

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

    println!("omg failed! step size: {} | expensions: {}", step_duration * 120.0, visualization_points.len());
    PlanResult { plan: None, desired: desired.clone(), visualization_lines, visualization_points }
}

fn player_goal_reached(coarse_goal: &Goal, precise_goals: &Vec<Goal>, candidate: &PlayerState, previous: &PlayerState) -> bool {
    let coarse_box = coarse_goal.bounding_box;
    let coarse_heading = coarse_goal.heading;

    // NOTE we are just using the candidate. what we really want is the heading at the closest
    // positions. well, what we really really want is the heading at every intermediate position.
    // so we'd need to calculate intersection between the list of headings, and our desired
    // heading, with some tolerance. but idk the math for that yet so we're just using the
    // candidate heading
    let candidate_heading = candidate.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    // pretty broad angle check. does this actually make it faster?
    let correct_coarse_direction = na::dot(coarse_heading.as_ref(), &candidate_heading) > coarse_goal.min_dot;

    if !correct_coarse_direction { return false };

    let coarse_collision = line_collides_bounding_box(&coarse_box, previous.position, candidate.position);
    if !coarse_collision { return false };

    precise_goals.iter().any(|goal| {
        let correct_precise_direction = na::dot(goal.heading.as_ref(), &candidate_heading) > goal.min_dot;
        correct_precise_direction &&
            line_collides_bounding_box(&goal.bounding_box, previous.position, candidate.position)
    })
}

fn reverse_path(parents: &ParentsMap, initial_index: usize, initial_is_secondary: bool) -> Vec<(PlayerState, BrickControllerState)> {
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
                0.12 // TODO tune
            } else {
                0.08
            }
        }
        MEDIUM_STEP => 1.0, // TODO tune
        COARSE_STEP | VERY_COARSE_STEP => 1.0, // TODO tune
        _ => unimplemented!("grid factor") // we have only tuned for the values above, not allowing others for now
    }
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
        if bb.max_z - bb.min_z < BALL_RADIUS * 2.0 { bb.min_z = (bb.min_z + bb.max_z) / 2.0 - BALL_RADIUS }
        bb
    }

    fn from_boxes(boxes: &Vec<BoundingBox>) -> BoundingBox {
        let mut min_x = std::f32::MAX;
        let mut min_y = std::f32::MAX;
        let mut min_z = std::f32::MAX;
        let mut max_x = std::f32::MIN;
        let mut max_y = std::f32::MIN;
        let mut max_z = std::f32::MIN;
        for b in boxes {
            if b.min_x < min_x { min_x = b.min_x }
            if b.min_y < min_y { min_y = b.min_y }
            if b.min_z < min_z { min_z = b.min_z }
            if b.max_x > max_x { max_x = b.max_x }
            if b.max_y > max_y { max_y = b.max_y }
            if b.max_z > max_z { max_z = b.max_z }
        }
        // FIXME hack to extend bounds down to the ground, where the car can reach them
        if max_z - min_z < BALL_RADIUS * 2.0 { min_z = (min_z + max_z) / 2.0 - BALL_RADIUS }
        BoundingBox { min_x, max_x, min_y, max_y, min_z, max_z }
    }

    fn center(&self) -> Vector3<f32> {
        Vector3::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
            (self.min_z + self.max_z) / 2.0,
        )
    }

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

//// since the ray collision algorithm doesn't allow for the ray ending early, but does account
//// for point of origin, we just test it in both directions for a complete line test. minimize
//// the overhead by using the previous position as the ray origin first, assuming we'll
//// mostly be moving *towards* the desired position for most expanded a* paths
pub fn line_collides_bounding_box(bounding_box: &BoundingBox, start: Vector3<f32>, end: Vector3<f32>) -> bool {
    ray_collides_bounding_box(&bounding_box, start, end) &&
        ray_collides_bounding_box(&bounding_box, end, start)
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

fn out_of_bounds(player: &PlayerState) -> bool {
    let pos = player.position;
    pos.x > SIDE_WALL_DISTANCE || pos.x < -SIDE_WALL_DISTANCE ||
        pos.y > BACK_WALL_DISTANCE || pos.y < -BACK_WALL_DISTANCE
}

fn expand_vertex(index: usize, is_secondary: bool, vertex: &PlayerVertex, step_duration: f32, custom_filter: fn(&PlayerVertex) -> bool) -> Vec<PlayerVertex> {
    control_branches(&vertex.player).iter().map(|&controller| {
        PlayerVertex {
            player: predict::player::next_player_state(&vertex.player, &controller, step_duration),
            cost_so_far: vertex.cost_so_far + step_duration,
            prev_controller: controller,
            parent_index: index,
            parent_is_secondary: is_secondary,
        }
    }).filter(|new_vertex| {
        // if parent (ie vertex) is already out of bounds, allow going out of bounds since we need
        // to be able to move back in if we start planning from out of bounds (eg player inside
        // goal)
        // TODO ideally we'd only allow if we're getting *less* out of bounds than before
        let outside_arena = out_of_bounds(&new_vertex.player) && !out_of_bounds(&vertex.player);
        !outside_arena && custom_filter(&new_vertex)
    }).collect::<Vec<PlayerVertex>>()
}

fn heuristic_cost(candidate: &PlayerState, desired: &DesiredContact) -> f32 {
    // basic heuristic cost is a lower-bound for how long it would take, given max boost, to reach
    // the desired position and velocity. and we need to do rotation too.
    //
    // NOTE for now we ignore the fact that we are not starting at the max boost velocity pointed
    // directly at the desired position. the heuristic just needs to be a lower bound, until we
    // want to get it more accurate and thus ignore irrelevant branches more efficiently.
    let towards_contact = desired.position - candidate.position;
    let distance = towards_contact.norm();
    //let towards_contact_heading = Unit::new_normalize(towards_contact).unwrap();
    let movement_time_cost = distance / 1200.0; // FIXME should use predict::player::MAX_BOOST_SPEED, but it checks too many paths. we just get a slightly less optimal path, but get it a lot faster

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
    //let relative_velocity = (desired.velocity - candidate.velocity).norm();
    //let acceleration_time_cost = relative_velocity / predict::player::BOOST_ACCELERATION_FACTOR;

    // let current_heading = candidate.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    // let desired_contact_heading = Unit::new_normalize(desired.heading.clone()).unwrap(); // FIXME normalize before
    // //let rotation_match_factor = 1.0 - na::dot(&current_heading, &desired_contact_heading); // 0 for perfect match, -2 for exactly backwards, -1 for orthogonal
    // let rotation_match_factor = 1.0 - na::dot(&current_heading, &towards_contact_heading); // 0 for perfect match, -2 for exactly backwards, -1 for orthogonal
    // let distance_factor = (1.0 - distance/1000.0).max(0.0); // linearly reduce rotation cost the further we are away. 0 at max distance, ie yaw mismatch is complete ignored if far enough (TODO tune)
    // let rotation_cost = 1.1 * distance_factor * rotation_match_factor; // constant factor here is arbitrary (TODO tune)
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
    movement_time_cost //+ rotation_cost
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
