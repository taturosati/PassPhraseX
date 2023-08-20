use gloo_utils::format::JsValueSerdeExt;
use messages::{AppRequestPayload, AppResponsePayload, Request, Response};
use wasm_bindgen::JsValue;
use web_extensions_sys::chrome;

pub fn app_request<F>(payload: AppRequestPayload, callback: F)
where
    F: Fn(Result<AppResponsePayload, String>) + 'static,
{
    let msg = JsValue::from_serde(&Request::new(payload)).unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        match chrome().runtime().send_message(None, &msg, None).await {
            Ok(js_value) => {
                if js_value.is_object() {
                    let response: Result<AppResponsePayload, String> = js_value
                        .into_serde()
                        .map(|res: Response<AppResponsePayload>| res.payload)
                        .map_err(|err| format!("Unable to deserialize response: {:?}", err));

                    callback(response);
                } else {
                    callback(Err(
                        "The sender has unexpectedly not sent a reply".to_string()
                    ));
                }
            }
            Err(err) => {
                callback(Err(format!("Unable to send request: {:?}", err)));
            }
        };
    });
}
