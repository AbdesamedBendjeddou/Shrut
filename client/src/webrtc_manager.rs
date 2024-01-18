use std::{cell::{RefCell, Cell}, rc::Rc, collections::VecDeque};

use gloo::{
    console::{error, log},
    file::Blob,
};
use js_sys::{Array, ArrayBuffer, Object, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelInit,
    RtcIceCandidate, RtcIceCandidateInit, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcSdpType,
    RtcSessionDescriptionInit, Event,
};
use yew::platform::spawn_local;

use crate::components::atoms::{
    avatar::{FileData, send_till_buffer_full},
    messages::{
        AppMessage::{self, *},
        ClientMessage, ServerMessage, SignalingMessage,
    },
    other_peers_state::WebRTCRole,
};
pub const MAX_CHUNK_SIZE: u32 = 16384;
const BUFFERED_AMOUNT_LOW_THRESHOLD: u32 = MAX_CHUNK_SIZE * 4;
const STUN_SERVER: &str = "stun:stun.l.google.com:19302";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}
#[derive(Clone)]
pub struct WebRtcConnection {
    pub peer_connection: RtcPeerConnection,
    pub data_channel: RtcDataChannel,
}

impl WebRtcConnection {
    pub fn new() -> Self {
        let peer_connection = Self::create_peer_connection();
        let data_channel = Self::create_data_channel(&peer_connection);

        WebRtcConnection {
            peer_connection,
            data_channel,
        }
    }

    pub fn init(&self, tx: Sender<AppMessage>, other_peer: Uuid, role: WebRTCRole) {
        self.set_on_message_callback(other_peer);
        self.set_on_error_callback();
        self.set_on_ice_candidate(tx.clone(), other_peer);
        self.add_ice_candidate(tx.clone());
        self.set_peeer_connection_on_data_channel(other_peer);
        self.set_on_open();
        self.set_on_close_callback();
        self.exchange_offers(tx, other_peer, role);
        self.data_channel
            .set_buffered_amount_low_threshold(BUFFERED_AMOUNT_LOW_THRESHOLD);
    }

    pub fn exchange_offers(&self, tx: Sender<AppMessage>, other_peer: Uuid, role: WebRTCRole) {
        let peer_connection = self.peer_connection.clone();
        spawn_local(async move {
            log!(format!("the contacted peer is {:?}", role));
            match role {
                WebRTCRole::Client => {
                    let offer = Self::create_offer(&peer_connection).await;
                    let offer_msg = CltMsg(ClientMessage::SignalingMessage(
                        SignalingMessage::Offer(other_peer, offer),
                    ));
                    tx.send(offer_msg)
                        .map_err(|err| log!(format!("{:?}", err)))
                        .expect("error sending offer msg");
                    let mut rx = tx.subscribe();
                    while let Ok(msg) = rx.recv().await {
                        if let SrvrMsg(ServerMessage::SignalingMessage(SignalingMessage::Answer(
                            responded_peer,
                            answer,
                        ))) = msg
                        {
                            if other_peer == responded_peer {
                                Self::receive_answer(&peer_connection, answer).await;
                                break;
                            }
                        }
                    }
                }
                WebRTCRole::Server => {
                    let mut rx = tx.subscribe();
                    while let Ok(msg) = rx.recv().await {
                        log!(format!(" offer received{:?}", msg));
                        if let SrvrMsg(ServerMessage::SignalingMessage(SignalingMessage::Offer(
                            signaler_peer,
                            offer,
                        ))) = msg
                        {
                            if other_peer == signaler_peer {
                                let answer = Self::create_answer(&peer_connection, offer).await;
                                let answer_msg = CltMsg(ClientMessage::SignalingMessage(
                                    SignalingMessage::Answer(signaler_peer, answer),
                                ));
                                tx.send(answer_msg).expect("error sendig answer");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    async fn create_offer(peer_connection: &RtcPeerConnection) -> String {
        let offer = JsFuture::from(peer_connection.create_offer())
            .await
            .expect("error creating offer");
        let offer = Reflect::get(&offer, &JsValue::from_str("sdp"))
            .expect("error creating offer sdp")
            .as_string()
            .unwrap();
        log!("created offer");
        let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        offer_obj.sdp(&offer);
        let sld_promise = peer_connection.set_local_description(&offer_obj);
        JsFuture::from(sld_promise)
            .await
            .expect("error sld_promise");
        log!("pc1: state {:?}", peer_connection.signaling_state());
        offer
    }
    async fn create_answer(peer_connection: &RtcPeerConnection, offer: String) -> String {
        let mut remote_session_description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        remote_session_description.sdp(&offer);
        JsFuture::from(peer_connection.set_remote_description(&remote_session_description))
            .await
            .expect("error set remote desreption");

        let answer = JsFuture::from(peer_connection.create_answer())
            .await
            .expect("error creating answer");
        let answer = Reflect::get(&answer, &JsValue::from_str("sdp"))
            .expect("error Relect::get answer")
            .as_string()
            .expect("failed to represent object value as string");

        let mut local_session_description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        local_session_description.sdp(&answer);
        JsFuture::from(peer_connection.set_local_description(&local_session_description))
            .await
            .expect("failed to set local description");

        answer
    }

    async fn receive_answer(peer_connection: &RtcPeerConnection, answer: String) {
        let mut answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        answer_obj.sdp(&answer);
        let srd_promise = peer_connection.set_remote_description(&answer_obj);
        JsFuture::from(srd_promise)
            .await
            .expect("error receiving answer");
        log!("pc1: state {:?}", peer_connection.signaling_state());
    }

    fn add_ice_candidate(&self, tx: Sender<AppMessage>) {
        let peer_connection = self.peer_connection.clone();
        let mut rx = tx.subscribe();
        spawn_local(async move {
            while let Ok(msg) = rx.recv().await {
                if let SrvrMsg(ServerMessage::SignalingMessage(SignalingMessage::IceCandidate(
                    _,
                    ice_candidate,
                ))) = msg
                {
                    let mut rtc_candidate = RtcIceCandidateInit::new("");
                    rtc_candidate.candidate(&ice_candidate.candidate);
                    rtc_candidate.sdp_m_line_index(ice_candidate.sdp_m_line_index);
                    rtc_candidate.sdp_mid(ice_candidate.sdp_mid.as_deref());

                    let rtc_candidate = RtcIceCandidate::new(&rtc_candidate)
                        .expect("failed to create new RtcIceCandidate");
                    JsFuture::from(
                        peer_connection
                            .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&rtc_candidate)),
                    )
                    .await
                    .expect("failed to add ICE candidate");
                    log!("ice candidate added");
                }
            }
        });
    }

    fn set_on_ice_candidate(&self, tx: Sender<AppMessage>, other_peer: Uuid) {
        let on_ice_candidate: Box<dyn FnMut(RtcPeerConnectionIceEvent)> =
            Box::new(move |ev: RtcPeerConnectionIceEvent| {
                if let Some(candidate) = ev.candidate() {
                    let ice_candidate = IceCandidate {
                        candidate: candidate.candidate(),
                        sdp_mid: candidate.sdp_mid(),
                        sdp_m_line_index: candidate.sdp_m_line_index(),
                    };
                    let signaling_message = CltMsg(ClientMessage::SignalingMessage(
                        SignalingMessage::IceCandidate(other_peer, ice_candidate),
                    ));
                    tx.send(signaling_message)
                        .expect("error sendig form set on ice candidate");
                }
            });
        let on_ice_candidate = Closure::wrap(on_ice_candidate);
        self.peer_connection
            .set_onicecandidate(Some(on_ice_candidate.as_ref().unchecked_ref()));
        on_ice_candidate.forget();
    }

    fn set_peeer_connection_on_data_channel(&self, other_peer: Uuid) {
        let on_datachannel: Box<dyn FnMut(RtcDataChannelEvent)> =
            Box::new(move |data_channel_event: RtcDataChannelEvent| {
                log!("on_data_channel on this peer excuted");
                let data_channel = data_channel_event.channel();
                // let data_channel_clone = data_channel.clone();
                let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
                    log!(ev.data().is_array(), ev.data().is_object());
                    let message = ev.data().dyn_into::<ArrayBuffer>().ok();
                    log!(
                        "message recv fonm on data channel",
                        format!("{:?}", message.clone())
                    );
                    if let Some(message) = message {
                        log!(
                            "message from datachannel (will call on_message)",
                            format!("{:?}", message.byte_length())
                        );
                    }
                })
                    as Box<dyn FnMut(MessageEvent)>);
                data_channel.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();
                log!("this peer sent: it's working ya rab ");
                data_channel
                    .send_with_str("it's working ya rab")
                    .expect("error sending ya rab");
            });

        let on_datachannel = Closure::wrap(on_datachannel);
        self.peer_connection
            .set_ondatachannel(Some(on_datachannel.as_ref().unchecked_ref()));
        on_datachannel.forget();
    }

    fn set_on_open(&self) {
        let on_open_callback: Box<dyn FnMut(JsValue)> = Box::new(move |_| {
            log!("data channel is open");
        });
        let on_open_callback = Closure::wrap(on_open_callback);
        self.data_channel
            .set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));
        on_open_callback.forget();
    }

    fn set_on_message_callback(&self, other_peer: Uuid) {
        let data_channel_clone = self.data_channel.clone();
        let on_message_callback: Box<dyn FnMut(MessageEvent)> =
            Box::new(move |ev: MessageEvent| {
                let message = ev.data().dyn_into::<Uint8Array>().ok();
                log!("message rec", format!("{:?}", message));
                if let Some(message) =
                    message.and_then(|t| rmp_serde::from_slice::<FileData>(&t.to_vec()).ok())
                {
                    log!("message from datachannel (will call on_message)");
                    let blob = Blob::new_with_options(&*message.data, Some(&message.file_typ));
                    log!(blob.raw_mime_type());
                    log!(message.name);
                    //tbd
                }
                match ev.data().as_string() {
                    Some(message) => {
                        log!(
                            "this peer received: {}",
                            message,
                            "from",
                            other_peer.to_string()
                        );
                        log!("this peer sent: ping to: ", other_peer.to_string());
                        data_channel_clone
                            .send_with_str("ping")
                            .expect("error sending form on message callback");
                    }
                    None => todo!(),
                }
            });
        let on_message_callback = Closure::wrap(on_message_callback);
        self.data_channel
            .set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));
        on_message_callback.forget();
    }

    fn set_on_error_callback(&self) {
        let on_error: Box<dyn FnMut(JsValue)> = Box::new(move |data_channel_error| {
            error!("data channel error: {:?}", data_channel_error);
        });
        let on_error = Closure::wrap(on_error);
        self.data_channel
            .set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();
    }

    fn set_on_close_callback(&self) {
        let on_close: Box<dyn FnMut(JsValue)> = Box::new(move |_| {
            log!("data channel closed");
        });
        let on_close = Closure::wrap(on_close);
        self.data_channel
            .set_onerror(Some(on_close.as_ref().unchecked_ref()));
        on_close.forget();
    }

  
    
    fn create_data_channel(peer_connection: &RtcPeerConnection) -> RtcDataChannel {
        let mut init = RtcDataChannelInit::new();
        init.max_retransmits(16);
        init.ordered(false);
        let data_channel = peer_connection.create_data_channel_with_data_channel_dict("dc", &init);
        data_channel
    }

    fn create_peer_connection() -> RtcPeerConnection {
        let ice_servers = Array::new();
        {
            let server_entry = Object::new();

            Reflect::set(
                &server_entry,
                &"urls".into(),
                &STUN_SERVER.to_owned().into(),
            )
            .expect("error creating stun entry");

            ice_servers.push(&server_entry);
        }

        let mut rtc_configuration = RtcConfiguration::new();
        rtc_configuration.ice_servers(&ice_servers);

        RtcPeerConnection::new_with_configuration(&rtc_configuration)
            .expect("error creating RtcPeerConnection")
    }
}
