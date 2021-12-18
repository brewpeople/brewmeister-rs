use yew::{function_component, html, classes, Properties};

#[derive(Properties, PartialEq)]
pub struct TemperatureProps {
    pub emphasize: bool,
    pub temperature: f32,
}

#[function_component(Temperature)]
pub fn temperature(props: &TemperatureProps) -> Html {
    let class = if props.emphasize {
        vec!["has-text-weight-bold", "is-size-1"]
    }
    else {
        vec!["is-size-4"]
    };

    html! { <span class={classes!(class)}>{ &props.temperature.round() }{ "°C" }</span> }
}
