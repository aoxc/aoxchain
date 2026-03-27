use dioxus::prelude::*;

#[component]
pub fn GlassSurface(
    children: Element,
    class: Option<String>,
    intensity: Option<&'static str>,
) -> Element {
    let blur = match intensity.unwrap_or("high") {
        "low" => "backdrop-blur-md",
        _ => "backdrop-blur-[30px]",
    };

    let class_name = class.unwrap_or_default();

    rsx! {
        div {
            class: "rounded-3xl border border-white/15 bg-white/5 {blur} shadow-[0_8px_30px_rgba(0,0,0,0.45)] {class_name}",
            {children}
        }
    }
}
