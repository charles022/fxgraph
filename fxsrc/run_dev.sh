#!/bin/bash
set -e

# cleanup function
cleanup() {
    echo "Stopping processes..."
    kill $(jobs -p) 2>/dev/null
}
trap cleanup EXIT

if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed."
    echo "Please install it via: cargo install wasm-pack"
    echo "Or visit: https://rustwasm.github.io/wasm-pack/installer/"
    exit 1
fi

echo ">>> Building WASM Client..."
cd client
wasm-pack build --target web
cd ..

echo ">>> Installing Frontend Dependencies..."
cd www
npm install
cd ..

echo ">>> Starting Server..."
cargo run -p fxgraph-server &
SERVER_PID=$!

# Wait a bit for server to start
sleep 2

echo ">>> Starting Frontend..."
cd www
npm run dev &
FRONTEND_PID=$!

echo ">>> App is running!"
echo "    Server: 127.0.0.1:3000"
echo "    Web:    http://localhost:5173 (usually)"
echo "Press Ctrl+C to stop."

wait $SERVER_PID $FRONTEND_PID
