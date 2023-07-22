use gloo::{
    console::log,
    file::callbacks::{read_as_array_buffer, read_as_bytes},
    utils::format::JsValueSerdeExt,
};
use js_sys::{Array, ArrayBuffer, Uint8Array};
use std::{cell::Cell, rc::Rc, thread::sleep, time::Duration};
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use stylist::css;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{File, FileList, FilePropertyBag, FileReader, HtmlInputElement};
use yew::prelude::*;

use crate::webrtc_manager::WebRtcConnection;

use super::{messages::AppMessage, other_peers_state::WebRTCRole};

#[derive(Serialize, Deserialize)]
pub struct FileData {
    pub name: String,
    pub file_typ: String,
    pub data: Vec<u8>,
}

#[derive(Properties, Clone, Deserialize, Debug)]
pub struct OtherPeer {
    pub id: Uuid,
    pub name: String,
    pub os: String,
    #[serde(skip_deserializing)]
    pub role: WebRTCRole,
    #[serde(skip_deserializing)]
    pub tx: Option<Sender<AppMessage>>,
}

impl PartialEq for OtherPeer {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.os == other.os && self.id == other.id
    }
}

impl OtherPeer {
    pub fn set_role(&mut self, role: WebRTCRole) {
        self.role = role;
    }
}

#[function_component]
pub fn Avatar(props: &OtherPeer) -> Html {
    log!("new peer joind with id:", props.id.clone().to_string());
    let icon = match &props.os.to_lowercase()[..] {
        "windows" => "assets/windows.png",
        "mac os x" => "assets/mac.png",
        "linux" => "assets/linux.png",
        "android" => "assets/android.png",
        "ios" => "assets/ios.png",
        _ => "assets/unknown.png",
    };

    let stylesheet = css!(
        "
        input {
            visibility: hidden;
            position: absolute;
        }

        icon {
            display: flex;
            animation: pop 600ms ease-out 1;
            width: 65px;
            height: 65px;
            border-radius: 50%;
            background:#C1C8E4;
            margin-bottom: 8px;
            transition: transform 150ms;
            will-change: transform;
            align-items: center;
            justify-content: center;
        }
        img {
            width: 50px;
        }
        name {
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            text-align: center;
        }
        p {
            opacity: 0.5;
            transition: opacity 300ms;
        }
  
        "
    );

    let webrtc_connection = use_state(|| {
        log!("id of peer connecting to:", props.id.clone().to_string());
        let webrtc_connection = WebRtcConnection::new();
        webrtc_connection.init(
            props.tx.clone().unwrap(),
            props.id.clone(),
            props.role.clone(),
        );
        webrtc_connection
    });

    let ondragover = Callback::from(move |event: DragEvent| {
        event.prevent_default();
    });
    let ondragenter = Callback::from(move |event: DragEvent| {
        event.prevent_default();
    });

    let ondrop = Callback::from(move |event: DragEvent| {
        event.prevent_default();
        //let file = event.data_transfer().unwrap().files().unwrap();
    });

    let webrtc_connection_clone = webrtc_connection.clone();
    let onchange = Callback::from(move |event: Event| {
        log!("on change");
        let input: HtmlInputElement = event.target_unchecked_into();
        let files = upload_files(input.files());
        send_files(&*webrtc_connection_clone, files);
    });

    html! {
         <avatar class={classes!("column","center",{stylesheet})}>
            <label for="input" {ondrop} {ondragenter} {ondragover}>
               <icon>
                    <img class="icon" src={icon} alt="avatar" />
                </icon>
                <name class="smallfont">{&props.name}</name>
                //todo! if *transfering {
                    <p class="smallfont">{"Transfering"}</p>
               // }


            </label>
               <input {onchange} id="input" type="file" multiple=true/>

        </avatar>
    }
}

fn upload_files(files: Option<FileList>) -> Vec<File> {
    let mut uploaded_files = vec![];
    if let Some(files) = files {
        let files = js_sys::try_iter(&files)
            .unwrap()
            .unwrap()
            .map(|v| web_sys::File::from(v.unwrap()));
        //.map(File::from);
        uploaded_files.extend(files);
    }
    log!("file unloaded", uploaded_files.len());
    uploaded_files
}


fn send_files(webrtc_connection: &WebRtcConnection, files: Vec<File>) {
    for file in files {
        send_file(webrtc_connection, file)
    }
}

fn send_file(webrtc_connection: &WebRtcConnection, file: File /*FileData*/) {
    log!("sending file");
    log!(format!("{}", file.size()));

    let file_ = file.clone();
    let name = file.name();
    let data_channel = webrtc_connection.data_channel.clone();
    
    let file_clone = file.clone();
    let file_reader = web_sys::FileReader::new().unwrap();
    let file_reader_clone = file_reader.clone();
    let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: web_sys::ProgressEvent| {
        let file = file_reader_clone.result().unwrap();
        let arraybuff = js_sys::ArrayBuffer::from(file);
        log!(arraybuff.byte_length());

        //slicing
        //let chunks = array.slice(begin);

        //for chunk in chunks {
        //    data_channel.send_with_array_buffer(&chunks).expect("err data channel");
        //}

        let array = Array::from(&arraybuff);
        log!(array.length()); // length = 0
        let file = File::new_with_buffer_source_sequence(&array, &name).unwrap();
        log!(file.size()); // size = 0
    });
    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader
        .read_as_array_buffer(&file_)
        .expect("blob not readable");
    onloadend_cb.forget();
}


