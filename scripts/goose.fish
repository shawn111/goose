function goose_preexec --on-event fish_preexec
    # Generate a session ID if one doesn't exist
    if not set -q GOOSE_SESSION_ID
        set -gx GOOSE_SESSION_ID (random uuid)
    end

    # Log the command to the goose history
    goose history add --session-id "$GOOSE_SESSION_ID" --command "$argv" --cwd "(pwd)"
end

function goose_magic
    # Get the last command from the history
    set -l last_command (history --max=1)

    # Explain the last command
    goose run --text "Explain the following shell command: $last_command"
end

# Bind Ctrl+G to the goose_magic function
