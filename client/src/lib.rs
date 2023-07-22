mod components;
mod connection_manager;
mod webrtc_manager;

use yew::prelude::*;

use crate::components::{atoms::global_style::GlobalCss, organisms::{header::Header, center::Center, footer::Footer}};


#[function_component(App)]
pub fn app() -> Html {  

    html!(
        <>
        <GlobalCss />
        <Header />
        <Center />
        <Footer />
        <canvas id="canvas"></canvas>
        </>
    )
}
