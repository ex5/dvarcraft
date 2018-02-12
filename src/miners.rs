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
    CuttingTree,
}

pub struct Miner {
    pub movement_state: MovementState,
    pub state: State,
    pub tile: tiles::Tile,
    pub waypoints: Vec<Vector2<f32>>,
    pub speed: f32,
    pub state_counter: u32,
    pub working_on: Option<usize>,
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
            tile: tiles::Tile::new(position, tex_id, None),
            movement_state: MovementState::Idle,
            state: State::Idle,
            waypoints: Vec::new(),
            speed: 10.0,
            state_counter: 0,
            working_on: None,
        }
    }
}

impl Miners {
    pub fn new(count: u8, tiles: &tiles::Tiles) -> Miners {
        let mut miners = Vec::new();
        for tile in tiles.get_random_walkable(count) {
            miners.push(Miner::new(tile.position, ::SPRITE_MINER));
        }
        Miners {
            miners: miners,
        }
    }

    pub fn get_tiles(&mut self) -> Vec<&tiles::Tile> {
        self.miners.iter().map(|miner| &miner.tile).collect::<Vec<_>>()
    }

    pub fn update(&mut self, duration: f32, tiles: &mut tiles::Tiles) {
        for miner in self.miners.iter_mut() {
            miner.state = match miner.state {
                State::Idle => {
                    let closest_tile = tiles.resource_at(miner.tile.position);
                    if closest_tile.is_some() {
                        let r = closest_tile.unwrap();
                        if r.resource_id.map_or(false, |x| x == ::RESOURCE_WOOD) && r.resource_count > 0 && !r.can_be_carried {
                            miner.state_counter = miner.speed as u32;
                            miner.working_on = tiles.index_of(&r);
                            State::CuttingTree
                        } else {
                            State::Idle
                        }
                    } else {
                        State::Idle
                    }
                },
                State::CuttingTree => {
                    if miner.state_counter > 0 {
                        miner.state_counter -= 1;
                        State::CuttingTree
                    } else {
                        tiles.replace(miner.working_on, ::SPRITE_WOOD, true);
                        miner.working_on = None;
                        State::Idle
                    }
                }
            };
            miner.movement_state = match miner.state {
                State::Idle => {
                    if miner.waypoints.len() < 1 {
                        if rand::random::<f32>() < 0.2 {
                            let next_waypoint = tiles.get_closest_walkable(miner.tile.position);
                            if next_waypoint.is_some() {
                                miner.waypoints.push(next_waypoint.unwrap().position);
                                println!("New waypoint: from {:?} to {:?}",
                                         miner.tile.position, miner.waypoints[miner.waypoints.len() - 1]);
                                MovementState::Moving
                            } else {
                                MovementState::Idle
                            }
                        } else {
                            MovementState::Idle
                        }
                    } else if (miner.tile.position - miner.waypoints[miner.waypoints.len() - 1]).magnitude() < 2.0 {
                        miner.waypoints.pop();
                        MovementState::Idle
                    } else {
                        miner.movement_state
                    }
                },
                State::CuttingTree => {
                    miner.movement_state
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
