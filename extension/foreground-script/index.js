const wasm_url = chrome.runtime.getURL("foreground-script/pkg/foreground_script_bg.wasm");
wasm_bindgen(wasm_url)
    .then(module => module.start())
    .catch(console.error);
