extern crate uuid;

use piston::input::{ GenericEvent };
use ai_behavior;

#[derive(Copy, Clone, PartialEq)]
pub enum TileState {
    Selected,
    Selecting,
    Idle,
}

pub struct Tile {
    pub id: uuid::Uuid,
    pub state: ai_behavior::State<TileState, ()>,
}

impl Tile {
    pub fn new(
        id: uuid::Uuid,
    ) -> Tile {
        Tile {
            id: id,
            state: ai_behavior::State::new(ai_behavior::Action(TileState::Idle)),
        }
    }

    pub fn set_state(&mut self, state: TileState) {
        self.state = ai_behavior::State::new(ai_behavior::Action(state)); 
    }
}

pub struct Tiles {
    pub tiles: Vec<Tile>,
}

impl Tiles {
    pub fn new() -> Tiles {
        Tiles {
            tiles: Vec::new(),
        }
    }
}

pub fn update_tiles<E: GenericEvent>(e: &E) {
    use current_tiles;
    use current_scene;
    use current_selection;

    let tiles = unsafe { &mut *current_tiles() };
    let scene = unsafe { &mut *current_scene() };
    let selection = unsafe { current_selection() };

    for tile in tiles.tiles.iter_mut() {
        let &mut Tile {
            ref id,
            ref mut state,
        } = tile;
        let sprite = scene.child_mut(*id).unwrap();
        state.event(e, &mut |args| {
             match *args.action {
                TileState::Selecting => {
                    if selection.released {
                        (ai_behavior::Success, 0.0)
                    } else {
                        sprite.set_color(0.2, 0.4, 0.5);
                        (ai_behavior::Running, 0.0)
                    }
                },
                TileState::Selected => {
                    sprite.set_color(0.7, 0.4, 0.5);
                    (ai_behavior::Running, 0.0)
                },
                TileState::Idle => {
                    sprite.set_color(1.0, 1.0, 1.0);
                    (ai_behavior::Running, 0.0)
                },
            }
        });
    }
}
