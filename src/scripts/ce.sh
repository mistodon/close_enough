function ce()
{
    if [ "$#" -gt 0 ]; then
        local dest=$(cle -ce "$@")
        if [ $? -eq 0 ]; then
            cd "${dest}"
        fi
    fi
}
