use once_cell::sync::Lazy;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashSet};

// if app are running, its address in this list
static mut APP_RUNNING: Lazy<HashSet<usize>> = Lazy::new(|| HashSet::new());

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

pub trait Application: 'static {
  fn init(&self) -> AppConfig {
    AppConfig::default()
  }

  fn close_app(&self, ptr: usize) {
    web_sys::console::log_1(&format!("closing ...{}", ptr).into());
    unsafe {
      APP_RUNNING.remove(&ptr.into());
    }
  }

  fn should_close(&self, ptr: usize) -> bool {
    unsafe { APP_RUNNING.get(&ptr).is_none() }
  }

  fn info(&self) -> AppConfig {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    AppConfig { width:  canvas.width(),
                height: canvas.height(),
                title:  "", }
  }

  fn run(&'static mut self, ptr: usize, width: Option<u32>, height: Option<u32>, id: Option<String>)
    where Self: 'static
  {
    let mut info = self.init();
    info.width = width.unwrap_or(info.width);
    info.height = height.unwrap_or(info.height);
    let id = id.unwrap_or("app".into());

    let app: web_sys::Element = web_sys::window().unwrap()
                                  .document()
                                  .unwrap()
                                  .get_element_by_id(&id)
                                  .unwrap().dyn_into().unwrap();

    if app.query_selector("#canvas").unwrap().is_none() {
      app.set_inner_html(r#"
        <canvas id="canvas"></canvas>  
        <details>
          <summary id="title">Hello, Rust! (Loading.....)</summary>
          <div id="ui"></div>
        </details>
      "#);
    }

    let canvas: web_sys::HtmlCanvasElement = app.query_selector("#canvas").unwrap()
                                                .unwrap().dyn_into().unwrap();

    canvas.set_width(info.width);
    canvas.set_height(info.height);

    let ui = app.query_selector("#ui").unwrap().unwrap();
    if id == "app" {
      ui.set_inner_html("");
    }

    let gl: web_sys::WebGl2RenderingContext = canvas.get_context("webgl2")
                                                    .unwrap()
                                                    .unwrap()
                                                    .dyn_into()
                                                    .unwrap();

    let performance = web_sys::window().unwrap().performance().unwrap();


    if let Some(h1) = web_sys::window().unwrap()
                                       .document()
                                       .unwrap()
                                       .get_element_by_id("title")
    {
      h1.set_inner_html(info.title);
    }

    gl.viewport(0, 0, info.width as _, info.height as _);

    self.startup(&gl);
    self.ui(&gl, &ui);

    // register running app
    unsafe {
      APP_RUNNING.insert(ptr);
    }

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let app = Rc::new(RefCell::new(self));

    // let _app = app.clone();
    let _gl = gl.clone();

    let render = move || {
      if app.borrow().should_close(ptr) {
        app.borrow_mut().shutdown(&gl);
        return;
      }

      app.borrow_mut()
         .render(&_gl.clone(), performance.now() / 1000.0);

      // Schedule ourself for another requestAnimationFrame callback.
      request_animation_frame(f.borrow().as_ref().unwrap());
    };
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(render) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
  }

  fn startup(&mut self, _gl: &web_sys::WebGl2RenderingContext)
    where Self: 'static
  {
  }

  fn ui(&mut self, _gl: &web_sys::WebGl2RenderingContext, _ui: &web_sys::Element)
  {
  }

  fn render(&self, gl: &web_sys::WebGl2RenderingContext, current_time: f64) {
    let g = (current_time.sin() * 0.5 + 0.5) as f32;
    gl.clear_color(g, g, g, 1.0);
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
  }
  fn shutdown(&mut self, _gl: &web_sys::WebGl2RenderingContext) {}
}
