use std::collections::VecDeque;

use dioxus::prelude::*;
use gloo_timers::callback::Timeout;

use crate::{
       Dashboard, Header,
    NotificationInfo,
};

const FAVICON: Asset = asset!("/assets/favicon.png");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");


pub(crate) static GLOBAL_MESSAGE: GlobalSignal<VecDeque<NotificationInfo>> =
    Signal::global(|| VecDeque::default());


pub(crate) static LOADING: GlobalSignal<Option<()>> = Signal::global(|| Option::default());


#[component]
pub(crate) fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Title {"Dust Checker"}

        div { class: "w-full flex min-h-screen font-[sans-serif] dark:bg-rich-black bg-white text-black dark:text-white",

            Notification {}


            div { class: "flex flex-col w-full min-h-full justify-between items-center",
                Router::<Route> {}
            }
        }
    }
}

#[component]
fn Notification() -> Element {
    if GLOBAL_MESSAGE.read().is_empty() {
        return rsx! {};
    }

    let message_index = |key: u32| {
        let messages = GLOBAL_MESSAGE.read();
        let found_message = messages
            .iter()
            .enumerate()
            .find(|(_, value)| value.key() == key)
            .map(|(index, _value)| index);
        drop(messages);

        found_message
    };

    let timer_callback = |secs: u32, key: u32| {
        // Start a timeout for each notification
        spawn(async move {
            let timeout = Timeout::new(secs * 1000, move || {
                message_index(key).map(|index| GLOBAL_MESSAGE.write().remove(index));
            });
            timeout.forget();
        });
    };

    let mut key = Some(0u32);

    rsx! {
        div {
            class: "cursor-pointer fixed z-1000 top-4 right-4 flex flex-col space-y-2 min-w-[300px] shadow-lg",
            for notification_info in GLOBAL_MESSAGE.read().clone().iter() {
                {key.replace(notification_info.key());}
                {timer_callback(notification_info.secs(), notification_info.key())}

                div {
                    onclick:move|_|{
                        if let Some(key_inner) = key {
                            message_index(key_inner).map(|index| GLOBAL_MESSAGE.write().remove(index));
                        }
                        key.take();
                    },
                    key: "{notification_info.key()}",
                    class: "flex border dark:border-none items-center opacity-0 translate-y-4 animate-fade-in w-full max-w-xs p-2 space-x-2 text-gray-600 bg-white divide-x divide-gray-200 rounded-lg shadow-sm dark:text-gray-400 dark:divide-gray-700 dark:bg-gray-800",
                    div { class:"flex w-[30px]",
                        svg {
                            view_box: "0 0 24 24",
                            xmlns: "http://www.w3.org/2000/svg",
                            path {
                                d: "m10 20h4a2 2 0 0 1 -4 0zm8-4v-6a6 6 0 0 0 -5-5.91v-1.09a1 1 0 0 0 -2 0v1.09a6 6 0 0 0 -5 5.91v6l-2 2h16z",
                                fill: "#0060df",
                            }
                        }
                    }
                    div { class: "ps-4 text-sm font-normal", "{notification_info.message()}" }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route { 
    #[layout(Header)]
        #[route("/")]
        Dashboard(),
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}
