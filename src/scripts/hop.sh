function hop() {
    if [ "$#" -gt 0 ]; then
        local subcmd=$1
        shift 1
        local output=$(cle -hop "$subcmd" "$@")
        if [ $? -eq 0 ]; then
            local linecount=$(echo "$output" | wc -l)
            if [ "$linecount" -eq 1 ] && [ "$subcmd" == "to" ]; then
                cd "$output"
            else
                # Help message
                echo "$output"
            fi
        fi
    fi
}
