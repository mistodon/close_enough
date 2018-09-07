function ce() {
    if [ "$#" -gt 0 ]; then
        local dest=$(cle -ce "$@")
        if [ $? -eq 0 ]; then
            local linecount=$(echo "$dest" | wc -l)
            if [ "$linecount" -eq 1 ]; then
                cd "$dest"
            else
                # Help message
                echo "$dest"
            fi
        fi
    fi
}
