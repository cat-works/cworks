# Cat Framework/Workspace Notes

## CWorks URI

```js
/([^:]+):\/\/(([^@]+@)?)([^:]+(:[?\d]+))([^\?]+)((\?[^\?]+)?)/
```

```plain
<scheme>://[<connection arg>@]<host>[:<port>]/<path>?<request argument>
```

### Schemes

- http
- https
- cworks
- cftp
- ...

## CWorks domains

- local
  - ...
- group
  - <group-name>
    - ...
- usr
  - <user-id>
    - ...
- me: config

## Protocols

### CWorks Protocol

```plain
string: <length: uint16_t><str: char[length]>
```

#### Handshake

- `Server -> Client` `number[]`
  - version = 01
- `Client -> Server` `<version>`
  - version = 00: No Versions are supported (terminate)
  - version = ff: Latest

#### After Handshake

- `Server --> Client` `{event:"connection",server:u32,client:u32,client:string}`
  - Notify connection.
  - 00 -> Allow
  - ff -> Deny
- `Client --> Server` `{event:"connect",host:string}`
  - Connect to host.
  - `00 <c_socket: uint32_t>` -> Success
  - `<error: errno>` -> Failed

- `Client <-> Server` `{event:"disconnect",socket:u32}`

- `Client <-> Server` `{event:"send",}`: Send data

- `Client --> Server` `{event:"listen",host:string}`
  - Set host:port as connectable
  - `00: <s_sock: uint32_t>`: Successed
  - `<error: errno>`: Failed

#### Example

##### Simple server

```plain
C->S: 10 "codes.wiiu.~syoch.usr" 0050[80]
S->C: 00 00 00000000 ""

S->C: 02 00000000 00000001 'Wii U User'
C->S: 00

S->C: 00 00000001 b'{"url": "/code", query="combo=0x40"}'
C->S: 00 00000001 '{"response": "0: OK", 'data': {"fmt": "PCode1", "code": "["
      "[0x10000000]+0x10] = 1 as 8bit"}}'

S->C: 03 00000001
C->S: 00
```

## CCFW API

### CCFW Token

Header の X-Token
abcdefghABCDEFGH みたいな形式（[\w0-9-_]{16}）

### CCFW REST API

- /ccfw
  - /api
    - /serve: WebSocket: Serving Protocol
    - /cworks-proto: WebSocket Client Protocol
    - /auth/salt: POST "id=..."
    - /auth/login: POST "id=...&salt=...&hash=<SHA512>"
  - spec.json: `{"features": ["serve", "auth"]}`

## Cat Script

### Example

```plain
# Copy cworks://skins.wiiu.org/0.pck (cftp://192.168.3.16/.../DLC/{pack}/skins.pck|pck.pck)
pck = Fetch cworks://skins.wiiu.org/0.pck cache='auto update'
for (fname, file) in pck.Keys() {
  Copy file cworks://me/workspace/0.pck/{fname}
}
```