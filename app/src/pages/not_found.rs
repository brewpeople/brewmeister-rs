use yew::prelude::*;

pub struct NotFound;

impl Component for NotFound {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ybc::Columns>
                <ybc::Column>
                    <h1>{ "404" }</h1>
                </ybc::Column>
            </ybc::Columns>
        }
    }
}
