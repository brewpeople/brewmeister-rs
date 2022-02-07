use yew::{classes, function_component, html, Properties};

#[derive(Properties, PartialEq)]
pub struct TemperatureProps {
    pub emphasize: bool,
    pub temperature: Option<f32>,
}

#[function_component(Temperature)]
pub fn temperature(props: &TemperatureProps) -> Html {
    let class = if props.emphasize {
        vec!["emphasize"]
    } else {
        vec![]
    };

    match props.temperature {
        Some(temperature) => html! {
            <span class={classes!(class)}>{ &temperature.round() }{ "°C" }</span>
        },
        None => html! {
            <span class={classes!(class)}>{ "ERR" }</span>
        }
    }
}
