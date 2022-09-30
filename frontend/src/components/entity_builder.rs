use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yewdux::prelude::*;

// use web_sys::HtmlInputElement;

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
struct EntityContent {
    content: String,
}

// enum Msg {
//     Clicked,
// }

// struct EntityBuilder;

// impl Component for EntityBuilder {

//     type Message = Msg;
//     type Properties = ();

//     fn create(_ctx: &Context<Self>) -> Self {
//         let onclick = dispatch.reduce_mut_callback(|content| content.content = "sdflkj".to_string());
//         Self
//     }

//     fn view(&self, ctx: &Context<Self>) -> Html {

//         let onkeypress = ctx.link().batch_callback(|event: KeyboardEvent| {
//             if event.key() == "Enter" {
//                 Some(Msg::Submit)
//             } else {
//                 None
//             }
//         });

//         html! {
//             <input type="text" {onkeypress} />
//         }
//     }
// }

#[function_component(EntityBuilder)]
pub fn entity_builder() -> Html {
    let (content, dispatch) = use_store::<EntityContent>();
    let onclick = dispatch.reduce_mut_callback(|content| content.content = "sdflkj".to_string());
    // let handle_username_change = dispatch.reduce_mut_callback_with(|content, event: KeyboardEvent| {
    //     let val: HtmlInputElement  = event.target_unchecked_into();
    //     content.content += val.v
    // });

    html! {

        <div>
            <h3>{"Create entity"}</h3>
            <div>
                // <textarea  onkeyup={handle_username_change} />
            </div>
            <div>{content.content.to_string()}</div>
            <button {onclick}>{"+1"}</button>
        </div>
    }
}
