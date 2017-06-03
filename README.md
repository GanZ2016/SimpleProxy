# Simple Proxy - Team Zero

A simple SOCKS5 proxy server.
## Build on your local machine

<<<<<<< HEAD
- First, you need to install rustup nightly. Because we used some unstable features.
- Try to run following command on your command line in your local directory.
cargo build --release
- Next, for server part. Under target/release path. run following command
./server(server.exe for windows)  -l listen-address
- Clien part:
./client(client.exe for windows) -s server-address -c tunnel-count [-l listen-address]
---
=======
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
./server -s your_DO_ip:8080 -c 5 -l 127.0.0.1:1080
```

You can choose how many tunnels you want to create by changing the number after `-c`. You can also change the local listening address and port after `-l`.

Please check your ip if your get any error message.

#### Chrome/Firefox

- If you are using Chrome, download [SwitchyOmega](https://chrome.google.com/webstore/detail/proxy-switchyomega/padekgcemlokbadohgkifijomclgjgif?hl=en) and enable the socks5 proxy.
- If you are using Firefox, configure the socks5 proxy in settings using the local address `127.0.0.1:1080`.

----------
>>>>>>> 7c0fa12d78cca44b99546918d490464184d0673c
## Week 9 Update
**Socks5**

- Finished functions in socks5. All the messages from client could get correctly respond, and server will reply connection status to client. 
- So far our socks server support:
	- Protocal version: 5
	- Address type: IPV4 or DomainName
	- Connection method: No authentication required

**Timer**

- We create a concurrent timer to to let tunnel thread waiting for a certain amount of time.

**Client**

- Build a tunnel for a specified port and transfer message in this tunnel.
- Reference https://doc.rust-lang.org/std/sync/mpsc/
- The main function of client is to:
	- open port
	- close port
	- get server status
	- write message into stream
- The message in client tunnel includes:
```
enum Message {
    CSOpenport(u32, Sender<PortMessage>), 
    CSConnect(u32,Vec<u8>),
    CSShutdownWrite(u32),
    CSConnectDN(u32, Vec<u8>, u16),
    CSClosePort(u32),
    CSData(u32,Vec<u8>),

    SCHeartbeat,
    SCClosePort(u32),
    SCShutDownWrite(u32),
    SCConnectOk(u32,Vec<u8>),
    SCData(u32,Vec<u8>),

    PortDrop(u32)
}
```

**Server**

- Server part did the similar job as Client. It also has following functions:
	- open port
	- close port
	- get client status
	- write message into stream

### TODO

- There are still some bugs in client and server. We will fix them in this week.
- The data transmitted through the tunnel is unencrypted. Our Nice-To-Have part is to encrypt the data.
- We'll use a remote server to see how our program works.