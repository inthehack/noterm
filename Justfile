#
# This file defines the rules that one can call from the `just` utility.
#
# Authors:
#   Julien Peeters <inthehack@mountainhacks.org>
#

set quiet := true

# Print this message.
help:
    just --list --unsorted

# Build the current workspace using the given PROFILE.
[group('build')]
build *OPTS:
    cargo build {{ OPTS }}

# Run test suite.
[group('quality')]
test *OPTS:
    cargo nextest run {{ OPTS }}

# Clean the cargo build artifacts.
[group('utility')]
clean:
    rm -rf target

# Wipe all non-versioned data.
[group("utility")]
mrproper:
    git clean -dffx
