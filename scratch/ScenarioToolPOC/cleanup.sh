#!/bin/bash

# Loop through all subdirectories in the current directory
for dir in */ ; do
    # Check if it is purely a directory
    if [ -d "$dir" ]; then
        echo "Cleaning directory: $dir"
        # Find all files/directories inside the subdirectory (depth 1 only)
        # Exclude README.md
        # Execute rm -rf on everything else
        find "$dir" -mindepth 1 -maxdepth 1 -not -name 'README.md' -exec rm -rf {} +
    fi
done
