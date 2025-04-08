#!/bin/bash

# Navigate to the scheduler directory
cd scheduler

# Export environment variables
export ACCOUNT_API_KEY="apikeytest"
export DATABASE_URL="postgresql://postgres:postgres@localhost:8000/nettuscheduler"
export PORT="3000"
export ADMIN_STAGING_URL="https://senior-concierge-admin-staging.up.railway.app"
export ADMIN_PRODUCTION_URL="https://senior-concierge-admin-production.up.railway.app"

# Run the cargo command
cargo run