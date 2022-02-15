use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub recipe: models::Recipe,
}

#[function_component(Recipe)]
pub fn recipe(Props { recipe }: &Props) -> Html {
    let steps = recipe
        .steps
        .iter()
        .map(|step| {
            html! {
                <p>{step.target_temperature} {"°C"} {format!("{:?}", step.duration)}</p>
            }
        })
        .collect::<Html>();

    html! {
        <>
        <h1>{ recipe.name.clone() }</h1>
        <p>{ recipe.description.clone() }</p>
        {steps}
        </>
    }
}
