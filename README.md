# Simple Proxy - Team Zero

A simple SOCKS5 proxy server.

---
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