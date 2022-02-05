use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

#[derive(Component)]
struct Position {
  x: i32,
  y: i32,
}

#[derive(Component)]
struct Renderable {
  glyph: rltk::FontCharType,
  fg: RGB,
  bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
  type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

  fn run(&mut self, (lefty, mut pos): Self::SystemData) {
    for (_lefty, pos) in (&lefty, &mut pos).join() {
      pos.x -= 1;
      if pos.x < 0 {
        pos.x = 79
      }
    }
  }
}

struct State {
  ecs: World,
}

impl State {
  fn run_systems(&mut self) {
    let mut lw = LeftWalker {};
    lw.run_now(&self.ecs);
    self.ecs.maintain();
  }
}

impl GameState for State {
  fn tick(&mut self, ctx: &mut Rltk) {
    ctx.cls();

    self.run_systems();
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
  gs.ecs.register::<Position>();
  gs.ecs.register::<Renderable>();
  gs.ecs.register::<LeftMover>();

  gs.ecs
    .create_entity()
    .with(Position { x: 40, y: 25 })
    .with(Renderable {
      glyph: rltk::to_cp437('@'),
      fg: RGB::named(rltk::YELLOW),
      bg: RGB::named(rltk::BLACK),
    })
    .build();

  for i in 0..10 {
    gs.ecs
      .create_entity()
      .with(Position { x: i * 7, y: 20 })
      .with(Renderable {
        glyph: rltk::to_cp437('☺'),
        fg: RGB::named(rltk::RED),
        bg: RGB::named(rltk::BLACK),
      })
      .with(LeftMover {})
      .build();
  }
  rltk::main_loop(context, gs)
}
