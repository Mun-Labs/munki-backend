#!/bin/bash

export PORT=3000
export DATABASE_URL=REPLACE_YOUR_KEY
export BIRDEYE_API_KEY=REPLACE_YOUR_KEY
export BIRDEYE_API_URL=REPLACE_YOUR_KEY
export MONI_API_KEY=REPLACE_YOUR_KEY

echo "Starting app with the following environment variables:"
echo "PORT: $PORT"
echo "DATABASE_URL: $DATABASE_URL"
echo "BIRDEYE_API_KEY: $BIRDEYE_API_KEY"
echo "BIRDEYE_API_URL: $BIRDEYE_API_URL"
echo "MONI_API_KEY: $MONI_API_KEY"

cargo run