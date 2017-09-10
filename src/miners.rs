use tiles;
use rand;
use cgmath::Vector2;
use cgmath::prelude::*;

#[derive(Copy, Clone, PartialEq)]
pub enum MovementState {
    Moving,
    Idle,
}

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    Idle,
}

pub struct Miner {
    pub movement_state: MovementState,
    pub state: State,
    pub tile: tiles::Tile,
    pub waypoints: Vec<Vector2<f32>>,
    pub speed: f32,
}

pub struct Miners {
    pub miners: Vec<Miner>,
}

impl Miner {
    pub fn new(
        position: Vector2<f32>,
        tex_id: u32,
    ) -> Miner {
        Miner {
            tile: tiles::Tile::new(position, tex_id),
            movement_state: MovementState::Idle,
            state: State::Idle,
            waypoints: Vec::new(),
            speed: 10.0,
        }
    }
}

impl Miners {
    pub fn new(count: u8, tiles: &tiles::Tiles) -> Miners {
        let mut miners = Vec::new();
        for tile in tiles.get_random(count) {
            miners.push(Miner::new(tile.position, 2));
        }
        Miners {
            miners: miners,
        }
    }

    pub fn get_tiles(&mut self) -> Vec<&tiles::Tile> {
        self.miners.iter().map(|miner| &miner.tile).collect::<Vec<_>>()
    }

    pub fn update(&mut self, duration: f32, tiles: &tiles::Tiles) {
        for miner in self.miners.iter_mut() {
            miner.movement_state = match miner.state {
                State::Idle => {
                    if miner.waypoints.len() < 1 {
                        if rand::random::<f32>() < 0.2 {
                            miner.waypoints.push(tiles.get_closest_random(miner.tile.position).unwrap().position);
                            println!("New waypoint: from {:?} to {:?}",
                                     miner.tile.position, miner.waypoints[miner.waypoints.len() - 1]);
                            MovementState::Moving
                        } else {
                            MovementState::Idle
                        }
                    } else if (miner.tile.position - miner.waypoints[miner.waypoints.len() - 1]).magnitude() < 10.0 {
                        miner.waypoints.pop();
                        MovementState::Idle
                    } else {
                        miner.movement_state
                    }
                }
            };
            miner.tile.position = match miner.movement_state {
                MovementState::Idle => miner.tile.position,
                MovementState::Moving =>
                    calculate_point(
                        miner.tile.position, miner.waypoints[miner.waypoints.len() - 1],
                        miner.speed * duration)
            };
        }
    }
}

fn calculate_point(a: Vector2<f32>, b: Vector2<f32>, distance: f32) -> Vector2<f32> {
    a - (a - b).normalize() * distance
}
