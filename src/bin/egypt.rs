use egypt::{generate_adj_matrix, generate_xes};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{HtmlAnchorElement, HtmlTextAreaElement};
use yew::prelude::*;

enum Msg {
    TextInput(String),
    ConvertToXES,
    DownloadXES,
    ConvertToAdjMatrix,
}

struct App {
    text: String,
    processed: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            text: String::new(),
            processed: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TextInput(text) => {
                self.text = text;
                self.processed = false;
                true
            }
            Msg::ConvertToXES => {
                self.text = generate_xes(&self.text);
                self.processed = true;
                true
            }
            Msg::ConvertToAdjMatrix => {
                self.text = generate_adj_matrix(&self.text);
                true
            }
            Msg::DownloadXES => {
                let window = web_sys::window().unwrap_throw();
                let document = window.document().unwrap_throw();

                let blob = web_sys::Blob::new_with_str_sequence_and_options(
                    &js_sys::Array::of1(&JsValue::from_str(&self.text)),
                    web_sys::BlobPropertyBag::new().type_("text/plain"),
                )
                .unwrap_throw();

                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_throw();

                let anchor: HtmlAnchorElement = document
                    .create_element("a")
                    .unwrap_throw()
                    .dyn_into()
                    .unwrap_throw();

                anchor.set_href(&url);
                anchor.set_download("event_log.xes");
                anchor.click();

                web_sys::Url::revoke_object_url(&url).unwrap_throw();

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            Msg::TextInput(input.value())
        });

        let onprocess = ctx.link().callback(|_| Msg::ConvertToXES);
        let ondownload = ctx.link().callback(|_| Msg::DownloadXES);

        let onmatrix = ctx.link().callback(|_| Msg::ConvertToAdjMatrix);

        html! {
            <div style="height: 90vh; display: flex; flex-direction: column;">
                <textarea
                    value={self.text.clone()}
                    oninput={oninput}
                    placeholder="Enter your text here"
                    style="flex-grow: 1; width: 99%; background-color: #393939; color: white; padding: 10px; font-size: 16px; resize: none;"
                />
                <div style="display: flex; padding: 10px; justify-content: right;">
                    <button onclick={onmatrix} style="padding: 10px 20px; font-size: 16px; margin-right: 10px;">
                        {"Convert To Adjacency Matrix"}
                    </button>
                    <button onclick={onprocess} disabled={self.processed} style="padding: 10px 20px; font-size: 16px; margin-right: 10px;">
                        {"Convert To XES"}
                    </button>
                    <button onclick={ondownload} disabled={!self.processed} style="padding: 10px 20px; font-size: 16px;">
                        {"Download XES"}
                    </button>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
