#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod components;
mod pages;

use anyhow::Result;
use components::Temperature;
use components::ThemeSwitch;
use gloo::timers::callback::Interval;
use log::error;
use reqwasm::http;
use std::sync::{Arc, RwLock};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/recipe/:id")]
    Recipe { id: i64 },
    #[at("/recipes")]
    Recipes,
    #[at("/new")]
    NewBrew,
    #[not_found]
    #[at("/404")]
    NotFound,
}

enum Message {
    Tick,
}

struct Model {
    device: Arc<RwLock<models::Device>>,
    dark_mode: Arc<RwLock<bool>>,
    _interval: Interval,
}

async fn fetch_state() -> Result<models::Device> {
    Ok(http::Request::get("http://0.0.0.0:3000/api/state")
        .send()
        .await?
        .json()
        .await?)
}

impl Component for Model {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let interval = Interval::new(1000, move || link.send_message(Message::Tick));

        Self {
            device: Arc::new(RwLock::new(models::Device::default())),
            dark_mode: Arc::new(RwLock::new(false)),
            _interval: interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Tick => {
                let device = self.device.clone();

                spawn_local(async move {
                    match fetch_state().await {
                        Ok(new_state) => {
                            // TODO: we use std::sync::RwLock here which should lock everything but
                            // it does not ... strange
                            let mut device = device.write().unwrap();
                            *device = new_state;
                        }
                        Err(err) => {
                            error!("error: {}", err.to_string());
                        }
                    }
                });

                // this basically says update immediately, we need a different way to notify the
                // view to update ...
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let state = self.device.clone().read().unwrap().clone();

        let (current, target) = if !state.serial_problem {
            (state.current_temperature, state.target_temperature)
        } else {
            (None, None)
        };

        let on_theme_switch = {
            let dark_mode = self.dark_mode.clone();

            Callback::from(move |checked: bool| {
                *dark_mode.write().unwrap() = !checked;

                let attribute = if checked { "dark" } else { "light" };

                gloo_utils::document()
                    .document_element()
                    .unwrap()
                    .set_attribute("data-theme", attribute)
                    .unwrap();
            })
        };

        let dark_mode = *self.dark_mode.clone().read().unwrap();

        html! {
            <div>
                <header class="header">
                    <div class="center">
                        <Temperature temperature={current} emphasize=true/>
                        <Temperature temperature={target} emphasize=false/>
                        <ThemeSwitch checked={dark_mode} on_click={on_theme_switch}/>
                    </div>
                </header>
                <main class="center">
                    <BrowserRouter>
                        <Switch<Route> render={Switch::render(switch)} />
                    </BrowserRouter>
                </main>
            </div>
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <pages::Home/> },
        Route::NewBrew => html! { <pages::NewBrew/> },
        Route::NotFound => html! { <pages::NotFound/> },
        Route::Recipes => html! { <pages::Recipes/> },
        Route::Recipe { id } => html! { <pages::Recipe id={id.clone()} /> },
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
