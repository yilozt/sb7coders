use once_cell::sync::OnceCell;

use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;

use glfw::{Action, Context, Key};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub title: String,
    pub width: usize,
    pub height: usize,
    pub flags: AppFlags,
    pub glfw: glfw::Glfw,
}

#[derive(Default, Debug, Clone)]
pub struct AppFlags {
    pub fullscreen: bool,
    pub vsync: bool,
    pub cursor: bool,
    pub stereo: bool,
    pub debug: bool,
    pub robust: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: String::from("OpenGL SuperBible Example"),
            width: 800,
            height: 600,
            flags: Default::default(),
            glfw: glfw::init(glfw::FAIL_ON_ERRORS).unwrap(),
        }
    }
}

static mut INFO: OnceCell<AppConfig> = OnceCell::new();

pub trait Application {
    fn init(&self) -> AppConfig {
        AppConfig::default()
    }

    fn info(&self) -> AppConfig {
        unsafe { INFO.get().unwrap().clone() }
    }

    fn ui(&mut self, _ui: &imgui::Ui) {}

    fn run(&mut self) {
        let info = unsafe {
            INFO.set(self.init()).unwrap();
            INFO.get_mut().unwrap()
        };

        let glfw = &mut info.glfw;
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        let (mut window, events) = glfw
            .create_window(
                info.width as u32,
                info.height as u32,
                &info.title,
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window.");

        gl::load_with(|s| window.get_proc_address(s));

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let mut imgui_glfw = imgui_glfw_rs::ImguiGLFW::new(&mut imgui, &mut window);

        super::gl! {
          gl::Viewport(0, 0, info.width as i32, info.height as i32);
        }

        window.set_all_polling(true);
        window.make_current();

        self.startup();

        while !window.should_close() {
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                imgui_glfw.handle_event(&mut imgui, &event);
                self.handle_window_event(&mut window, event);
            }

            self.render(glfw.get_time());

            let ui = imgui_glfw.frame(&mut window, &mut imgui);

            self.ui(&ui);

            imgui_glfw.draw(ui, &mut window);

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
                    let info = INFO.get_mut().unwrap();
                    info.width = w as _;
                    info.height = h as _;
                }
                self.on_resize(w, h);
            },
            _ => {}
        }
    }

    fn set_vsync(&mut self, enable: bool) {
        let info = unsafe { INFO.get_mut().unwrap() };
        info.flags.vsync = enable;
        info.glfw
            .set_swap_interval(glfw::SwapInterval::Sync(if enable { 1 } else { 0 }));
    }
}
