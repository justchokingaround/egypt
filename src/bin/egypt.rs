use egypt::{generate_adj_matrix_from_traces, generate_xes, parser::{parse_into_traces, variants_of_traces}};
use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{File, FileReader, HtmlAnchorElement, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

enum Msg {
    TextInput(String),
    XESImport(Option<File>),
    XESLoaded(Result<String, String>),
    ConvertToXES,
    DownloadXES,
    // ConvertToAdjMatrix,
}

struct App {
    text: String,
    processed: bool,
    file_reader_closure: Option<Closure<dyn FnMut(web_sys::ProgressEvent)>>, // store the closure
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            text: String::new(),
            processed: false,
            file_reader_closure: None, // initialize the closure storage
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TextInput(text) => {
                self.text = text;
                self.processed = false;
                true
            }
            Msg::XESImport(file_option) => {
                if let Some(file) = file_option {
                        let link = ctx.link().clone();
                        let reader = FileReader::new().unwrap_throw();
                        let reader_clone = reader.clone();

                        let onload = Closure::once(move |_event: web_sys::ProgressEvent| {
                            match reader_clone.result() {
                                Ok(result) => {
                                    match result.as_string() {
                                        Some(text) => link.send_message(Msg::XESLoaded(Ok(text))),
                                        None => link.send_message(Msg::XESLoaded(Err("Failed to convert file content to string".to_string())))
                                    }
                                },
                                Err(e) => link.send_message(Msg::XESLoaded(Err(format!("Error reading file: {:?}", e))))
                            }
                        });

                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));

                        // store the closure in self to keep it alive
                        self.file_reader_closure = Some(onload);

                        if let Err(_e) = reader.read_as_text(&file) {
                            self.text = "Error reading file".to_string();
                            return true;
                        }
                }
                false
            }
            Msg::XESLoaded(result) => {
                match result {
                    Ok(content) => {
                        let traces = parse_into_traces(None, Some(&content));
                        match traces {
                            Ok(traces) => {
                                let (adj_matrix, full_independences, pure_existences) = generate_adj_matrix_from_traces(traces.clone());
                                let traces_as_str: Vec<Vec<&str>> = traces
                                    .iter()
                                    .map(|trace| trace.iter().map(|s| s.as_str()).collect())
                                    .collect();
                                let variants = variants_of_traces(traces_as_str);
                                let max_variant_frequency = *variants.values().max().unwrap() as f64 / traces.len() as f64;
                                let variants_per_traces = variants.len() as f64 / traces.len() as f64;
                                self.text = format!(
                                    "{}\n\nFull Independences (-,-): {}\nPure Existences (-,x): {}\nMaximum frequence of variants / total #traces: {}\n#variants / total #traces: {}\n",adj_matrix,
                                    full_independences,
                                    pure_existences,
                                    max_variant_frequency,
                                    variants_per_traces
                                );
                            }
                            Err(e) => {
                                self.text = format!("Error parsing file: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.text = format!("Error loading file: {}", e);
                    }
                }
                true
            }
            // Msg::ConvertToAdjMatrix => {
            //     self.text = generate_adj_matrix(&self.text);
            //     true
            // }
            Msg::ConvertToXES => {
                self.text = generate_xes(&self.text);
                self.processed = true;
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

        let onxesimport = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                Msg::XESImport(Some(file))
            } else {
                Msg::XESImport(None)
            }
        });

        // let onmatrix = ctx.link().callback(|_| Msg::ConvertToAdjMatrix);
        let onprocess = ctx.link().callback(|_| Msg::ConvertToXES);
        let ondownload = ctx.link().callback(|_| Msg::DownloadXES);

        html! {
            <div style="height: 90vh; display: flex; flex-direction: column;">
                <textarea
                    value={self.text.clone()}
                    oninput={oninput}
                    placeholder="Enter your text here"
                    style="flex-grow: 1; width: 99%; background-color: #393939; color: white; padding: 10px; font-size: 16px; resize: none;"
                />
                <div style="display: flex; padding: 10px; justify-content: right;">
                    <input type="file" id="xes-file" accept=".xes" onchange={onxesimport} style="display: none;" />
                    <label for="xes-file" style="padding: 10px 20px; font-size: 16px; margin-right: 10px; background-color: #4CAF50; color: white; cursor: pointer; border-radius: 5px;">
                        {"Import XES"}
                    </label>
                    // <button onclick={onmatrix} style="padding: 10px 20px; font-size: 16px; margin-right: 10px;">
                    //     {"Convert To Adjacency Matrix"}
                    // </button>
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
