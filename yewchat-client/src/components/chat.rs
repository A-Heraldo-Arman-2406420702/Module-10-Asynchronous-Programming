use serde::{Deserialize, Serialize};
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
                // Gunakan pengecekan aman, bukan unwrap() agar tidak crash
                if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&s) {
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
                            if let Some(data) = msg.data {
                                if let Ok(message_data) = serde_json::from_str::<MessageData>(&data) {
                                    self.messages.push(message_data);
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                false
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
        
        // Avatar default jika ada kendala sinkronisasi user lambat
        let default_avatar = "https://avatars.dicebear.com/api/adventurer-neutral/Unknown.svg".to_string();

        html! {
            <div class="flex w-screen h-screen font-sans overflow-hidden">
                
                <div class="flex-none w-72 bg-slate-900 border-r border-slate-800 flex flex-col shadow-2xl z-20">
                    <div class="p-6 border-b border-slate-800 bg-slate-900/50">
                        <h1 class="text-2xl font-extrabold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-500 tracking-wide">
                            {"Chatty"}
                        </h1>
                        <p class="text-xs text-slate-400 mt-1">{"Online Users"}</p>
                    </div>
                    
                    <div class="flex-grow overflow-y-auto p-4 space-y-3">
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex items-center p-3 bg-slate-800/80 hover:bg-slate-700 rounded-xl transition-all shadow-sm cursor-pointer border border-slate-700/50 hover:border-slate-600">
                                        <div class="relative">
                                            <img class="w-12 h-12 rounded-full ring-2 ring-blue-500/50 bg-slate-200" src={u.avatar.clone()} alt="avatar"/>
                                            <span class="absolute bottom-0 right-0 w-3.5 h-3.5 bg-green-500 border-2 border-slate-800 rounded-full"></span>
                                        </div>
                                        <div class="ml-4 flex-grow">
                                            <div class="text-sm font-bold text-slate-200">{u.name.clone()}</div>
                                            <div class="text-xs text-slate-400 font-medium">{"Active now"}</div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                <div class="grow flex flex-col bg-slate-950 relative">
                    
                    <div class="w-full h-16 bg-slate-900/90 backdrop-blur-md border-b border-slate-800 flex items-center px-8 z-10 shadow-sm sticky top-0">
                        <div class="flex items-center">
                            <span class="text-xl font-bold text-slate-100">{"💬 Global Channel"}</span>
                            <span class="ml-3 px-2 py-1 bg-blue-500/10 text-blue-400 text-xs font-semibold rounded-md border border-blue-500/20">{"Public"}</span>
                        </div>
                    </div>

                    <div class="w-full grow overflow-auto p-6 space-y-6">
                        {
                            self.messages.iter().map(|m| {
                                let avatar_url = self.users.iter()
                                    .find(|u| u.name == m.from)
                                    .map(|u| u.avatar.clone())
                                    .unwrap_or_else(|| default_avatar.clone());
                                    
                                html!{
                                    <div class="flex items-end max-w-3xl hover:bg-slate-900/50 p-2 rounded-xl transition-colors">
                                        <img class="w-10 h-10 rounded-full mb-1 shadow-md ring-1 ring-slate-700 bg-slate-200" src={avatar_url} alt="avatar"/>
                                        <div class="ml-3">
                                            <div class="text-xs text-slate-400 mb-1 ml-1 font-semibold flex items-center">
                                                {m.from.clone()} 
                                                <span class="text-[10px] text-slate-600 ml-2 font-normal">{"Just now"}</span>
                                            </div>
                                            <div class="px-5 py-3 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-2xl rounded-bl-none shadow-lg transition-colors border border-blue-400/20">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-2 rounded-lg max-h-48 object-cover" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    <div class="w-full p-6 bg-slate-950/80 backdrop-blur-lg border-t border-slate-800/80">
                        <div class="relative flex items-center max-w-5xl mx-auto">
                            <input 
                                ref={self.chat_input.clone()} 
                                type="text" 
                                placeholder="Type your message here..." 
                                class="w-full py-4 pl-6 pr-16 bg-slate-800/80 text-slate-100 placeholder-slate-400 rounded-full outline-none focus:ring-2 focus:ring-blue-500 transition-all shadow-inner border border-slate-700" 
                                name="message" 
                                required=true 
                            />
                            <button 
                                onclick={submit} 
                                class="absolute right-2 w-12 h-12 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-400 hover:to-purple-500 rounded-full flex justify-center items-center shadow-lg transition-transform transform hover:scale-105 hover:shadow-blue-500/25"
                            >
                                <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white w-5 h-5 ml-1">
                                    <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}