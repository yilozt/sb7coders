
use std::{sync::Mutex};
use once_cell::sync::OnceCell;

use glfw::{Action, Context, Key};

#[derive(Debug, Clone)]
pub struct AppConfig {
  pub title:  String,
  pub width:  usize,
  pub height: usize,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self { title:  String::from("OpenGL SuperBible Example"),
           width:  800,
           height: 600, }
  }
}

static INFO: OnceCell<Mutex<AppConfig>> = OnceCell::new();

pub trait Application {
  fn init(&self) -> AppConfig {
    AppConfig::default()
  }

  fn info(&self) -> AppConfig {
    INFO.get().unwrap().lock().unwrap().clone()
  }

  fn run(&mut self) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let info = {
      INFO.set(Mutex::new(self.init())).unwrap();
      INFO.get().unwrap().lock().unwrap().clone()
    };

    let (mut window, events) =
      glfw.create_window(info.width as u32, info.height as u32, &info.title, glfw::WindowMode::Windowed)
          .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s));

    super::gl! {
      gl::Viewport(0, 0, info.width as i32, info.height as i32);
    }

    std::mem::drop(info);

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    self.startup();

    while !window.should_close() {
      glfw.poll_events();
      for (_, event) in glfw::flush_messages(&events) {
        self.handle_window_event(&mut window, event);
      }

      self.render(glfw.get_time());

      window.swap_buffers();
    }

    self.shutdown();
  }

  fn startup(&mut self) {}
  fn render(&self, current_time: f64) {
    super::gl! {
      let g = (current_time.sin() * 0.5 + 0.5) as f32;
      gl::ClearBufferfv(gl::COLOR, 0, [g, g, g, 1.0f32].as_ptr());
    }
  }
  fn shutdown(&mut self) {}

  fn on_resize(&mut self, _w: i32, _h: i32) {}

  fn on_key(&mut self, _key: Key, _press: Action) {}

  fn handle_window_event(&mut self, window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
      glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
        window.set_should_close(true)
      }
      glfw::WindowEvent::Key(key, _, action, _) => {
        self.on_key(key, action);
      }
      glfw::WindowEvent::Size(w, h) => unsafe {
        gl::Viewport(0, 0, w, h);
        {
          let mut lck = INFO.get().unwrap().lock();
          let info = lck.as_mut().unwrap();
          info.width = w as _;
          info.height = h as _;
        }
        self.on_resize(w, h);
      },
      _ => {}
    }
  }
}
