console.debug('Load WASM popup module..');
import wasm_bindgen from './pkg/popup.js';
wasm_bindgen("./pkg/popup_bg.wasm")
  .then(module => module.start())
  .catch(console.error);
