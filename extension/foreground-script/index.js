
function start() {
    const port = connect();
    const site = getSite();

    let added_listener_username = false;
    let added_listener_password = false;

    const on_change = function () {
        const [username_input, password_input] = getInputs();
        if (!username_input && !password_input) {
            return;
        }

        if (username_input && !added_listener_username) {
            added_listener_username = true;
            username_input.addEventListener(
                "input",
                (e) => setTmpCredentialUsername(port, site, e.target.value)
            );
        }

        if (password_input && !added_listener_password) {
            added_listener_password = true;
            password_input.addEventListener(
                "input",
                (e) => setTmpCredentialPassword(port, site, e.target.value)
            );
        }

        port.onMessage.addListener(function (msg) {
            if (msg.payload.Credential) {
                const { username, password } = msg.payload.Credential;

                if (username_input) {
                    username_input.value = username;
                }

                if (password_input) {
                    password_input.value = password;
                }
            }
        });

        getCredential(port, site);
    };

    const mutation_observer = new MutationObserver(on_change);

    const config = {
        attributes: false,              // Observe changes to attributes
        childList: true,                // Observe additions or removals of child nodes
        subtree: true,                  // Observe mutations in the entire subtree
        characterData: false,           // Observe changes to the data of text nodes
        attributeOldValue: false,       // Record the previous value of attributes
        characterDataOldValue: false    // Record the previous value of text nodes
    };

    window.onsubmit = function () {
        mutation_observer.disconnect();
        storeTmpCredential(port, site);
    }

    try {
        mutation_observer.observe(document.body, config);
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

function getCredential(port, site) {
    console.debug("Getting credential");
    const payload = {
        header: {},
        payload: {
            GetCredential: {
                site
            }
        }
    }

    port.postMessage(payload)
}

function setTmpCredentialUsername(port, site, username) {
    console.debug("Saving temp username", username);
    const payload = {
        header: {},
        payload: {
            SetTmpCredentialUsername: {
                site,
                username
            }
        }
    }

    port.postMessage(payload)
}

function setTmpCredentialPassword(port, site, password) {
    console.debug("Saving temp password", password);
    const payload = {
        header: {},
        payload: {
            SetTmpCredentialPassword: {
                site,
                password
            }
        }
    }
    port.postMessage(payload)
}

function storeTmpCredential(port, site) {
    console.debug("Saving temp credential");
    const payload = {
        header: {},
        payload: {
            StoreTmpCredential: {
                site
            }
        }
    }
    port.postMessage(payload)
}

document.addEventListener("DOMContentLoaded", start);