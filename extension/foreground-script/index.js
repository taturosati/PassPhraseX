// const wasm_url = chrome.runtime.getURL("foreground-script/pkg/foreground_script_bg.wasm");
// wasm_bindgen(wasm_url)
//     .then(module => module.start())
//     .catch(console.error);


function start() {
    console.debug("Starting");
    const port = connect();

    let username_value = null;
    let password_value = null;

    const on_change = function () {
        console.debug("Changed");

        const [username_input, password_input] = getInputs();
        if (!username_input && !password_input) {
            return;
        }

        console.debug("Found input");

        port.onMessage.addListener(function (msg) {
            console.debug("Received message", msg);
            if (msg.payload.Credential) {
                const { username, password } = msg.payload.Credential;

                if (username_input) {
                    username_input.value = username;
                }

                if (password_input) {
                    password_input.value = password;
                }
            } else {
                console.log("No credential found, looking for autosave");
                if (username_input) {
                    username_input.addEventListener("input", function (e) {
                        username_value = e.target.value;
                        console.log("Sending autosave", username_value, password_value);
                    });
                }

                if (password_input) {
                    password_input.addEventListener("input", function (e) {
                        password_value = e.target.value;
                        console.log("Sending autosave", username_value, password_value);
                    });
                }
            }
        });

        let site = getSite();
        const payload = {
            header: {},
            payload: {
                GetCredential: {
                    site: site,
                }
            }
        }

        port.postMessage(payload);

    };

    const mutation_observer = new MutationObserver(on_change);

    const config = {
        attributes: false,            // Observe changes to attributes
        childList: true,             // Observe additions or removals of child nodes
        subtree: true,               // Observe mutations in the entire subtree
        characterData: false,         // Observe changes to the data of text nodes
        attributeOldValue: false,    // Record the previous value of attributes
        characterDataOldValue: false // Record the previous value of text nodes
    };

    try {
        mutation_observer.observe(document.body, config);
        console.debug("Successfully started observer");
    } catch (e) {
        console.error("Failed to start observer", e);
    }

    on_change();
}

function getSite() {
    let href = window.location.href.replace("https://", "")
        .replace("http://", "")
        .replace("www.", "");

    let index = href.indexOf("/");
    if (index > 0) {
        href = href.substring(0, index);
    }

    return href;
}

function getInputs() {
    return [
        document.querySelector('input[name="email"], input[name="username"], input[type="email"], input[autocomplete="username"], input[autocomplete="email"]'),
        document.querySelector('input[type="password"], input[name="password"], input[autocomplete="current-password"]'),
    ];
}

function connect() {
    const connect_info = null;
    return chrome.runtime.connect(null, connect_info);
}

document.addEventListener("DOMContentLoaded", start);