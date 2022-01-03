use yew::{function_component, html, Callback, Properties};

#[derive(Properties, PartialEq)]
pub struct ThemeSwitchProps {
    pub checked: bool,
    pub on_click: Callback<bool>,
}

#[function_component(ThemeSwitch)]
pub fn temperature(props: &ThemeSwitchProps) -> Html {
    let checked = props.checked;

    let on_click = {
        let event = props.on_click.clone();
        Callback::from(move |_| event.emit(checked))
    };

    html! {
        <div class="toggle">
            <input class="toggle-input" type="checkbox" checked={checked} onclick={on_click} />
            <div class="toggle-bg"></div>
            <div class="toggle-switch">
                <div class="toggle-switch-figure"></div>
                <div class="toggle-switch-figureAlt"></div>
            </div>
        </div>
    }
}
