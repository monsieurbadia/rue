mod utils;

use std::f64;
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Math;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, Window, console};

// when the `wee_alloc` feature is enabled,
// use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn greet(whoami: &str) {
  let greetings: String = format!("hello {} from rust!", whoami);

  console::log_1(&JsValue::from(greetings));
}

/// id of the canvas
const CANVAS_ID: &str = "renderer";

/// width of the canvas
const CANVAS_WIDTH: f64 = 200.0;

/// height of the canvas
const CANVAS_HEIGHT: f64 = 200.0;

/// x-center of the canvas
const CANVAS_CENTER_X: f64 = CANVAS_WIDTH / 2.0;

/// y-center of the canvas
const CANVAS_CENTER_Y: f64 = CANVAS_HEIGHT / 2.0;

/// x-position of the canvas
const CANVAS_POS_X: f64 = 0.0;

/// y-position of the canvas
const CANVAS_POS_Y: f64 = 0.0;

/// context of the canvas
const CONTEXT_2D_TYPE: &str = "2d";

/// radius of the firefly
const FIREFLY_RADIUS: f64 = 2.0;

/// maximum number of firefly
const FIREFLY_MAX: u32 = 50;

/// start angle of the firefly
const FIREFLY_ANGLE_START: f64 = 0.0;

/// end angle of the firefly
const FIREFLY_ANGLE_END: f64 = f64::consts::PI * 2.0;

/// angle offset of the firefly angle
const ANGLE_OFFSET: f64 = 2.0;

/// radius offset of the firefly
const RADIUS_OFFSET: f64 = 50.0;

/// offset unit of the rgb color
const RGB_OFFSET: f64 = 255.0;

/// rgb color instance
#[derive(Copy, Clone)]
struct Rgb {
  r: f64,
  g: f64,
  b: f64,
}

impl Rgb {
  #[inline]
  fn new(r: f64, g: f64, b: f64) -> Self {
    Self { r, g, b }
  }
}

/// a vector in 2D space (x, y)
#[derive(Clone, Copy)]
struct Vector {
  x: f64,
  y: f64,
}

impl Vector {
  #[inline]
  fn new(x: f64, y: f64) -> Self {
    Self { x, y }
  }
}

/// a firefly instance
#[derive(Clone, Copy)]
struct Firefly {
  radius: f64,
  angle: Vector,
  speed: Vector,
  color: Rgb,
}

impl Firefly {
  #[inline]
  fn new(angle: Vector, radius: f64, speed: Vector, color: Rgb) -> Self {
    Self {
      angle,
      radius,
      speed,
      color,
    }
  }

  fn update(&mut self, ctx: &web_sys::CanvasRenderingContext2d) {
    // get positions
    let x = Math::cos(self.angle.x) * self.radius;
    let y = Math::sin(self.angle.y) * self.radius;

    // increase the angle by the speed
    self.angle.x += self.speed.x;
    self.angle.y += self.speed.y;

    // draw the firefly
    ctx.begin_path();

    // colorize the firefly
    ctx.set_fill_style(&JsValue::from_str(&format!(
      "rgb({}, {}, {})",
      self.color.r as u32,
      self.color.g as u32,
      self.color.b as u32
    )));

    ctx.arc(
      CANVAS_CENTER_X + x,
      CANVAS_CENTER_Y + y,
      FIREFLY_RADIUS,
      FIREFLY_ANGLE_START,
      FIREFLY_ANGLE_END,
    ).unwrap();

    ctx.fill();
  }
}

// javascript window object
fn window() -> Window {
  web_sys::window().expect("no global `window` exists")
}

// javascript document object
fn document() -> Document {
  window()
    .document()
    .expect("should have a document on window")
}

// javascript request animation frame function
fn raf(f: &Closure<dyn FnMut()>) {
  window()
    .request_animation_frame(f.as_ref().unchecked_ref())
    .expect("should register `requestAnimationFrame` OK");
}

/// the render function that will be use on the `vue` side
/// using the `wasm_bindgen` attributes to compile to wasm
#[wasm_bindgen]
pub fn render() {
  // set the actual size in memory (extra pixel for antialiasing)
  let scale = window().device_pixel_ratio();

  // get the canvas element
  let canvas = document().get_element_by_id(CANVAS_ID).unwrap();

  // turn the canvas elment into right canvas element type
  let canvas = canvas
    .dyn_into::<HtmlCanvasElement>()
    .map_err(|_| ())
    .unwrap();

  // set the normalized width
  let width = scale * CANVAS_WIDTH;

  // set the normalized height
  let height = scale * CANVAS_HEIGHT;

  // set the actual canvas size 
  canvas.set_width(width as u32);
  canvas.set_height(height as u32);

  // get the 2D context
  let context = canvas
    .get_context(CONTEXT_2D_TYPE)
    .unwrap()
    .unwrap()
    .dyn_into::<web_sys::CanvasRenderingContext2d>()
    .unwrap();

  // normalize coordinate system to use css pixels
  context.scale(scale as f64, scale as f64).unwrap();

  // create the fireflies buffer
  let mut fireflies: Vec<Firefly> = vec![];

  for _ in 0..FIREFLY_MAX {
    // get the firefly angle
    let angle = Vector::new(
      Math::random() * f64::consts::PI * ANGLE_OFFSET,
      Math::random() * f64::consts::PI * ANGLE_OFFSET,
    );

    // get the firefly radius
    let radius = RADIUS_OFFSET + Math::random() * RADIUS_OFFSET;

    // get the firefly speed
    let speed = Vector::new(
      Math::random() * 0.1 - 0.05,
      Math::random() * 0.1 - 0.05,
    ); // velocity & gravity

    // get the firefly color
    let color = Rgb::new(
      Math::random() * RGB_OFFSET,
      Math::random() * RGB_OFFSET,
      Math::random() * RGB_OFFSET,
    );

    // create the firefly
    let c = Firefly::new(angle, radius, speed, color);

    // save the firefly
    fireflies.push(c);
  }

  // inside the closure we've got a persistent `Rc` reference,
  // which we use for all future iterations of the loop
  let f = Rc::new(RefCell::new(None));
  let g = f.clone();

  *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
    // clear the canvas
    context.clear_rect(CANVAS_POS_X, CANVAS_POS_Y, width, height);

    // update the fireflies
    fireflies.iter_mut().for_each(|c| c.update(&context));

    // schedule ourself for another requestAnimationFrame callback
    raf(f.borrow().as_ref().unwrap());
  }) as Box<dyn FnMut()>));

  // then render animation loop
  raf(g.borrow().as_ref().unwrap());
}
