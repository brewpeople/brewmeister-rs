use yew::prelude::*;

pub struct NewBrew;

impl Component for NewBrew {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <h1>{ "New Brew" }</h1>
        }
    }
}
