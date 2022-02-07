use rltk::{GameState, Point, Rltk};
use specs::prelude::*;

mod gui;

mod inventory_system;
pub use inventory_system::*;
mod spawner;
pub use spawner::*;
mod components;
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
  ShowInventory,
  ShowDropItem,
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
    let mut pickup = ItemCollectionSystem {};
    pickup.run_now(&self.ecs);
    let mut potions = PotionUseSystem {};
    potions.run_now(&self.ecs);
    let mut drop_items = ItemDropSystem {};
    drop_items.run_now(&self.ecs);
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
        self.ecs.maintain();
        newrunstate = RunState::AwaitingInput;
      }
      RunState::AwaitingInput => {
        newrunstate = player_input(self, ctx);
      }
      RunState::PlayerTurn => {
        self.run_systems();
        self.ecs.maintain();
        newrunstate = RunState::MonsterTurn;
      }
      RunState::MonsterTurn => {
        self.run_systems();
        self.ecs.maintain();
        newrunstate = RunState::AwaitingInput;
      }
      RunState::ShowInventory => {
        let result = gui::show_inventory(self, ctx);
        match result.0 {
          gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
          gui::ItemMenuResult::NoResponse => {}
          gui::ItemMenuResult::Selected => {
            let item_entity = result.1.unwrap();
            let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
            intent
              .insert(
                *self.ecs.fetch::<Entity>(),
                WantsToDrinkPotion {
                  potion: item_entity,
                },
              )
              .expect("Unable to insert intent");
            newrunstate = RunState::PlayerTurn;
          }
        }
      }
      RunState::ShowDropItem => {
        let result = gui::drop_item_menu(self, ctx);
        match result.0 {
          gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
          gui::ItemMenuResult::NoResponse => {}
          gui::ItemMenuResult::Selected => {
            let item_entity = result.1.unwrap();
            let mut intent = self.ecs.write_storage::<WantsToDropItem>();
            intent
              .insert(
                *self.ecs.fetch::<Entity>(),
                WantsToDropItem { item: item_entity },
              )
              .expect("Unable to insert intent");
            newrunstate = RunState::PlayerTurn;
          }
        }
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
    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render) in data.iter() {
      let idx = Map::xy_idx(pos.x, pos.y);
      if map.visible_tiles[idx] {
        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
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
  gs.ecs.register::<Item>();
  gs.ecs.register::<Potion>();
  gs.ecs.register::<WantsToPickupItem>();
  gs.ecs.register::<WantsToDrinkPotion>();
  gs.ecs.register::<InBackpack>();
  gs.ecs.register::<WantsToDropItem>();
  gs.ecs.insert(RunState::PreRun);
  gs.ecs.insert(rltk::RandomNumberGenerator::new());

  let map: Map = Map::new_map_rooms_and_corridors();
  let (player_x, player_y) = map.rooms[0].center();

  for room in map.rooms.iter().skip(1) {
    spawner::spawn_room(&mut gs.ecs, room);
  }

  gs.ecs.insert(map);

  gs.ecs.insert(Point::new(player_x, player_y));
  let player_entity = player(&mut gs.ecs, player_x, player_y);
  gs.ecs.insert(player_entity);

  gs.ecs.insert(gamelog::GameLog {
    entries: vec!["Welcome to Rusty Roguelike".to_string()],
  });

  rltk::main_loop(context, gs)
}
