use std::rc::Rc;
use std::cell::RefCell;

// if app are running, its address in this list

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

  fn close_app(id: Option<String>) {
    let id = id.unwrap_or("app".into());
    let app = web_sys::window().unwrap().document().unwrap().get_element_by_id(&id).unwrap();
    app.dyn_into::<web_sys::HtmlElement>().unwrap().dataset().set("closed", "").unwrap();
  }

  fn info(&self) -> AppConfig {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    AppConfig { width:  canvas.width(),
                height: canvas.height(),
                title:  "", }
  }

  fn run(mut self: Box<Self>, width: Option<u32>, height: Option<u32>, id: Option<String>)
  {
    let (gl, app_elem) = {
      let mut info = self.init();

      info.width = width.unwrap_or(info.width);
      info.height = height.unwrap_or(info.height);
      let id = id.unwrap_or("app".into());

      let app_elem: web_sys::HtmlElement = web_sys::window().unwrap()
                                    .document()
                                    .unwrap()
                                    .get_element_by_id(&id)
                                    .unwrap().dyn_into().unwrap();

      if app_elem.query_selector("#canvas").unwrap().is_none() {
        app_elem.set_inner_html(r#"
          <canvas id="canvas"></canvas>  
          <details>
            <summary id="title" class="apptitle">Hello, Rust! (Loading.....)</summary>
            <div id="ui"></div>
          </details>
        "#);
      }

      let canvas: web_sys::HtmlCanvasElement = app_elem.query_selector("#canvas").unwrap()
                                                  .unwrap().dyn_into().unwrap();

      canvas.set_width(info.width);
      canvas.set_height(info.height);

      let ui = app_elem.query_selector("#ui").unwrap().unwrap();
      if id == "app" {
        ui.set_inner_html("");
      }

      let gl: web_sys::WebGl2RenderingContext = canvas.get_context("webgl2")
                                                      .unwrap()
                                                      .unwrap()
                                                      .dyn_into()
                                                      .unwrap();


      if let Some(h1) = app_elem.query_selector("#title").unwrap()
      {
        h1.set_inner_html(info.title);
      }

      gl.viewport(0, 0, info.width as _, info.height as _);

      self.startup(&gl);

      let gl = Rc::new(gl);

      self.ui(gl.clone(), &ui);

      (gl, app_elem)
    };

    let performance = web_sys::window().unwrap().performance().unwrap();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    // let _app = app.clone();
    let _gl = gl.clone();

    let render = move || {
      if app_elem.dataset().get("closed").is_some() {
        self.shutdown(&gl);
        web_sys::console::log_1(&"closing.....".into());
        app_elem.dataset().delete("closed");
        return;
      }

      self.render(&_gl.clone(), performance.now() / 1000.0);

      // Schedule ourself for another requestAnimationFrame callback.
      request_animation_frame(f.borrow().as_ref().unwrap());
    };
    
    let closure = Closure::wrap(Box::new(render) as Box<dyn FnMut()>);
    *g.borrow_mut() = Some(closure);

    request_animation_frame(g.borrow().as_ref().unwrap());
  }

  fn startup(&mut self, _gl: &web_sys::WebGl2RenderingContext)
  {
  }

  fn ui(&mut self, _gl: Rc<web_sys::WebGl2RenderingContext>, _ui: &web_sys::Element)
  {
  }

  fn render(&self, gl: &web_sys::WebGl2RenderingContext, current_time: f64) {
    let g = (current_time.sin() * 0.5 + 0.5) as f32;
    gl.clear_color(g, g, g, 1.0);
    gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
  }
  fn shutdown(&mut self, _gl: &web_sys::WebGl2RenderingContext) {}
}
