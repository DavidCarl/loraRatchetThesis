#!/bin/bash

PASSWORD=""

OPTION=$1

SERVER_IP=192.168.43.192
CLIENT_IP=192.168.43.206

SERVER_USERNAME="pi"
CLIENT_USERNAME="pi"
SERVER_PASSWORD=$PASSWORD
CLIENT_PASSWORD=$PASSWORD

function server(){
    cd server || exit
    ~/.cargo/bin/cross build --target arm-unknown-linux-gnueabihf
    sshpass -p "$SERVER_PASSWORD" scp target/arm-unknown-linux-gnueabihf/debug/rasp_lora_server $SERVER_USERNAME@$SERVER_IP:/home/$SERVER_USERNAME/
    cd ..
}

function client(){
    cd client || exit
    ~/.cargo/bin/cross build --target arm-unknown-linux-gnueabihf
    sshpass -p "$CLIENT_PASSWORD" scp target/arm-unknown-linux-gnueabihf/debug/rasp_lora_client $CLIENT_USERNAME@$CLIENT_IP:/home/$CLIENT_USERNAME/
    cd ..
}

if [[ "$OPTION" == "both" || "$OPTION" == "" ]]; then
    echo "Building and uploading both"
    client
    server

elif [[ "$OPTION" == "client" ]]; then
    echo "Building and uploading client"
    client

elif [[ "$OPTION" == "server" ]]; then
    echo "Building and uploading server"
    server

else
    echo "Misisng options"
    echo "use: 'both' for client and server"
    echo "use: 'server' for server"
    echo "use: 'client' for client"
fi
