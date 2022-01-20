use glfw::{Action, Context, Key};

pub struct AppConfig {
  pub title: String,
  pub width: usize,
  pub height: usize,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      title: String::from("OpenGL SuperBible Example"),
      width: 800,
      height: 600,
    }
  }
}

pub trait Application {
  fn init(&self) -> AppConfig {
    AppConfig::default()
  }

  fn run(&mut self) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let AppConfig { title, width, height } = self.init();

    let (mut window, events) = glfw
      .create_window(width as u32, height as u32, &title, glfw::WindowMode::Windowed)
      .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s));

    unsafe {
      gl::Viewport(0, 0, width as i32, height as i32);
    }

    window.set_key_polling(true);
    window.set_size_polling(true);
    window.make_current();

    self.startup();

    while !window.should_close() {
      glfw.poll_events();
      for (_, event) in glfw::flush_messages(&events) {
        handle_window_event(&mut window, event);
      }

      self.render(glfw.get_time());

      window.swap_buffers();
    }

    self.shutdown();
  }

  fn startup(&mut self) {}
  fn render(&self, current_time: f64) {
    unsafe {
      let g = (current_time.sin() * 0.5 + 0.5) as f32;
      gl::ClearBufferfv(gl::COLOR, 0, [g, g, g, 1.0f32].as_ptr());
    }
  }
  fn shutdown(&self) {}
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
  match event {
    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
    glfw::WindowEvent::Size(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
    _ => {}
  }
}
