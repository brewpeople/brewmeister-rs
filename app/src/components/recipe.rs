use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub recipe: models::Recipe,
}

#[function_component(Recipe)]
pub fn recipe(Props { recipe }: &Props) -> Html {
    html! {
        <>
        <h1>{ recipe.name.clone() }</h1>
        <p>{ recipe.description.clone() }</p>
        </>
    }
}
