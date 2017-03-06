function ce()
{
    if [ "$#" -gt 0 ]; then
        local dest=$(cle $@ --cwd -d -r --sep //)
        if [ $? -eq 0 ]; then
            cd "${dest}"
        fi
    fi
}
