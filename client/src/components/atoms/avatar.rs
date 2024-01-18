use gloo::{console::log, events::EventListener};
use js_sys::{Array, ArrayBuffer, Uint8Array};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use serde::{Deserialize, Serialize};
use stylist::css;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{File, FileList, HtmlInputElement, RtcDataChannel};
use yew::prelude::*;

use crate::webrtc_manager::{WebRtcConnection, MAX_CHUNK_SIZE};

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

    let data_channel = webrtc_connection.data_channel.clone();
    let onchange = Callback::from(move |event: Event| {
        log!("on change");
        let input: HtmlInputElement = event.target_unchecked_into();
        let files = upload_files(input.files());
        send_files(data_channel.clone(), files);
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

fn send_files(data_channel: RtcDataChannel, files: Vec<File>) {
    for file in files {
        read_as_array_buffer_then_send(data_channel.clone(), file)
    }
}
fn read_as_array_buffer_then_send(data_channel: RtcDataChannel, file: File) {
    log!("sending file");
    log!(format!("{}", file.size()));

    let file_reader = web_sys::FileReader::new().unwrap();
    let file_reader_clone = file_reader.clone();
    let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: web_sys::ProgressEvent| {
        let file = file_reader_clone.result().unwrap();
        let array_buffer: ArrayBuffer = file.unchecked_into();
        send_as_array_buffer(data_channel.clone(), array_buffer);
    });

    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader
        .read_as_array_buffer(&file)
        .expect("blob not readable");
    onloadend_cb.forget();
}

fn send_as_array_buffer(data_channel: RtcDataChannel, array_buffer: ArrayBuffer) {
    /*
    let data_channel = webrtc_connection.data_channel.clone();
    {
        let data_channel_ = data_channel.clone();
        let queue = queue.clone();
        let bytes_sent = bytes_sent.clone();
        let onbuff_amount_low = Closure::wrap(Box::new(move |_| {
            send_till_buffer_full(data_channel_.clone(), queue.clone(), bytes_sent.clone());
        }) as Box<dyn FnMut(Event)>);
            data_channel.set_onbufferedamountlow(Some(onbuff_amount_low.as_ref().unchecked_ref()));
            onbuff_amount_low.forget();
        }
        */
    let bytes_sent = Rc::new(Cell::new(0));
    let queue = Rc::new(RefCell::new(VecDeque::new()));
    set_onbufferedamountlow(data_channel.clone(), queue.clone(), bytes_sent.clone());

    let array_length = array_buffer.byte_length();
    let numb_of_slices = (array_length / MAX_CHUNK_SIZE) + 1;
    let mut begin = 0;
    for _ in 0..numb_of_slices {
        let chunk = array_buffer.slice_with_end(begin, begin + MAX_CHUNK_SIZE);
        log!(chunk.byte_length());
        queue.borrow_mut().push_back(chunk);
        send_till_buffer_full(data_channel.clone(), queue.clone(), bytes_sent.clone());
        begin += MAX_CHUNK_SIZE;
    }
}

fn set_onbufferedamountlow(
    data_channel: RtcDataChannel,
    queue: Rc<RefCell<VecDeque<ArrayBuffer>>>,
    bytes_sent: Rc<Cell<u32>>,
) {
    let data_channel_ = data_channel.clone();
    let onbuff_amount_low_cb = Closure::wrap(Box::new(move |_| {
        send_till_buffer_full(data_channel_.clone(), queue.clone(), bytes_sent.clone());
    }) as Box<dyn FnMut(Event)>);
    data_channel.set_onbufferedamountlow(Some(onbuff_amount_low_cb.as_ref().unchecked_ref()));
    onbuff_amount_low_cb.forget();
}

pub fn send_till_buffer_full(
    data_channel: RtcDataChannel,
    queue: Rc<RefCell<VecDeque<ArrayBuffer>>>,
    bytes_sent: Rc<Cell<u32>>,
) {
    loop {
        let chunk = queue.borrow_mut().pop_front();
        if let Some(chunk) = chunk {
            let res = data_channel.send_with_array_buffer(&chunk);
            if res.is_err() {
                queue.borrow_mut().push_front(chunk);
                break;
            }
            bytes_sent.set(bytes_sent.get() + MAX_CHUNK_SIZE);
            log!("bytes sent: ", bytes_sent.get());
        } else {
            break;
        }
    }
}

/*  //let len = queue.borrow().len();
for _ in  0..queue.borrow().len() {
    log!("buff ammount", data_channel.buffered_amount());
    //if data_channel.buffered_amount() < data_channel.buffered_amount_low_threshold() {
       // log!("buff is less");
       // if err {continue;}
//slicing
//let chunks = array.slice(begin);

//for chunk in chunks {
//}

let array = Array::of1(&arraybuff);

log!(array.length()); // length = 0
let file = File::new_with_buffer_source_sequence(&array, &name).unwrap();
log!(file.size()); // size = 0
log!(file.type_()); */
