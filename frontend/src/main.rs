mod components;

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use components::entity_builder::EntityBuilder;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    EntityBuilder,
    #[at("/hello-server")]
    HelloServer,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::HelloServer => html! { <HelloServer /> },
        Route::EntityBuilder => html! { <EntityBuilder /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/entity").send().await.unwrap();
                    let result = {
                        if !resp.ok() {
                            Err(format!("Error fetching data {} ({})", resp.status(), resp.status_text()))
                        } else {
                            resp.text().await.map_err(|err| err.to_string())
                        }
                    };
                    data.set(Some(result));
                });
            }

            || {}
        });
    }

    match data.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div>{"Got server response: "}{data}</div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
