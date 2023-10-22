# just manual: https://github.com/casey/just/#readme

set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

_default:
    @just --list

# Build the web extension
build:
  cd background-script/ && wasm-pack build --release -t web
  cd options/ && wasm-pack build --release -t web
  cd popup/ && NODE_ENV=production tailwindcss -c ./tailwind.config.js -o ./tailwind.css --minify && wasm-pack build --release -t web