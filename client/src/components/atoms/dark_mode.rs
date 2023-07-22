use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yewdux::prelude::*;


#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Store)]
#[store(storage = "local")]
pub struct ModeState {
    pub mode: Mode,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    #[default]
    Dark,
    Light,
}



#[function_component]
pub fn DarkMode() -> Html {
    let (store, dispatch) = use_store::<ModeState>();
    let onclick = {
        let store = store.clone();
        let dispatch = dispatch.clone();
        Callback::from(move |_: MouseEvent| match store.mode {
            Mode::Dark => {
                dispatch.reduce_mut(|store| store.mode = Mode::Light);
            }
            Mode::Light => {
                dispatch.reduce_mut(|store| store.mode = Mode::Dark);
            }
        })
    };
    let button_icon = match store.mode {
        Mode::Dark => "assets/light.png",
        Mode::Light => "assets/dark.png",
    };

    html! {
    <button {onclick}>
        <img class="icon" src={button_icon} />
    </button>
    }
}
