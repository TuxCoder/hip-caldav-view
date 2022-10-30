use yew::prelude::*;

use caldav_viewer::CaldavViewer;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <CaldavViewer />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
