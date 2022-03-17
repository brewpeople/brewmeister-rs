#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod components;
mod pages;

use anyhow::Result;
use components::Header;
use gloo::timers::callback::Interval;
use gloo_net::http;
use log::error;
use std::sync::{Arc, RwLock};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/recipes/:id")]
    Recipe { id: models::RecipeId },
    #[at("/recipes")]
    Recipes,
    #[at("/recipes/new")]
    NewRecipe,
    #[not_found]
    #[at("/404")]
    NotFound,
}

enum Message {
    Tick,
}

struct Model {
    device: Arc<RwLock<models::Device>>,
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
                            error!("error: {err}");
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
        let device = self.device.clone().read().unwrap().clone();

        html! {
            <div>
                <Header device={device}/>
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
        Route::NewRecipe => html! { <pages::NewRecipe/> },
        Route::NotFound => html! { <pages::NotFound/> },
        Route::Recipes => html! { <pages::Recipes/> },
        Route::Recipe { id } => html! { <pages::Recipe id={id.0} /> },
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
