# Simple Proxy - Team Zero

A simple SOCKS5 proxy server.
## Build on your local machine


----------

## Final version

### Usage

#### Server

- At first you need a remote server. The [digital ocean](http://https://m.do.co/c/a4c16f8bdb56)  server has reasonable price and friendly UI. 
   
- Pick up the region and which system you want to use. We recommend
   Cent OS. Log into your server and make sure you have root access.
   
- Open the port (usually 8080 or 8000) for connection :

```
nc -l -p 8080 
```
- Install rust with [rustup](https://www.rustup.rs/). Clone this repo and build with following command. It requires nightly features so [this](https://github.com/rust-lang-nursery/rustup.rs) might helpful.
```
cargo build --release
```
- Run the program :
```
./server -l your_DO_ip:8080
```
Please check your ip if your get any error message.

#### Client
- Install rust on your local machine. Clone this repo and build with:
```
cargo build --release
```
- Run the program :
```
./client -s your_DO_ip:8080 -c 5 -l 127.0.0.1:1080
```

You can choose how many tunnels you want to create by changing the number following `-c`. You can also change the local listening address and port following `-l`.

Please check your ip if your get any error message.

#### Chrome/Firefox

- For Chrome user: download [SwitchyOmega](https://chrome.google.com/webstore/detail/proxy-switchyomega/padekgcemlokbadohgkifijomclgjgif?hl=en) and enable the socks5 proxy.
- For Firefox user: configure the socks5 proxy in settings using the local address `127.0.0.1:1080`.

----------


### TODO

- The data transmitted through the tunnel is unencrypted. Our Nice-To-Have part is to encrypt the data.
