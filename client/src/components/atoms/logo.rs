use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::components::atoms::{
    animation::{background_animation, store_theme},
    dark_mode::{Mode, ModeState},
};

#[function_component(Logo)]
pub fn logo() -> Html {
    let (store, _) = use_store::<ModeState>();

    let (logo, theme) = match store.mode {
        Mode::Dark => ("assets/logo-dark.png", "dark"),
        Mode::Light => ("assets/logo-light.png", "light"),
    };

    use_effect(|| {
        store_theme(theme);
        background_animation();
    });

    html! {
        <img class="logo" src={logo} alt ="logo" />
    }
}
