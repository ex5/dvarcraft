use tiles;
use rand;

#[derive(Copy, Clone, PartialEq)]
pub enum MinerState {
    Moving,
    Idle,
}

pub struct Miner {
    pub state: MinerState,
    pub tile: tiles::Tile,
}

pub struct Miners {
    pub miners: Vec<Miner>,
}

impl Miner {
    pub fn new(
        position: (f32, f32),
        tex_id: u32,
    ) -> Miner {
        Miner {
            tile: tiles::Tile::new(position, tex_id),
            state: MinerState::Idle,
        }
    }
}

impl Miners {
    pub fn new(count: u8) -> Miners {
        let mut miners = Vec::new();
        for i in 0..count {
            miners.push(Miner::new((800.0 * rand::random::<f32>(),
                                    600.0 * rand::random::<f32>()), 2));
        }
        Miners {
            miners: miners,
        }
    }

    pub fn get_tiles(&mut self) -> Vec<&tiles::Tile> {
        self.miners.iter().map(|miner| &miner.tile).collect::<Vec<_>>()
    }
}
