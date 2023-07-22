use gloo::utils::document;
use serde::{Deserialize, Serialize};
use stylist::css;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::components::atoms::dark_mode::{Mode, ModeState};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Store)]
#[store(storage = "session")]
pub struct Overlay {
    pub state: State,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum State {
    #[default]
    Close,
    Open,
}

#[function_component]
pub fn Info() -> Html {
    let (theme, _) = use_store::<ModeState>();

    let (button_icon, overlay_background_color) = match theme.mode {
        Mode::Dark => ("assets/info-dark.png", "gray"),
        Mode::Light => ("assets/info-light.png", "white"),
    };
    let (overlay_state, dispatch) = use_store::<Overlay>();
    let first_load = use_state(|| true);
    let onclick = {
        let dispatch = dispatch.clone();
        let first_load = first_load.clone();
        Callback::from(move |_| {
            if *first_load {
                let container = document()
                    .get_element_by_id("overlay-container")
                    .expect("no element cantainer found");

                let overlay = document()
                    .create_element("div")
                    .expect("no elemnet overlay");
                overlay.set_class_name("overlay");
                //let text = document(). .create_text_node("onahutonahunatohunt");
                let text_div = document().get_element_by_id("text").unwrap();
                overlay.append_child(&text_div).unwrap();
                //overlay.append_child(&text).unwrap();
                container.append_child(&overlay).unwrap();
                let body = document().body().unwrap();
                body.append_child(&container).unwrap();

                first_load.set(false);
            }
            dispatch.reduce_mut(|overlay| overlay.state = State::Open);
        })
    };

    let close = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.reduce_mut(|overlay| overlay.state = State::Close))
    };

    let display = match overlay_state.state {
        State::Close => "none",
        State::Open => "block",
    };

    let modal = css!(
        ".overlay{
            display: ${display};
            position: absolute;
            z-index: 301;
            top: 10%;
            left: 0;
            right: 0;
            margin: auto auto 40px;
            width: 580px;
            padding: 20px;
            border-radius: 5px;
            background-color: ${overlay_background_color};
            box-sizing: border-box;
        }
        ",
        overlay_background_color = overlay_background_color,
        display = display
    );

    html! {
        <>
            <button {onclick}>
                <img class="icon" src={button_icon} />
            </button>
            <div class="modal-overlay" onclick={close}>
            </div>
            <div class={modal}  id="overlay-container">

               <div id="text">
                if !*first_load {
                <h3>{"What is it?"}</h3>
                <p>
                    {"ShareDrop is a free,web app that allows you to easily and securely share
                     files directly between devices without uploading them to any server first"}
                </p>
                }
               </div>
            </div>
        </>
    }
}
