use glfw::{Action, Context, Key};

pub trait Application {
  fn run(&mut self) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
      .create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
      .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s));

    unsafe {
      gl::Viewport(0, 0, 300, 300);
    }

    window.set_key_polling(true);
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
    glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl::Viewport(0, 0, w, h) },
    _ => {}
  }
}
