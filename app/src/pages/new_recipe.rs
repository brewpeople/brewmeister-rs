use crate::components::TextInput;
use yew::prelude::*;

#[derive(Clone, Default)]
struct Mash {
    duration: u32,
    temperature: u32,
}

#[derive(Clone)]
enum Step {
    Mash(Mash),
}

#[function_component(NewRecipe)]
pub fn new_recipe() -> Html {
    let name = use_state(String::default);
    let steps = use_state(Vec::<Step>::default);

    let cloned_name = name.clone();

    let on_change = Callback::from(move |value| {
        name.set(value);
    });

    let on_save = Callback::from(move |_| {
        log::info!("Storing {}", *cloned_name);
    });

    let cloned_steps = steps.clone();

    let on_new_step = Callback::from(move |_| {
        let mut new_steps = (*cloned_steps).clone();
        new_steps.push(Step::Mash(Mash {
            duration: 10,
            temperature: 30,
        }));
        cloned_steps.set(new_steps);
    });

    let rendered_steps = steps
        .iter()
        .map(|step| match step {
            Step::Mash(mash) => {
                let temperature = format!("{}", mash.temperature);
                let duration = format!("{}", mash.duration);

                html! {
                    <p>
                    <input type="number" min="20" max="100" value={temperature}/>
                    <input type="number" min="1" max="60" value={duration}/>
                    </p>
                }
            }
        })
        .collect::<Html>();

    html! {
        <>
        <TextInput on_change={on_change}/>
        {rendered_steps}
        <button onclick={on_new_step}>{"+"}</button>
        <button onclick={on_save}>{"Save"}</button>
        </>
    }
}
