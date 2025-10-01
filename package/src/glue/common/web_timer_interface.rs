use crate::SendWrapper;
use js_sys::{Function, Object, Reflect};
use std::sync::OnceLock;
use wasm_bindgen::prelude::{JsCast, JsValue};
use web_sys::{Window, WorkerGlobalScope};

pub enum WebTimerInterface {
  NodeJs {
    clear_interval: Function,
    set_interval: Function,
    set_timeout: Function,
  },
  Window(Window),
  WorkerGlobalScope(WorkerGlobalScope),
}

impl WebTimerInterface {
  pub fn clear_interval(&self, interval_id: i32) {
    match &self {
      WebTimerInterface::NodeJs { clear_interval, .. } => {
        let _ = clear_interval
          .call1(&js_sys::global(), &JsValue::from_f64(interval_id as f64))
          .expect("failed to call global js function `clearInterval`");
      }
      WebTimerInterface::Window(window) => {
        window.clear_interval_with_handle(interval_id)
      }
      WebTimerInterface::WorkerGlobalScope(scope) => {
        scope.clear_interval_with_handle(interval_id)
      }
    }
  }

  pub fn set_interval(
    &self,
    callback: &Function,
    delay_milliseconds: i32,
  ) -> Result<i32, JsValue> {
    match self {
      WebTimerInterface::NodeJs { set_interval, .. } => set_interval
        .call2(
          &JsValue::UNDEFINED,
          callback,
          &JsValue::from_f64(delay_milliseconds as f64),
        )
        .map(|timeout| get_node_js_timeout_id(&timeout)),
      WebTimerInterface::Window(window) => window
        .set_interval_with_callback_and_timeout_and_arguments_0(
          callback,
          delay_milliseconds,
        ),
      WebTimerInterface::WorkerGlobalScope(scope) => scope
        .set_interval_with_callback_and_timeout_and_arguments_0(
          callback,
          delay_milliseconds,
        ),
    }
  }

  pub fn set_timeout(
    &self,
    callback: &Function,
    delay_milliseconds: i32,
  ) -> Result<i32, JsValue> {
    match self {
      WebTimerInterface::NodeJs { set_timeout, .. } => set_timeout
        .call2(
          &JsValue::UNDEFINED,
          callback,
          &JsValue::from_f64(delay_milliseconds as f64),
        )
        .map(|timeout| get_node_js_timeout_id(&timeout)),
      WebTimerInterface::Window(window) => window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
          callback,
          delay_milliseconds,
        ),
      WebTimerInterface::WorkerGlobalScope(scope) => scope
        .set_timeout_with_callback_and_timeout_and_arguments_0(
          callback,
          delay_milliseconds,
        ),
    }
  }
}

fn get_js_function_from_object(
  object: &Object,
  name: &str,
) -> Result<Function, JsValue> {
  Reflect::get(object, &JsValue::from_str(name)).and_then(|value| {
    value
      .dyn_into::<Function>()
      .map_err(|_| format!("failed to get js function `{name}`").into())
  })
}

fn get_web_timer_interface() -> Result<WebTimerInterface, JsValue> {
  let global = js_sys::global();

  if js_sys::eval(
        "typeof WorkerGlobalScope !== 'undefined' && self instanceof WorkerGlobalScope",
    )?
    .as_bool()
    .unwrap_or(false)
    {
        Ok(global
            .dyn_into::<WorkerGlobalScope>()
            .map(WebTimerInterface::WorkerGlobalScope)?)
    } else if js_sys::eval("typeof Window !== 'undefined' && self instanceof Window")?
        .as_bool()
        .unwrap_or(false)
    {
        Ok(global.dyn_into::<Window>().map(WebTimerInterface::Window)?)
    } else if is_node_js_env() {
        Ok(WebTimerInterface::NodeJs {
            clear_interval: get_js_function_from_object(&global, "clearInterval")?,
            set_interval: get_js_function_from_object(&global, "setInterval")?,
            set_timeout: get_js_function_from_object(&global, "setTimeout")?,
        })
    } else {
        Err("failed to get web timer interface".into())
    }
}

pub fn clear_interval(interval_id: i32) {
  web_timer_interface().clear_interval(interval_id)
}

pub fn set_interval(callback: &Function, delay_milliseconds: i32) -> i32 {
  web_timer_interface()
    .set_interval(callback, delay_milliseconds)
    .expect("failed to call setInterval in web environment")
}

pub fn set_timeout(callback: &Function, delay_milliseconds: i32) -> i32 {
  web_timer_interface()
    .set_timeout(callback, delay_milliseconds)
    .expect("failed to call setTimeout in web environment")
}

pub fn web_timer_interface() -> &'static SendWrapper<WebTimerInterface> {
  static INSTANCE: OnceLock<SendWrapper<WebTimerInterface>> = OnceLock::new();
  INSTANCE.get_or_init(|| SendWrapper::new(get_web_timer_interface().unwrap()))
}

/// Get the timeout ID from a NodeJS Timeout object
/// Reference: https://nodejs.org/api/timers.html#class-timeout
fn get_node_js_timeout_id(timeout: &JsValue) -> i32 {
  js_sys::Reflect::get(timeout, &js_sys::Symbol::to_primitive())
    .and_then(|primitive_fn_obj| primitive_fn_obj.dyn_into::<Function>())
    .and_then(|primitive_fn| primitive_fn.call0(timeout))
    .ok()
    .and_then(|primitive_value| primitive_value.as_f64())
    .map(|primitive_f64| primitive_f64 as i32)
    .expect("failed to get timeout id from NodeJS timeout object")
}

fn is_node_js_env() -> bool {
  let global = js_sys::global();

  Reflect::get(&global, &JsValue::from_str("process"))
    .ok()
    .filter(|process| process.is_object())
    .and_then(|process| {
      Reflect::get(&process, &JsValue::from_str("versions")).ok()
    })
    .filter(|versions| versions.is_object())
    .and_then(|versions| {
      Reflect::has(&versions, &JsValue::from_str("node")).ok()
    })
    .unwrap_or(false)
}
