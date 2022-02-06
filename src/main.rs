use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;

pub struct State {
  ecs: World,
}

impl State {
  fn run_systems(&mut self) {
    self.ecs.maintain();
  }
}

impl GameState for State {
  fn tick(&mut self, ctx: &mut Rltk) {
    ctx.cls();

    player_input(self, ctx);

    self.run_systems();
    let map = self.ecs.fetch::<Map>();
    draw_map(&map.tiles, ctx);
    let positions = self.ecs.read_storage::<Position>();
    let renderables = self.ecs.read_storage::<Renderable>();

    for (pos, render) in (&positions, &renderables).join() {
      ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
    }
  }
}

fn main() -> rltk::BError {
  use rltk::RltkBuilder;
  let context = RltkBuilder::simple80x50()
    .with_title("Roguelike Tutorial")
    .build()?;

  let mut gs = State { ecs: World::new() };
  gs.ecs.register::<Player>();
  gs.ecs.register::<Position>();
  gs.ecs.register::<Renderable>();

  let map: Map = Map::new_map_rooms_and_corridors();
  let (player_x, player_y) = map.rooms[0].center();
  gs.ecs.insert(map);

  gs.ecs
    .create_entity()
    .with(Position {
      x: player_x,
      y: player_y,
    })
    .with(Renderable {
      glyph: rltk::to_cp437('@'),
      fg: RGB::named(rltk::YELLOW),
      bg: RGB::named(rltk::BLACK),
    })
    .with(Player {})
    .build();
  rltk::main_loop(context, gs)
}
