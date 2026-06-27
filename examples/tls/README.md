# TLS Example

Note that this will NOT work unless an `identity.pfx` file is created.
You can create one using `openssl`:

```sh
$ openssl genrsa -out key.pem 2048 && \
    openssl req -new -x509 -key key.pem -out cert.pem -days 365 && \
    openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem

# You will get prompted with some personal info for security.
```

In this example, the password "1234" is used, but if you want to make a safe server,
consider getting one generated with a password generator.

If you test the server with `curl` using the recently self-signed certificate, you will need to run it with `-k`.

## WebSockets

As implemented, websockets also work with TLS. You can test it with `websocat` by running `websocat ws://localhost:3000/ws --insecure`. _(`--insecure` is needed to allow self-signed certificates, probably not needed in your production environment)_
