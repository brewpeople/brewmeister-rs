use crate::components::RecipesList;
use gloo_net::http::Request;
use yew::prelude::*;

#[function_component(Recipes)]
pub fn recipes() -> Html {
    let recipes = use_state(models::Recipes::default);

    {
        let recipes = recipes.clone();

        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    // TODO: proper error handling
                    let fetched: models::Recipes = Request::get("/api/recipes")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    recipes.set(fetched);
                });
                || ()
            },
            (),
        );
    }

    html! {
        <>
        <RecipesList recipes={(*recipes).clone()} />
        </>
    }
}
