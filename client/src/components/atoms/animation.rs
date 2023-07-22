use gloo::utils::document;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Event, UiEvent};

#[wasm_bindgen]
pub fn background_animation() {
    let c = document().get_element_by_id("canvas").expect("no canv");

    let c: web_sys::HtmlCanvasElement = c
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let ctx = c
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let ctx_clone = ctx.clone();
    let c_clone = c.clone();
    let init = Closure::<dyn FnMut(_)>::new(move |_: UiEvent| {
        let w = window().unwrap().inner_width().unwrap().as_f64().unwrap();
        let h = window().unwrap().inner_height().unwrap().as_f64().unwrap();

        c_clone.set_width(w as u32);
        c_clone.set_height(h as u32);
        let mut offset = if h > 380. { 117. } else { 65. };
        offset = if h > 800. { 115. } else { offset };
        let x0 = w / 2.;
        let y0 = h - offset;
        let dw = w.max(h).max(1000.) / 13.;
        let step = Rc::new(Cell::new(1));

        let ctx_clone_ = ctx_clone.clone();
        let draw_circle = move |radius| {
            ctx_clone_.begin_path();
            ctx_clone_
                .arc(x0, y0, radius, 0.0, std::f64::consts::PI * 2.0)
                .unwrap();
            ctx_clone_.stroke();
            let storage = window().unwrap().session_storage().unwrap().unwrap();

            let color = match &storage.get_item("theme").unwrap().unwrap()[..] {
                "dark" => "rgba(255,255,255,0.2)",
                _ => "rgba(0,0,0,0.2)",
            };
            ctx_clone_.set_stroke_style(&color.to_string().into());
            ctx_clone_.set_line_width(2.);
        };

        let draw_circle_clone = draw_circle.clone();
        let ctx_clone_ = ctx_clone.clone();
        let step_clone = step.clone();
        let draw_cicles = move || {
            ctx_clone_.clear_rect(0., 0., w, h);
            for i in 0..8 {
                draw_circle_clone((dw * i as f64 + step_clone.get() as f64 % dw) as f64);
            }
            step_clone.set(step_clone.get() + 1);
        };

        animate(draw_cicles);
    });

    /*window()
    .unwrap()
    .set_onload(Some(init.as_ref().unchecked_ref()));*/
    //  window().unwrap().add_event_listener_with_callback("load", init.as_ref().unchecked_ref()).unwrap();
    // c.add_event_listener_with_callback("load", init.as_ref().unchecked_ref()).unwrap();
    window()
        .unwrap()
        .set_onresize(Some(init.as_ref().unchecked_ref()));
    window()
        .unwrap()
        .dispatch_event(&Event::new("resize").unwrap())
        .unwrap();
    init.forget();
}

fn animate<T: Fn() -> () + Clone + 'static>(draw_circles: T) {
    //let draw_circles_clone = draw_circles.clone();
    let closure = Closure::<dyn FnMut()>::new(move || {
        draw_circles();
        animate(draw_circles.clone());
    });
    window()
        .unwrap()
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .expect("Failed to request animation frame");

    closure.forget();
}

#[wasm_bindgen]
pub fn store_theme(theme: &str) {
    let storage = window().unwrap().session_storage().unwrap().unwrap();
    storage.set_item("theme", theme).unwrap();
}
