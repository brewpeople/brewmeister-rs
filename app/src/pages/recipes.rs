use crate::Route;
use crate::components::recipes_list::RecipesList;
use reqwasm::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;

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
        <Link<Route> to={Route::NewBrew}>{ "Sud starten" }</Link<Route>>
        </>
    }
}
