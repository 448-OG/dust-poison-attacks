use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Header() -> Element {
    rsx! {
        div { class:"flex flex-col w-full gap-4 justify-between items-center",
            nav {class:"flex w-full justify-around items-center p-1 dark:shadow-lg shadow-sm border-b-[1px] dark:border-true-blue",
                div{ class:"flex items-center justify-around w-[80%] mx-2",
                    "DUST CHECKER"
                }
            }
        }
        Outlet::<Route> {}
    }
}
