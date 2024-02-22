use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy_ecs::world::World;
use egui::{epaint::Shadow, Context, RawInput, Rounding, Visuals};
use foxy_renderer::renderer::mesh::StaticMesh;
use foxy_utils::time::{EngineTime, Time};
use winit::{event::WindowEvent, window::Window};

use super::input::Input;

#[derive(Clone)]
pub struct Foxy(Arc<RwLock<State>>);

impl Foxy {
  pub fn new(state: State) -> Self {
    Self(Arc::new(RwLock::new(state)))
  }

  pub fn as_ref(&self) -> RwLockReadGuard<State> {
    self.0.read().expect("unable to obtain read lock")
  }

  pub fn as_mut(&self) -> RwLockWriteGuard<State> {
    self.0.write().expect("unable to obtain write lock")
  }
}

pub struct State {
  pub(crate) engine_time: EngineTime,
  pub(crate) window: Arc<Window>,
  pub(crate) egui_context: Context,
  pub(crate) egui_state: egui_winit::State,
  pub(crate) input: Input,
  // pub(crate) meshes: Vec<StaticMesh>,
  pub(crate) world: World, 
}

impl State {
  pub fn new(engine_time: EngineTime, window: Arc<Window>) -> Self {
    let egui_context = Context::default();

    let id = egui_context.viewport_id();

    const BORDER_RADIUS: f32 = 6.0;

    let visuals = Visuals {
      window_rounding: Rounding::same(BORDER_RADIUS),
      menu_rounding: Rounding::same(BORDER_RADIUS),
      window_shadow: Shadow::NONE,
      ..Default::default()
    };

    egui_context.set_visuals(visuals);

    let egui_state = egui_winit::State::new(egui_context.clone(), id, &window, None, None);
    
    let world = World::new();

    Self {
      engine_time,
      window,
      egui_context,
      egui_state,
      input: Input::new(),
      world,
    }
  }

  pub fn time(&self) -> Time {
    self.engine_time.time()
  }

  pub fn window(&self) -> &Arc<Window> {
    &self.window
  }

  pub fn input(&self) -> &Input {
    &self.input
  }

  // // TEMPORARY UNTIL ECS IS IMPLEMENTED
  // pub fn submit_mesh(&mut self, mesh: StaticMesh) {
  //   self.meshes.push(mesh);
  // }
  
  pub fn world(&self) -> &World {
    &self.world
  }

  pub fn world_mut(&mut self) -> &mut World {
    &mut self.world
  }

  pub(crate) fn handle_input(&mut self, event: &WindowEvent) -> bool {
    let response = self.egui_state.on_window_event(&self.window, event);

    if response.repaint {
      self.egui_context.request_repaint();
    }

    response.consumed
  }

  pub(crate) fn take_egui_input(&mut self) -> RawInput {
    self.egui_state.take_egui_input(&self.window)
  }
}
