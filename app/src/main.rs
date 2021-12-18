#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod components;

use anyhow::Result;
use components::temperature::Temperature;
use gloo::timers::callback::Interval;
use gloo_console::log;
use reqwasm::http;
use std::sync::{Arc, RwLock};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;


enum Message {
    Tick,
}

struct Model {
    state: Arc<RwLock<models::State>>,
    _interval: Interval,
}

async fn fetch_state() -> Result<models::State> {
    Ok(http::Request::get("http://0.0.0.0:3000/state")
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
            state: Arc::new(RwLock::new(models::State::default())),
            _interval: interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Tick => {
                let state = self.state.clone();

                spawn_local(async move {
                    match fetch_state().await {
                        Ok(new_state) => {
                            // TODO: we use std::sync::RwLock here which should lock everything but
                            // it does not ... strange
                            let mut state = state.write().unwrap();
                            *state = new_state;
                        }
                        Err(err) => {
                            log!("error: ", err.to_string());
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
        let state = self.state.clone().read().unwrap().clone();

        let (current, target) = if !state.serial_problem {
            (state.current_temperature, state.target_temperature)
        } else {
            (0.0, 0.0)
        };

        html! {
            <>
            <ybc::Container>
                <ybc::Columns>
                    <ybc::Column>
                        <Temperature temperature={current} emphasize=true/>
                        <Temperature temperature={target} emphasize=false/>
                    </ybc::Column>
                    <ybc::Column>
                        <ybc::Progress classes={classes!("is-primary")} max=100.0 value=50.0/>
                    </ybc::Column>
                </ybc::Columns>
            </ybc::Container>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
