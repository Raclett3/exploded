use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub x: f64,
    pub y: f64,
    pub font_size: String,
    #[prop_or_default]
    pub children: Children,
    pub onclick: Callback<web_sys::MouseEvent>,
}

#[function_component(Button)]
pub fn button(props: &Props) -> Html {
    let Props {
        x,
        y,
        font_size,
        children,
        onclick,
    } = props;

    let rect = use_state(|| html! {<></>});
    let text_ref = use_node_ref();

    let cloned_rect = rect.clone();
    use_effect_with_deps(move |text_ref| {
        if let Some(text) = text_ref.cast::<web_sys::SvgGraphicsElement>() {
            if let Ok(rect) = text.get_b_box() {
                let x = (rect.x() - 0.5).to_string();
                let y = rect.y().to_string();
                let width = (rect.width() + 1.).to_string();
                let height = rect.height().to_string();
                cloned_rect.set(html! {
                    <rect x={x} y={y} width={width} height={height} stroke="#FFF" stroke-width="0.02px" fill="#000" />
                })
            }
        }
        || ()
    }, text_ref.clone());

    html! {
        <>
            {(*rect).clone()}
            <text
                x={x.to_string()}
                y={y.to_string()}
                font-size={font_size.clone()}
                onclick={onclick}
                fill="#FFF"
                dominant-baseline="middle"
                text-anchor="middle"
                ref={text_ref}>
                {for children.iter()}
            </text>
        </>
    }
}
