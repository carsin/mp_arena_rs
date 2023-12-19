# Stops all the child processes on interrupt
trap 'pkill -P $$; exit' SIGINT SIGTERM

cargo run -- -p 21100 -r 127.0.0.1:21101 -r 127.0.0.1:21102 -r 127.0.0.1:21103 &
cargo run -- -p 21101 -r 127.0.0.1:21100 -r 127.0.0.1:21102 -r 127.0.0.1:21103 &
cargo run -- -p 21102 -r 127.0.0.1:21100 -r 127.0.0.1:21101 -r 127.0.0.1:21103 &
cargo run -- -p 21103 -r 127.0.0.1:21100 -r 127.0.0.1:21101 -r 127.0.0.1:21102 &

wait