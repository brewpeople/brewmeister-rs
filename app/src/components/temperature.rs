use yew::{classes, function_component, html, Properties};

#[derive(Properties, PartialEq)]
pub struct TemperatureProps {
    pub emphasize: bool,
    pub temperature: f32,
}

#[function_component(Temperature)]
pub fn temperature(props: &TemperatureProps) -> Html {
    let class = if props.emphasize {
        vec!["emphasize"]
    } else {
        vec![]
    };

    html! { <span class={classes!(class)}>{ &props.temperature.round() }{ "°C" }</span> }
}
