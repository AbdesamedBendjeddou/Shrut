use yew::prelude::*;

#[derive(Properties, PartialEq, Default)]
pub struct Props {
    pub icon: Option<String>,
    pub text: Option<String>,
    pub alt: Option<String>,
    pub target: String,
}

#[function_component(Link)]
pub fn link(props: &Props) -> Html {
    html!(
        <a href={props.target.clone()}>
        if props.icon.is_some(){
            <img class="icon" src={props.icon.as_ref().unwrap().to_owned()}
            alt={props.alt.as_deref().unwrap_or_default().to_owned() }
             />
        }
        if props.text.is_some() {
            <span class="link-text">{props.text.as_ref().unwrap().to_owned()}</span>
        }

        </a>
    )
}
