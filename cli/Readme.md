# PassPhraseX
## A simple password manager
### Features
- Passwords are locally encrypted
- Passwords are synced through a public api (you can use your own)
- Use a separate password for each device
- Use seed phrase to login on new device

### How to use
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