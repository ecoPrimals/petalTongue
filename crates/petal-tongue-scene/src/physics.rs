// SPDX-License-Identifier: AGPL-3.0-only
//! Physics bridge types for barraCuda integration.
//!
//! These types serialize to/from IPC requests for the `math.physics.nbody` RPC.
//! CPU fallback uses simple Euler integration; production uses barraCuda.

use serde::{Deserialize, Serialize};

/// Collision shape for physics bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionShape {
    /// Sphere with given radius.
    Sphere { radius: f64 },
    /// Axis-aligned box with half-extents.
    Box { half_extents: [f64; 3] },
    /// No collision (e.g. sensor).
    None,
}

/// A single physics body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    /// Unique identifier.
    pub id: String,
    /// Mass in kg.
    pub mass: f64,
    /// Position [x, y, z].
    pub position: [f64; 3],
    /// Velocity [vx, vy, vz].
    pub velocity: [f64; 3],
    /// Collision shape.
    pub collision_shape: CollisionShape,
}

/// Physics world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsWorld {
    /// All bodies in the simulation.
    pub bodies: Vec<PhysicsBody>,
    /// Gravity vector [x, y, z].
    pub gravity: [f64; 3],
    /// Time step for integration (seconds).
    pub time_step: f64,
}

impl PhysicsWorld {
    /// Create a new physics world with default gravity [0, -9.81, 0].
    #[must_use]
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            time_step: 1.0 / 60.0,
        }
    }

    /// Add a body to the world.
    pub fn add_body(&mut self, body: PhysicsBody) {
        self.bodies.push(body);
    }

    /// Simple Euler integration (CPU fallback).
    /// position += velocity * dt
    /// velocity += gravity * dt
    pub fn step_euler(&mut self) {
        let dt = self.time_step;
        let [gx, gy, gz] = self.gravity;
        for body in &mut self.bodies {
            body.position[0] += body.velocity[0] * dt;
            body.position[1] += body.velocity[1] * dt;
            body.position[2] += body.velocity[2] * dt;
            body.velocity[0] += gx * dt;
            body.velocity[1] += gy * dt;
            body.velocity[2] += gz * dt;
        }
    }

    /// Serialize to JSON for `math.physics.nbody` IPC call.
    #[must_use]
    pub fn to_ipc_request(&self) -> serde_json::Value {
        let bodies: Vec<serde_json::Value> = self
            .bodies
            .iter()
            .map(|b| {
                serde_json::json!({
                    "id": b.id,
                    "mass": b.mass,
                    "position": b.position,
                    "velocity": b.velocity,
                    "collision_shape": serde_json::to_value(&b.collision_shape).unwrap_or(serde_json::Value::Null)
                })
            })
            .collect();
        serde_json::json!({
            "bodies": bodies,
            "gravity": self.gravity,
            "time_step": self.time_step
        })
    }

    /// Update positions from barraCuda response.
    pub fn apply_ipc_response(&mut self, response: &serde_json::Value) {
        let Some(bodies_arr) = response.get("bodies").and_then(|v| v.as_array()) else {
            return;
        };
        for body_val in bodies_arr {
            let Some(id) = body_val.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(pos) = body_val.get("position").and_then(|v| v.as_array()) else {
                continue;
            };
            if pos.len() < 3 {
                continue;
            }
            let px = pos[0].as_f64().unwrap_or(0.0);
            let py = pos[1].as_f64().unwrap_or(0.0);
            let pz = pos[2].as_f64().unwrap_or(0.0);
            if let Some(body) = self.bodies.iter_mut().find(|b| b.id == id) {
                body.position = [px, py, pz];
            }
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_body_creation() {
        let body = PhysicsBody {
            id: "b1".to_string(),
            mass: 1.0,
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            collision_shape: CollisionShape::Sphere { radius: 0.5 },
        };
        assert_eq!(body.id, "b1");
        assert!((body.mass - 1.0).abs() < 1e-10);
        assert_eq!(body.position, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn physics_world_step_euler_updates_position() {
        let mut world = PhysicsWorld::new();
        world.gravity = [0.0, 0.0, 0.0];
        world.time_step = 1.0;
        world.add_body(PhysicsBody {
            id: "b1".to_string(),
            mass: 1.0,
            position: [0.0, 0.0, 0.0],
            velocity: [10.0, 0.0, 0.0],
            collision_shape: CollisionShape::None,
        });
        world.step_euler();
        assert!((world.bodies[0].position[0] - 10.0).abs() < 1e-10);
        assert!((world.bodies[0].position[1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn physics_world_gravity_affects_velocity() {
        let mut world = PhysicsWorld::new();
        world.gravity = [0.0, -10.0, 0.0];
        world.time_step = 1.0;
        world.add_body(PhysicsBody {
            id: "b1".to_string(),
            mass: 1.0,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            collision_shape: CollisionShape::None,
        });
        world.step_euler();
        assert!((world.bodies[0].velocity[1] - (-10.0)).abs() < 1e-10);
    }

    #[test]
    fn to_ipc_request_produces_valid_json_with_bodies() {
        let mut world = PhysicsWorld::new();
        world.add_body(PhysicsBody {
            id: "b1".to_string(),
            mass: 2.0,
            position: [1.0, 2.0, 3.0],
            velocity: [0.0, 0.0, 0.0],
            collision_shape: CollisionShape::Sphere { radius: 1.0 },
        });
        let req = world.to_ipc_request();
        assert!(req.get("bodies").is_some());
        let bodies = req.get("bodies").unwrap().as_array().unwrap();
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].get("id").unwrap().as_str().unwrap(), "b1");
    }

    #[test]
    fn apply_ipc_response_updates_positions() {
        let mut world = PhysicsWorld::new();
        world.add_body(PhysicsBody {
            id: "b1".to_string(),
            mass: 1.0,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            collision_shape: CollisionShape::None,
        });
        let response = serde_json::json!({
            "bodies": [
                {"id": "b1", "position": [5.0, 10.0, 15.0], "velocity": [0.0, 0.0, 0.0]}
            ]
        });
        world.apply_ipc_response(&response);
        assert!((world.bodies[0].position[0] - 5.0).abs() < 1e-10);
        assert!((world.bodies[0].position[1] - 10.0).abs() < 1e-10);
        assert!((world.bodies[0].position[2] - 15.0).abs() < 1e-10);
    }
}
