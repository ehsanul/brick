use na::{Unit, Vector3};
use state::*;
use std::error::Error;

use crate::HeuristicModel;

#[derive(Debug)]
pub struct BasicHeuristic {
    pub(crate) goal_center: Vector3<f32>,
    pub(crate) desired_heading: Vector3<f32>,
    pub(crate) scale: f32,
}

impl BasicHeuristic {
    pub(crate) fn single_heuristic(&self, player: &PlayerState) -> f32 {
        // basic heuristic cost is a lower-bound for how long it would take, given max boost, to reach
        // the desired position and velocity. and we need to do rotation too.
        //
        // NOTE for now we ignore the fact that we are not starting at the max boost velocity pointed
        // directly at the desired position. the heuristic just needs to be a lower bound, until we
        // want to get it more accurate and thus ignore irrelevant branches more efficiently.
        let towards_goal = self.goal_center - player.position;
        let distance = towards_goal.norm();

        // XXX more correct to use predict::player::MAX_BOOST_SPEED, but it checks way too many paths.
        // with a lower value, ie higher heuristic cost, we get a potentially less optimal path, but we
        // get it a lot faster. it's not so bad given that we aren't actually going in a straight line
        // boosting at max speed anyways
        //
        //let movement_time_cost = distance / 1850.0;
        //
        // XXX we are boosting now so have to up this to 2300.0 for good paths... but for
        // a no-boost path plan, or just one without infinite boost, it makes sense to have a lower
        // number here...
        let movement_time_cost = distance / 2300.0;

        // basic penalty for being on the wrong side of the ball which will require a big turn. this
        // allows us to forgo searching right near the ball on the wrong side when it'll never work
        // out.
        let current_heading = player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let car_to_desired = Unit::new_normalize(self.goal_center - player.position).into_inner();

        #[allow(clippy::if_same_then_else)]
        let mut penalty_time_cost = if distance < 800.0 && na::Matrix::dot(&self.desired_heading, &car_to_desired) < -0.70 {
            0.5
        } else if distance < 1500.0 && na::Matrix::dot(&self.desired_heading, &car_to_desired) < -0.88 {
            0.5
        } else if distance < 2000.0 && na::Matrix::dot(&self.desired_heading, &car_to_desired) < -0.95 {
            0.5
        } else {
            0.0
        };

        // we have a tighter radius when slow, the numbers above are tuned for going fast
        if player.velocity.norm() < 800.0 {
            penalty_time_cost *= 0.2;
        }
        // if passing sideways, the penalty should be way lower since we're moving out of the deadzone
        penalty_time_cost *= na::Matrix::dot(&current_heading, &car_to_desired).abs();

        movement_time_cost + penalty_time_cost
    }
}

impl Default for BasicHeuristic {
    fn default() -> BasicHeuristic {
        BasicHeuristic {
            goal_center: Vector3::new(0.0, 0.0, 0.0),
            desired_heading: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}

impl HeuristicModel for BasicHeuristic {
    fn unscaled_heuristic(&mut self, players: &[PlayerState], costs: &mut [f32]) -> Result<(), Box<dyn Error>> {
        assert!(players.len() == costs.len());
        for (i, cost) in costs.iter_mut().enumerate() {
            let player = unsafe { players.get_unchecked(i) };
            *cost = self.single_heuristic(player);
        }

        Ok(())
    }

    fn scale(&self) -> f32 {
        self.scale
    }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.desired_heading = Unit::new_normalize(desired.heading).into_inner();
        self.goal_center = desired.position - (CAR_DIMENSIONS.x / 2.0) * self.desired_heading;
        self.scale = scale;
    }

    fn ball_configure(&mut self, ball: &BallState, ball_goal: &Vector3<f32>) {
        self.desired_heading = Unit::new_normalize(ball_goal - ball.position).into_inner();
        self.goal_center =
            ball.position - (BALL_COLLISION_RADIUS + (CAR_DIMENSIONS.x / 2.0) + CAR_OFFSET.x.abs()) * self.desired_heading;
    }
}
