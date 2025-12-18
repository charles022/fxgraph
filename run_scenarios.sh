#!/bin/bash

# Ensure the template file exists
if [ ! -f tprompt.txt ]; then
    echo "Error: tprompt.txt not found."
    exit 1
fi

# Ensure the scenarios list exists
if [ ! -f scenarios_list.txt ]; then
    echo "Error: scenarios_list.txt not found."
    exit 1
fi

# Read the template into a variable
TEMPLATE=$(cat tprompt.txt)

# Set to true to exit after the first scenario for testing
TEST_MODE=false

# Loop through each line in the scenarios file
while IFS= read -r scenario; do
    # Skip empty lines
    if [ -z "$scenario" ]; then
        continue
    fi

    echo "Processing scenario: $scenario"

    # Replace the placeholder '______' with the current scenario text
    # We use awk for safer substitution to avoid issues with special characters in the scenario string
    PROMPT=$(echo "$TEMPLATE" | awk -v scen="$scenario" '{gsub("______", scen); print}')

    # Execute the codex command
    # We echo the command first for visibility
    echo "Executing: codex exec \"$PROMPT\""
    codex exec "$PROMPT"

    if [ "$TEST_MODE" = true ]; then
        echo "Test mode enabled: Exiting after first scenario."
        break
    fi

done < scenarios_list.txt

echo "All scenarios processed."

