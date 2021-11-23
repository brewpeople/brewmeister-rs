use anyhow::Result;
use gloo::timers::callback::Interval;
use gloo_console::log;
use yew::prelude::*;
use reqwasm::http;
use std::sync::{Arc, RwLock};
use wasm_bindgen_futures::spawn_local;

enum Message {
    Tick,
}

struct Model {
    _link: ComponentLink<Self>,
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

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cloned = link.clone();
        let interval = Interval::new(1000, move || cloned.send_message(Message::Tick));

        Self {
            _link: link,
            state: Arc::new(RwLock::new(models::State::default())),
            _interval: interval,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let state = self.state.clone();

        html! {
            <>
            <ybc::Container>
                <ybc::Navbar
                    navbrand=html!{}
                    navstart=html!{
                        <>
                        <ybc::NavbarItem classes=classes!("is-size-1", "has-text-weight-bold")>
                            { state.read().unwrap().current_temperature.round() }{"°C"}
                        </ybc::NavbarItem>
                        <ybc::NavbarItem classes=classes!("is-size-4")>
                            { state.read().unwrap().target_temperature.round() }{"°C"}
                        </ybc::NavbarItem>
                        </>
                    }
                    navend=html!{
                        <>
                        <ybc::NavbarItem classes=classes!("is-size-4")>
                            {"Heater "}{ if state.read().unwrap().heater_on { "on" } else { "off" }}
                        </ybc::NavbarItem>
                        <ybc::NavbarItem classes=classes!("is-size-4")>
                            {"Stirrer "}{ if state.read().unwrap().stirrer_on { "on" } else { "off" }}
                        </ybc::NavbarItem>
                        </>
                    }
                />
                <ybc::Columns>
                    <ybc::Column>
                        <ybc::Progress classes=classes!("is-primary") max=100.0 value=50.0/>
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
