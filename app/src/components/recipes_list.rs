use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub recipes: models::Recipes,
}

#[function_component(RecipesList)]
pub fn recipes_list(Props { recipes }: &Props) -> Html {
    recipes
        .recipes
        .iter()
        .map(|recipe| {
            html! {
                <>
                <Link<Route> to={Route::Recipe { id: recipe.id }}>{recipe.name.clone()}</Link<Route>>
                </>
            }
        })
        .collect()
}
