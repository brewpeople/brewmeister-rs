use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub recipes: models::Recipes,
}

#[function_component(RecipesList)]
pub fn recipes_list(Props { recipes }: &Props) -> Html {
    recipes.recipes.iter().map(|recipe| {
        html! {
            <>
            <h2>{ recipe.name.clone() }</h2>
            <p>{ recipe.description.clone() }</p>
            </>
        }
    }).collect()
}
