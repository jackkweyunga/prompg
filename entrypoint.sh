#!/bin/sh

# Exit immediately if a command exits with a non-zero status.
set -e

# This is a good place to add commands that need to run before the main application starts.
# For example:
# - Waiting for the database to be ready
# - Running database migrations
# - Seeding initial data

echo "Container entrypoint script is running..."
# --- Sanity checks before execution ---

# Check if a command was provided from the Dockerfile's CMD
if [ $# -eq 0 ]; then
    echo "ERROR: No command specified in Dockerfile CMD. Nothing to run." >&2
    exit 1
fi

CMD_PATH=$1
echo "  - Dockerfile CMD: '$@'"
echo "  - Checking binary path: '${CMD_PATH}'"

# Check if the binary exists
if [ ! -f "${CMD_PATH}" ]; then
    echo "ERROR: The specified command '${CMD_PATH}' does not exist or is not a file." >&2
    echo "--- Listing contents of /usr/local/bin/ for debugging ---" >&2
    ls -l /usr/local/bin/
    exit 1
fi

# Check if the binary is executable
if [ ! -x "${CMD_PATH}" ]; then
    echo "ERROR: The specified command '${CMD_PATH}' is not executable." >&2
    echo "--- Showing permissions for '${CMD_PATH}' ---" >&2
    ls -l "${CMD_PATH}"
    exit 1
fi

echo "✓ Checks passed. Handing over control to the main application..."
# --- End Sanity checks ---

# Execute the command passed as arguments to this script (the Dockerfile's CMD).
# The 'exec' command replaces the shell process with the new process,
# which is crucial for correct signal handling.
exec "$@"
