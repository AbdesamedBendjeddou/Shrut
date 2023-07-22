use stylist::style;
use yew::prelude::*;

use crate::components::atoms::{dark_mode::DarkMode, overlay::Info};

#[function_component]
pub fn Header() -> Html {
    let stylesheet = style!(
        "
        button {
            position: relative;
            display: flex;
            align-items: center;
            justify-content: center;
            border: none;
            background-color: transparent;
            outline: none;
            touch-action: manipulation;

        }
        .icon {
            width: 25px;
            height:25px;
        }
      
        "
    )
    .unwrap();

    html! {

        <header class ={classes!("row-reverse",{stylesheet})}>
                    <Info />
                    <DarkMode />
        </header>
    }
}
