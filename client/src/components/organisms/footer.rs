use stylist::css;
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::components::atoms::{
    dark_mode::{Mode, ModeState},
    link::Link,
    logo::Logo,
    this_peer_state::ThisPeer,
};

#[function_component(Footer)]
pub fn footer() -> Html {
    let (store, _) = use_store::<ThisPeer>();
    let current_peer_name = store.name.clone();

    let (store, _) = use_store::<ModeState>();
    let text_color = match store.mode {
        Mode::Dark => "white",
        Mode::Light => "black",
    };
    let stylesheet = css!(
        "
        .logo {
            width: 80px;
            
        }
        div {
            position: fixed;
            bottom: 0px;
        }
        .right {
            position: fixed;
            right: 0px;
        }
        .left {
            position: fixed;
            left: 0px;
        }
        span {
            color: ${text_color};
            margin: 3px;
        }
        a {
            color: ${text_color};
            margin: 5px;
        }
        .icon {
            width: 24px; 
            height: 24px;
            margin: 5px;
        }
      
        ",
        text_color = text_color
    );

    let github_icon = match store.mode {
        Mode::Dark => "assets/github-dark.png",
        Mode::Light => "assets/github-light.png",
    };
    html!(
        <footer class ={classes!("column",{stylesheet})}>
                <Logo />
                <display_name> {"You are known as"} <span>{current_peer_name}</span></display_name>
                <note class="smallfont">{"You can be discovered by everyone on this network"}</note>

                <div class="right">
                    <Link icon={github_icon} target="https://github.com/AbdesamedBendjeddou/Shrut/" alt="Github" />
                    <Link icon="assets/twitter.png" target="https://twitter.com/abd_esamad" alt="Twitter" />
                </div>

        </footer>
    )
}
