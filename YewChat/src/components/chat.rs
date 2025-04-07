﻿use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen font-sans">
                <div class="flex-none w-64 h-screen bg-gradient-to-b from-purple-700 to-purple-900 text-white overflow-auto">
                    <div class="text-2xl font-semibold p-4 border-b border-purple-500">{"👥 Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-purple-800 rounded-xl p-3 items-center shadow-md hover:bg-purple-700 transition">
                                    <img class="w-10 h-10 rounded-full border-2 border-white" src={u.avatar.clone()} alt="avatar"/>
                                    <div class="ml-3">
                                        <div class="text-sm font-medium">{u.name.clone()}</div>
                                        <div class="text-xs text-purple-200">{"Active now"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col bg-purple-50">
                    <div class="w-full h-16 bg-white shadow-md flex items-center px-6 border-b border-purple-200">
                        <div class="text-xl font-bold text-purple-800">{"💬 Purple Chat"}</div>
                    </div>
                    <div class="flex-grow overflow-auto px-6 py-4 space-y-4">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                if let Some(user) = user {
                                    html! {
                                        <div class="flex items-start space-x-3">
                                            <img class="w-8 h-8 rounded-full" src={user.avatar.clone()} alt="avatar"/>
                                            <div class="bg-white p-3 rounded-xl shadow-sm max-w-xl">
                                                <div class="text-sm font-semibold text-purple-700">{m.from.clone()}</div>
                                                <div class="text-sm text-gray-700 mt-1">
                                                    {
                                                        if m.message.ends_with(".gif") {
                                                            html! { <img class="mt-2 rounded-md" src={m.message.clone()} /> }
                                                        } else {
                                                            html! { { &m.message } }
                                                        }
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-16 bg-white flex items-center px-4 border-t border-purple-200">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Type your message..."
                            class="flex-grow px-4 py-2 rounded-full border border-purple-300 focus:outline-none focus:ring-2 focus:ring-purple-500 transition"
                            required=true
                        />
                        <button
                            onclick={submit}
                            class="ml-3 p-3 bg-purple-600 hover:bg-purple-700 rounded-full shadow-md transition"
                        >
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 fill-white">
                                <path d="M0 0h24v24H0z" fill="none"/>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}