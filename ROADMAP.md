# Project roadmap & goals

## Roadmap

### 0. (always) Performance improvements

It's always possible to make it faster. And we should.

### 1. HTTP/2 support

Currently, the library only supports HTTP/1.1. HTTP/2 would be a great add-on to the library, and it wouldn't be too hard to implement (possibly with a different file that implements `Reqeust` and `Response`).

### 2. `WebSocket` without `tungstenite`

Currently, the library just uses `tungstenite` for ws support. It would be nice to have a native implementation of `WebSocket` that doesn't require a third-party library, which would also be easier to join with the rest of the library.

## Goals

### Simplicity

Current frameworks are a bit overkill for most projects. They require too much boilerplate and even proc macros for something as simple as a web server can be. This project aims to be a drop-in solution for most projects, with a minimal set-up and an easy API.

### Performance

Simplicity also comes with great performance. By reducing layers of abstraction, servers can be faster and easier to understand.

### Reliability

Servers written with this library should be strong agains tons of requests and connections. We want to be able to make something as robust as possible.
