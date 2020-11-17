# Brick

Brick is a bot that plays Rocket League.

![Goal 2](https://i.ibb.co/tPK1tsr/goal2.gif)

![Goal 1](https://i.ibb.co/pWLbzCz/goal1.gif)

## How it works

Brick uses the hybrid A* algorithm to control the car. Consider that at each
point of time, the car can make a limited number of moves, eg turn left,
reverse, jump, etc. The moves in the future can thus be considered a tree of
possibilities, and thus a tree search may be used to find a series of moves
that leads to a desired result.

In Brick's case, the desired result is to shoot the ball into the opponent's
goal. This is the primary exit condition of the algorithm, at least when it is
successful.

What complicates matters is the fact a car in Rocket League is physics-based,
and has continuous values for position, velocity, angular velocity and
rotation. So a straightfoward use of the A* algorithm isn't appropriate, as
that will end up evaluating far more of the search space than necessary, by
evaluating points that are almost identical to each other.

That's where the "hybrid" part comes in: nodes in the search path maintain some
metadata about the exact value of the kinematic state of the car, but are
otherwise divided into bins in a grid. This greatly reduces the search
complexity, while giving up optimality, but maintains correctness wrt the car's
kinematics. This is described in the paper "Practical Search Techniques in Path
Planning for Autonomous Driving" by Sebastian Thrun et al.

The following is a visualization of the search process in a simulation of
Rocket League:

![Visualization of Hybrid
A Star](https://i.ibb.co/DK7TCTy/2020-11-15-18-56-39.gif)
