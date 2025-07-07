wget https://github.com/calcagebra/calcagebra/releases/latest/download/calcagebra
chmod +rwx ./calcagebra
cargo build --release
mv target/release/discord-bot-v2 .