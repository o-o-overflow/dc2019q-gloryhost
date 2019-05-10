#! /bin/bash

for ((i=0;i<10;i++)); do
    openssl enc -a -A </app/target/wasm32-unknown-wasi/release/gloryhost_interaction.wasm | ncat $1 $2 | grep -q 4f
    if [ $? -eq 0 ]; then
        echo "awesome!"
        exit 0
    fi
done

echo "good enough"
exit 0
