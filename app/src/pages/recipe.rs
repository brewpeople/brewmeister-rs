use crate::components;
use reqwasm::http::Request;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i64,
}

#[function_component(Recipe)]
pub fn recipe(Props { id }: &Props) -> Html {
    let recipe = use_state(models::Recipe::default);

    {
        let recipe = recipe.clone();
        let route = format!("/api/recipes/{id}");

        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    // TODO: proper error handling
                    let fetched: models::Recipe = Request::get(&route)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    recipe.set(fetched);
                });
                || ()
            },
            (),
        );
    }

    let id: models::RecipeId = (*id).into();

    let on_start = Callback::from(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::to_string(&models::NewBrew { id }).unwrap();

            let resp = Request::post("/api/brews")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .unwrap();

            log::info!("{:?}", resp);
        });
    });

    html! {
        <>
        <components::Recipe recipe={(*recipe).clone()} />
        <button onclick={on_start}>{"start"}</button>
        </>
    }
}
