use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
  pub width:  u32,
  pub height: u32,
  pub title:  &'static str,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self { width:  800,
           height: 600,
           title:  "OpenGL SuperBible Example", }
  }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
  web_sys::window().unwrap()
                   .request_animation_frame(f.as_ref().unchecked_ref())
                   .expect("should register `requestAnimationFrame` OK");
}

pub trait Application {
  fn init(&self) -> AppConfig {
    AppConfig::default()
  }

  fn info(&self) -> AppConfig {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    AppConfig { width:  canvas.width(),
                height: canvas.height(),
                title:  "", }
  }

  fn run(app: Rc<RefCell<Self>>)
    where Self: 'static
  {
    let canvas = web_sys::window().unwrap()
                                  .document()
                                  .unwrap()
                                  .get_element_by_id("canvas")
                                  .unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: web_sys::WebGl2RenderingContext = canvas.get_context("webgl2")
                                                    .unwrap()
                                                    .unwrap()
                                                    .dyn_into()
                                                    .unwrap();

    let performance = web_sys::window().unwrap().performance().unwrap();

    let info = { app.borrow().init() };

    gl.viewport(0, 0, info.width as _, info.height as _);

    {
      app.borrow_mut().startup(&gl)
    };

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let _app = app.clone();
    *g.borrow_mut() =
      Some(Closure::wrap(Box::new(move || {
        _app.borrow_mut().render(&gl, performance.now() / 1000.0);

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
      }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    app.borrow_mut().shutdown();
  }

  fn startup(&mut self, gl: &web_sys::WebGl2RenderingContext)
    where Self: 'static
  {
  }

  fn render(&self, gl: &web_sys::WebGl2RenderingContext, current_time: f64) {
    let g = (current_time.sin() * 0.5 + 0.5) as f32;
    gl.clear_color(g, g, 0.0, 1.0);
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
  }
  fn shutdown(&mut self) {}
}
