use stylist::css;
use tokio::sync::broadcast::{channel, Sender};
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::{
    components::atoms::{avatar::Avatar, other_peers_state::OtherPeers, this_peer_state::ThisPeer, messages::AppMessage},
    connection_manager::connection_manager,
};

#[function_component]
pub fn Center() -> Html {
    let stylesheet = css!(
        "
        no-peers {
            padding: 8px;
            text-align: center;
            animation: fade-in 300ms;
            animation-delay: 500ms;
            animation-fill-mode: backwards;
        }

        instruction {
            position: absolute;
            left:0;
            right:0;
            top: 200px;
            opacity: 0.5;
            transition: opacity 300ms;
            z-index: 1;
        }
        peers {
            width: 100%;
            overflow: hidden;
            flex-flow: row wrap;
            z-index: 2;
        }
        
        avatar {
            width: var(--peer-width);
            padding: 8px;
            cursor: pointer;
            touch-action: manipulation;
            position: relative;
        }
        
        
        "
    );

    let tx = &*use_state(|| channel::<AppMessage>(100).0);
    let (_, this_peer_dispatch) = use_store::<ThisPeer>();
    let other_peers = use_reducer_eq(|| OtherPeers::default());
    {
        let tx = tx.clone();
        let other_peers = other_peers.dispatcher();
        use_effect_with_deps(
            move |()| {
                connection_manager(other_peers, this_peer_dispatch, tx);
            },
            (),
        );
    }

    html! {
    <center class={stylesheet}>
        if other_peers.peers.is_empty() {
            <no-peers>
                <h3>{"Open Shrut on other devices to send files"}</h3>
            </no-peers>

        } else {
            <instruction class="smallfont">
                {"Tap or click to send a file"}
            </instruction>
            <peers class="center">
                {display_peers(other_peers.clone(), tx.clone())}
            </peers>
        }
    </center>
    }
}

fn display_peers(other_peers: UseReducerHandle<OtherPeers>, tx: Sender<AppMessage>) -> Vec<Html> {
    other_peers
        .peers
        .iter()
        .map(|peer| {
            html! {
                <Avatar key={peer.id.clone().to_string()} id={peer.id.clone()} name={peer.name.clone()} os={peer.os.clone()} role={peer.role.clone()} tx={tx.clone()} />
            }
        })
        .collect()
}
