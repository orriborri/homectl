#! /usr/bin/env nix-shell
#! nix-shell -i bash default.nix

cargo install trunk
cd frontend
npm install
trunk serve
