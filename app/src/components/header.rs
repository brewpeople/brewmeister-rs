use yew::prelude::*;
use crate::components::Temperature;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub device: models::Device,
}

#[function_component(Header)]
pub fn header(Props { device }: &Props) -> Html {

    html! {
        <header class="header">
            <div class="center">
                <Temperature temperature={device.current_temperature} emphasize=true/>
                <Temperature temperature={device.target_temperature} emphasize=false/>
            </div>
        </header>
    }
}
