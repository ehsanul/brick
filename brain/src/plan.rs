use state::*;
use predict;
use heuristic::HeuristicModel;
use na::{ self, Unit, Vector3, Point3, Rotation3 };
use std::cmp::Ordering;
use std::f32::consts::PI;
use std;

use std::collections::BinaryHeap;
use std::mem;
use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;
use std::usize;
use itertools;

use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
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
            //left_boost,
            //right_boost,
            //straight_boost,

            left_throttle,
            right_throttle,
            straight_throttle,

            //left_drift_throttle,
            //right_drift_throttle,
            //straight_drift_throttle,

            //left_drift_boost,
            //right_drift_boost,
            //straight_drift_boost,

            //left_idle,
            //right_idle,
            //straight_idle,

            //left_reverse,
            //right_reverse,
            //straight_reverse,
        ]
    };
}

const EXPLODED_STEP_DURATION: f32 = 2.0 * TICK;

/// wrapper around hybrid_a_star for convenience and some extra smarts
// TODO maybe we should take the entire gamestate instead. we also need a history component, ie BotState
pub extern fn plan<H: HeuristicModel>(model: &mut H, player: &PlayerState, desired_contact: &DesiredContact, last_plan: Option<&Plan>) -> PlanResult {
    let mut config = SearchConfig::default();
    if let Some(last_plan) = last_plan {
        config.max_cost = (10.0 + last_plan.len() as f32) * EXPLODED_STEP_DURATION;
    }

    let mut plan_result = hybrid_a_star(model, player, &desired_contact, &config);
    explode_plan(&mut plan_result);
    plan_result
}

/// modifies the plan to use finer-grained steps
pub fn explode_plan(plan_result: &mut PlanResult) {

    if let Some(ref mut plan) = plan_result.plan {
        if plan.get(0).is_none() { return }
        let mut exploded_plan = Vec::with_capacity(plan.len()); // will be at least this long
        exploded_plan.push(plan[0]);

        for i in 1..plan.len() {
            assert!((plan[i].2 % EXPLODED_STEP_DURATION).abs() < 0.000001); // ensure multiple, ignoring fp inaccuracies
            let exploded_length = (plan[i].2 / EXPLODED_STEP_DURATION).round() as i32;
            let last_player = plan[i - 1].0;
            let controller = plan[i].1;

            for j in 1..=exploded_length {
                let next_player = predict::player::next_player_state(&last_player, &controller, j as f32 * EXPLODED_STEP_DURATION);
                exploded_plan.push((next_player, controller, EXPLODED_STEP_DURATION));
            }
        }

        //println!("===================================");
        //println!("original: {:?}", plan.iter().map(|(p, c, s)| {
        //    (p.position.x, p.position.y, p.velocity.x, p.velocity.y, p.rotation.euler_angles().2, p.angular_velocity.z, c.steer, s)
        //}).collect::<Vec<_>>());
        //println!("-----------------------------------");
        //println!("exploded: {:?}", exploded_plan.iter().map(|(p, c, s)| {
        //    (p.position.x, p.position.y, p.velocity.x, p.velocity.y, p.rotation.euler_angles().2, p.angular_velocity.z, c.steer, s)
        //}).collect::<Vec<_>>());
        //println!("===================================");

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

fn setup_goals(desired_contact: &Vector3<f32>, desired_hit_direction: &Unit<Vector3<f32>>, slop: f32) -> Vec<Goal> {
    let contact_to_car = -(CAR_DIMENSIONS.x/2.0) * desired_hit_direction.as_ref();
    let car_corner_distance = (CAR_DIMENSIONS.x.powf(2.0) + CAR_DIMENSIONS.y.powf(2.0)).sqrt();
    let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, PI/2.0);

    let mut goals = vec![];

    // straight on basic hit
    goals.push(Goal {
        bounding_box: BoundingBox::new(&(desired_contact + contact_to_car), slop),
        heading: -Unit::new_normalize(contact_to_car.clone()),
        min_dot: (PI/16.0).cos(), // FIXME seems this should depend on the step size!
    });

    // slightly off but straight hits
    let mut offset = slop * 2.0;
    while offset <= 21.0 {
        let offset_right = offset * (clockwise_90_rotation * Unit::new_normalize(contact_to_car.clone()).into_inner());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + contact_to_car + offset_right), slop),
            heading: Unit::new_normalize(-contact_to_car - 2.5 * offset_right),
            min_dot: (PI/16.0).cos(), // FIXME seems this should depend on the step size!
        });

        let offset_left = -offset * (clockwise_90_rotation * Unit::new_normalize(contact_to_car.clone()).into_inner());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + contact_to_car + offset_left), slop),
            heading: Unit::new_normalize(-contact_to_car - 2.5 * offset_left),
            min_dot: (PI/16.0).cos(), // FIXME seems this should depend on the step size!
        });

        offset += slop * 2.0;
    }
    let num_straight_goals = goals.len();

    // corner hits from right side of the ball (left side of the car)
    let margin = PI/6.0;
    let mut angle = -PI/2.0 + margin;
    let mut i = 0;
    while angle < -margin {
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
        let offset = (CAR_DIMENSIONS.y/2.0) * (clockwise_90_rotation * Unit::new_normalize(rotated_contact_to_car.clone()).into_inner());
        goals.push(Goal {
            bounding_box: BoundingBox::new(&(desired_contact + rotated_contact_to_car + offset), slop),
            heading: -Unit::new_normalize(rotated_contact_to_car.clone()),
            min_dot: 0.0, // set later
        });
    }

    // corner hits from left side of the ball (right side of car)
    let mut angle = PI/2.0 - margin;
    let mut i = 0;
    while angle > margin {
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
        let offset = -(CAR_DIMENSIONS.y/2.0) * (clockwise_90_rotation * Unit::new_normalize(rotated_contact_to_car.clone()).into_inner());
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
            (*goal).min_dot = (1.0 * (PI - 2.0 * margin) / num_angle_goals as f32).cos(); // TODO TUNE
        }
    }

    goals
}

fn known_unreachable(_current: &PlayerState, desired: &DesiredContact) -> bool {
    // we can't fly yet :(
    desired.position.z > BALL_RADIUS + CAR_DIMENSIONS.z
}

type ParentsMap = IndexMap<RoundedPlayerState, (PlayerVertex, Option<PlayerVertex>), MyHasher>;

pub fn hybrid_a_star<H: HeuristicModel>(model: &mut H, current: &PlayerState, desired: &DesiredContact, config: &SearchConfig) -> PlanResult {
    let mut visualization_lines = vec![];

    #[allow(unused_mut)]
    let mut visualization_points = vec![];

    if known_unreachable(&current, &desired) {
        return PlanResult { plan: None, desired: desired.clone(), visualization_lines, visualization_points }
    }

    // sets up the model for this particular prediction. it can do some calculations upfront here
    // instead of over and over again for each prediction.
    model.configure(&desired);

    let mut to_see: BinaryHeap<SmallestCostHolder> = BinaryHeap::new();
    let mut parents: ParentsMap = IndexMap::default();

    let desired_contact = desired.position;
    let desired_hit_direction = Unit::new_normalize(desired.heading.clone());
    let goals = setup_goals(&desired_contact, &desired_hit_direction, config.slop);

    let coarse_box = BoundingBox::from_boxes(&(goals.iter().map(|g| g.bounding_box.clone())).collect());
    let coarse_goal = Goal {
        bounding_box: coarse_box,
        heading: desired_hit_direction,
        min_dot: (PI/2.0 - PI/8.0).cos(),
    };

    // buffers to avoid re-allocating in a loop
    let mut new_vertices = vec![];
    let mut new_players = vec![];
    let mut heuristic_costs: Vec<f32> = vec![];
    let mut single_heuristic_cost: Vec<f32> = vec![0.0];

    model.heuristic(&[current.clone()], &mut single_heuristic_cost[0..1]).expect("Heuristic failed initial!");

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
        step_duration: 0.0,
        parent_index: usize::MAX,
        parent_is_secondary: false,
    };

    parents.insert(round_player_state(&current, config.step_duration, current.velocity.norm()), (start, None));


    for goal in goals.iter() {
        let hit_pos = goal.bounding_box.center() + (config.slop + CAR_DIMENSIONS.x/2.0)*goal.heading.as_ref();
        visualization_lines.append(&mut goal.bounding_box.lines());
        for (l, ..) in goal.bounding_box.lines() {
            visualization_lines.push((
                Point3::new(l.x, l.y, l.z),
                Point3::new(hit_pos.x, hit_pos.y, hit_pos.z),
                Point3::new(0.0, 0.0, 1.0),
            ));
        }
    }

    let mut num_iterations = 0;

    while let Some(SmallestCostHolder { estimated_cost, cost_so_far, index, is_secondary, .. }) = to_see.pop() {

        // avoid an infinite graph search
        if cost_so_far > config.max_cost {
            println!("short circuit, hit max cost!");
            break;
        }

        // HACK avoid very large searches completely
        num_iterations += 1;
        if num_iterations > config.max_iterations {
            println!("short circuit, too many iterations!");
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

            if let Some(reached_goal) = player_goal_reached(&coarse_goal, &goals, &vertex.player, &parent_player) {
                let plan = reverse_path(&parents, index, is_secondary);
                let expansions = visualization_lines.len() - 2 * goals.len() * goals[0].bounding_box.lines().len();
                let cost = plan.iter().map(|(_, _, cost)| cost).sum::<f32>();
                visualization_lines.append(&mut reached_goal.bounding_box.lines().iter().map(|l| {
                    let c1 = l.0;
                    let c2 = l.1;
                    (Point3::new(c1.x, c1.y, c1.z), Point3::new(c2.x, c2.y, c2.z), Point3::new(1.0f32, 0.0f32, 1.0f32))
                }).collect());

                println!("omg reached! step size: {} | expansions: {} | cost: {}", config.step_duration * 120.0, expansions, cost);
                return PlanResult {
                    plan: Some(plan),
                    desired: desired.clone(),
                    visualization_lines,
                    visualization_points,
                };
            }

            // We may have inserted a node several times into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            //
            // NOTE: this also achieves the same thing as checking if we are in the closed set
            if cost_so_far > vertex.cost_so_far {
                continue;
            }

            expand_vertex(index, is_secondary, &vertex, &mut new_vertices, dur, |_| true)
        };

        new_players.clear();
        new_players.extend(new_vertices.iter().map(|v| v.player));
        while heuristic_costs.len() < new_players.len() {
            heuristic_costs.push(0.0)
        }
        model.heuristic(&new_players, &mut heuristic_costs[0..new_players.len()]).expect("Heuristic failed!");

        let mut heuristic_index = 0;
        for new_vertex in new_vertices.drain(0..) {
            let new_vertex_rounded = round_player_state(&new_vertex.player, dur, new_vertex.player.velocity.norm());
            let new_cost_so_far = new_vertex.cost_so_far;
            let new_index;
            let mut new_is_secondary = false;
            let line_end = new_vertex.player.position;
            let mut new_estimated_cost = 0.0;
            let i = heuristic_index;
            heuristic_index += 1;

            match parents.entry(new_vertex_rounded) {
                Vacant(e) => {
                    new_index = e.index();
                    new_estimated_cost = new_cost_so_far + heuristic_costs[i];
                    e.insert((new_vertex, None));

                }
                Occupied(mut e) => {
                    new_index = e.index();
                    new_is_secondary = true;
                    let insertable;
                    match e.get() {
                        (existing_vertex, None) => {
                            // basically just like the vacant case
                            new_estimated_cost = new_cost_so_far + heuristic_costs[i];
                            // TODO-perf avoid the clone here. nll? worst-case, can use mem::replace with an enum
                            insertable = Some((existing_vertex.clone(), Some(new_vertex)));
                        },
                        (existing_vertex, Some(existing_secondary_vertex)) => {
                            let mut new_cost_is_lower = existing_secondary_vertex.cost_so_far > new_vertex.cost_so_far;
                            if new_cost_is_lower {
                                // turns out that the index invalidation is a problem in general
                                // with the IndexMap approach when replacing occupied entries: we
                                // end up with some nonsensical plan jumps otherwise. instead,
                                // let's limit replacements to siblings. this does mean we may miss
                                // out on the best path, but at least doesn't give us corrupted
                                // paths! if this causes us to lose too many good paths, we'll need
                                // to reconsider the use of the IndexMap
                                if existing_secondary_vertex.parent_index != new_vertex.parent_index {
                                    continue;
                                }

                                new_estimated_cost = new_cost_so_far + heuristic_costs[i];
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
                                new_estimated_cost = new_cost_so_far + heuristic_costs[i];
                                model.heuristic(&[existing_secondary_vertex.player], &mut single_heuristic_cost[0..1]).expect("Heuristic failed 2!");
                                let existing_secondary_estimated_cost = existing_secondary_vertex.cost_so_far + single_heuristic_cost[0];

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

                    if let Some(v) = insertable { e.insert(v); }
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

    println!("omg failed! step size: {} | expansions: {}", config.step_duration * 120.0, visualization_lines.len());
    PlanResult { plan: None, desired: desired.clone(), visualization_lines, visualization_points }
}

fn player_goal_reached<'a> (coarse_goal: &Goal, precise_goals: &'a Vec<Goal>, candidate: &PlayerState, previous: &PlayerState) -> Option<&'a Goal> {
    let coarse_box = coarse_goal.bounding_box;

    // NOTE we are just using the candidate. what we really want is the heading at the closest
    // positions. well, what we really really want is the heading at every intermediate position.
    // so we'd need to calculate intersection between the list of headings, and our desired
    // heading, with some tolerance. but idk the math for that yet so we're just using the
    // candidate heading
    let candidate_heading = candidate.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);

    let coarse_collision = line_collides_bounding_box(&coarse_box, previous.position, candidate.position);
    if !coarse_collision { return None };

    precise_goals.iter().find(|goal| {
        let correct_precise_direction = na::Matrix::dot(goal.heading.as_ref(), &candidate_heading) > goal.min_dot;
        correct_precise_direction &&
            line_collides_bounding_box(&goal.bounding_box, previous.position, candidate.position)
    })
}

fn reverse_path(parents: &ParentsMap, initial_index: usize, initial_is_secondary: bool) -> Plan {
    let path = itertools::unfold((initial_index, initial_is_secondary), |vals| {
        let index = (*vals).0;
        let is_secondary = (*vals).1;
        parents.get_index(index).map(|(_rounded, (v1, maybe_v2))| {
            let vertex = if is_secondary { maybe_v2.as_ref().unwrap() }  else { v1 };
            (*vals).0 = vertex.parent_index;
            (*vals).1 = vertex.parent_is_secondary;
            (vertex.player, vertex.prev_controller, vertex.step_duration)
        })
    }).collect::<Vec<_>>();

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

// the margin is to allow hitting on the curve or from inside the goal. this is only needed until
// we get prediction for driving on the curves/walls
// TODO well we should model the goal area as drivable too
const BOUNDS_MARGIN: f32 = 200.0;
fn out_of_bounds(player: &PlayerState) -> bool {
    let pos = player.position;
    pos.x > SIDE_WALL_DISTANCE + BOUNDS_MARGIN || pos.x < -SIDE_WALL_DISTANCE - BOUNDS_MARGIN ||
        pos.y > BACK_WALL_DISTANCE + BOUNDS_MARGIN || pos.y < -BACK_WALL_DISTANCE - BOUNDS_MARGIN
}

fn expand_vertex(index: usize, is_secondary: bool, vertex: &PlayerVertex, new_vertices: &mut Vec<PlayerVertex>, step_duration: f32, custom_filter: fn(&PlayerVertex) -> bool) {
    let iterator = control_branches(&vertex.player).iter().map(|&controller| {
        PlayerVertex {
            player: predict::player::next_player_state(&vertex.player, &controller, step_duration),
            cost_so_far: vertex.cost_so_far + step_duration,
            prev_controller: controller,
            step_duration: step_duration,
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

    fn resting_position() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_velocity() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_rotation() -> UnitQuaternion<f32> { UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0) }

    fn resting_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            angular_velocity: resting_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
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
        let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, 0.5);
        //assert!(plan.is_some());
        if plan.is_some(){ count += 1 } else { failures.push(true) }
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
            for &step_duration in [0.5].iter() {
                let mut current = resting_player_state();
                current.position.y = distance as f32;
                let desired = test_desired_contact();
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
    // fn just_drive_straight_fuzz2() {
    //     let mut count = 0;
    //     let mut failures = vec![];
    //     for distance in -1000..-500 {
    //         for &step_duration in [0.5].iter() {
    //             let mut current = resting_player_state();
    //             current.position.y = distance as f32;
    //             let desired = test_desired_contact();
    //             let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
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
    //             let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
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
    //             let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
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
            let PlanResult { mut plan, .. } = hybrid_a_star(&current, &desired, step_duration);
            assert!(plan.is_none());
        }
    }
}
