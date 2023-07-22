use yew::prelude::*;
use yewdux::prelude::*;
use stylist::{yew::Global, css};
use crate::components::atoms::{dark_mode::{ModeState, Mode}, overlay::{Overlay, State}};


#[function_component]
pub fn GlobalCss() -> Html {
    let (store, _) = use_store::<ModeState>();

    let (background_color, text_color) = match store.mode {
        Mode::Dark => ("#0C0C1E", "white"),
        Mode::Light => ("#F3E3E2", "black"),
    };

    let (overlay_state, _) = use_store::<Overlay>();

    let display = match overlay_state.state {
        State::Close => "none",
        State::Open => "black",
    };

    let css = css!(
        "
        html,body {
          height: 100%;
        }

        html,body {
            overflow: hidden;
        }

        body {
          margin: 0;
          display: flex;
          flex-direction: column;
          width: 100%;
          overflow-x: hidden;
          flex-grow: 1;
          align-items: center;
          justify-content: center;
          overflow-y: hidden;
          color: ${text_color};
          background-color: ${background_color};
          font-size: 14px;
          font-weight: 400;
          line-height: 20px;
          font-family: 'Roboto', 'Arial';
          -webkit-font-smoothing: antialiased;
          /*-moz-osx-font-smoothing: grayscale;*/
      }
        .smallfont {
          font-size: 13px;
          line-height: 18px;
        }

        header {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          height: 56px;
          align-items: center;
          padding: 16px;
          box-sizing: border-box;
        }

        .center {
          display: flex;
          align-items: center;
          justify-content: center;
        }

        footer {
          position: absolute;
          bottom: 0;
          left: 0;
          right: 0;
          align-items: center;
          padding: 0 0 35px 0;
          text-align: center;
        }
       
        body>header a {
          margin-left: 8px;
        }

        [hidden] {
        display: none !important;
        }

        .row-reverse {
          display: flex;
          flex-direction: row-reverse;
        }  
        
        .row {
          display: flex;
          flex-direction: row;
        }
      
        .column {
          display: flex;
          flex-direction: column;
        }

        .grow {
          flex-grow: 1;
        }
      
        .full {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
        }

        .modal-overlay {
          display: ${display};
          position: fixed;
          z-index: 2;
          top: 0;
          bottom: 0;
          left: 0;
          right: 0;
          background-color: rgba(0,0,0,.6);
          overflow: auto;
          box-sizing: border-box;
          margin: 0;
      }

      canvas {
        position: absolute;
        width: 100%;
        z-index: -1;
        top: 0;
        left: 0;
        aspect-ratio: auto;
        /*overflow-clip-margin: content-box;*/
        overflow: clip;
      }

      @keyframes fade-in {
        0% {
            opacity: 0;
        }
      }

    @keyframes pop {
      0% {
          transform: scale(0.7);
      }
    
      40% {
          transform: scale(1.2);
        }
      }
      ",
        text_color = text_color,
        background_color = background_color,
        display = display
    );

    html! {
        <>
            <Global {css} />
        </>
    }
}
