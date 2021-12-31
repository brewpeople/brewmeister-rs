use yew::{function_component, html, Properties};

#[derive(Properties, PartialEq)]
pub struct ThemeSwitchProps {
    pub checked: bool,
}

#[function_component(ThemeSwitch)]
pub fn temperature(props: &ThemeSwitchProps) -> Html {
    html! {
        <div class="toggle">
            <input class="toggle-input" type="checkbox" checked={props.checked} />
            <div class="toggle-bg"></div>
            <div class="toggle-switch">
                <div class="toggle-switch-figure"></div>
                <div class="toggle-switch-figureAlt"></div>
            </div>
        </div>
    }
}
