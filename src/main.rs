use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;

mod components;
mod gui;
pub use components::*;
mod gamelog;
pub use gamelog::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
pub use visibility_system::VisibilitySystem;
mod monster_ai_system;
pub use monster_ai_system::MonsterAI;
mod map_indexing_system;
pub use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
pub use melee_combat_system::MeleeCombatSystem;
mod damage_system;
pub use damage_system::DamageSystem;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
  AwaitingInput,
  PreRun,
  PlayerTurn,
  MonsterTurn,
}

pub struct State {
  pub ecs: World,
}

impl State {
  fn run_systems(&mut self) {
    let mut vis = VisibilitySystem {};
    vis.run_now(&self.ecs);
    let mut mob = MonsterAI {};
    mob.run_now(&self.ecs);
    let mut mapindex = MapIndexingSystem {};
    mapindex.run_now(&self.ecs);
    let mut meleecombat = MeleeCombatSystem {};
    meleecombat.run_now(&self.ecs);
    let mut damage = DamageSystem {};
    damage.run_now(&self.ecs);
    self.ecs.maintain();
  }
}

impl GameState for State {
  fn tick(&mut self, ctx: &mut Rltk) {
    ctx.cls();

    let mut newrunstate;
    {
      let runstate = self.ecs.fetch::<RunState>();
      newrunstate = *runstate;
    }

    match newrunstate {
      RunState::PreRun => {
        self.run_systems();
        newrunstate = RunState::AwaitingInput;
      }
      RunState::AwaitingInput => {
        newrunstate = player_input(self, ctx);
      }
      RunState::PlayerTurn => {
        self.run_systems();
        newrunstate = RunState::MonsterTurn;
      }
      RunState::MonsterTurn => {
        self.run_systems();
        newrunstate = RunState::AwaitingInput;
      }
    }

    {
      let mut runwriter = self.ecs.write_resource::<RunState>();
      *runwriter = newrunstate;
    }
    damage_system::delete_the_dead(&mut self.ecs);

    draw_map(&self.ecs, ctx);
    let positions = self.ecs.read_storage::<Position>();
    let renderables = self.ecs.read_storage::<Renderable>();
    let map = self.ecs.fetch::<Map>();

    for (pos, render) in (&positions, &renderables).join() {
      let idx = Map::xy_idx(pos.x, pos.y);
      if map.visible_tiles[idx] {
        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
      }
    }
    gui::draw_ui(&self.ecs, ctx);
  }
}

fn main() -> rltk::BError {
  use rltk::RltkBuilder;
  let mut context = RltkBuilder::simple80x50()
    .with_title("Roguelike Tutorial")
    .build()?;
  context.with_post_scanlines(true);

  let mut gs = State { ecs: World::new() };
  gs.ecs.register::<Player>();
  gs.ecs.register::<Position>();
  gs.ecs.register::<Renderable>();
  gs.ecs.register::<Viewshed>();
  gs.ecs.register::<Monster>();
  gs.ecs.register::<Name>();
  gs.ecs.register::<BlocksTile>();
  gs.ecs.register::<CombatStats>();
  gs.ecs.register::<SufferDamage>();
  gs.ecs.register::<WantsToMelee>();
  gs.ecs.insert(RunState::PreRun);

  let map: Map = Map::new_map_rooms_and_corridors();
  let (player_x, player_y) = map.rooms[0].center();

  let mut rng = rltk::RandomNumberGenerator::new();
  for (i, room) in map.rooms.iter().skip(1).enumerate() {
    let (x, y) = room.center();
    let roll = rng.roll_dice(1, 2);
    let (glyph, name) = match roll {
      1 => (rltk::to_cp437('g'), "Goblin".to_string()),
      _ => (rltk::to_cp437('o'), "Orc".to_string()),
    };

    gs.ecs
      .create_entity()
      .with(Position { x, y })
      .with(Renderable {
        glyph,
        fg: RGB::named(rltk::RED),
        bg: RGB::named(rltk::BLACK),
      })
      .with(Viewshed {
        visible_tiles: Vec::new(),
        range: 8,
        dirty: true,
      })
      .with(Monster {})
      .with(Name {
        name: format!("{}, #{}", &name, i),
      })
      .with(BlocksTile {})
      .with(CombatStats {
        max_hp: 16,
        hp: 16,
        defense: 1,
        power: 4,
      })
      .build();
  }

  gs.ecs.insert(map);

  gs.ecs.insert(Point::new(player_x, player_y));
  let player_entity = gs
    .ecs
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
    .with(Name {
      name: "Player".to_string(),
    })
    .with(Viewshed {
      visible_tiles: Vec::new(),
      range: 8,
      dirty: true,
    })
    .with(CombatStats {
      max_hp: 30,
      hp: 30,
      defense: 2,
      power: 5,
    })
    .build();
  gs.ecs.insert(player_entity);

  gs.ecs.insert(gamelog::GameLog {
    entries: vec!["Welcome to Rusty Roguelike".to_string()],
  });

  rltk::main_loop(context, gs)
}
