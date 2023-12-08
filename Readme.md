# PassPhraseX
## Features
- Passwords are locally encrypted
- Passwords are synced through a public api (you can use your own)
- Use a separate password for each device
- Use seed phrase to login on new device

## How to use
### How to use the cli
- Download the latest release
  - Make sure to have cargo installed
  - Run `cargo install passphrasex`
- Run `passphrasex` to start the program
- Login with your seed phrase
  - If you don't have one, you can create one
    - `passphrasex register --device-pass <device password>`
  - If you have one, you can use it to login
    - `passphrasex login --device-pass <device password> --seed-phrase "<seed phrase>"`
- Add a new password
    - `passphrasex add --device-pass <device password> --site <site> --username <username> --password <password> `
- Get a password
  - `passphrasex get --device-pass <device password> --site <site> --username <username>`
- Edit a password
  - `passphrasex edit --device-pass <device password> --site <site> --username <username> --password <password>`
- Delete a password
  - `passphrasex delete --device-pass <device password> --site <site> --username <username>`
- Generate a new password
  - `passphrasex generate`

### How to use the Chrome extension
- Clone git repository
  - `git clone https://github.com/srosati/passphrasex`
- Build
  - Install just (https://github.com/casey/just)
  - Go into extension directory `cd passphrasex/extension`
  - Run `just build`
- Go to chrome://extensions
- Enable developer mode
- Click on load unpacked
- Select the extension directory

### How to use your own api
- CLI only
- Follow the instructions in the api directory
- Set the `API_URI` environment variable to your api url
  - `export API_URI=<your api url>`