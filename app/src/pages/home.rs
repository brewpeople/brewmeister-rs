use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Home;

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <Link<Route> to={Route::Recipes}>{ "Recipes" }</Link<Route>>
        }
    }
}
