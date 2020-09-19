# websocket experiment

Basic websocket 'echo' example using [tungstenite](https://crates.io/crates/tungstenite) that I've been playing around with, as a learning experience. It's a "one thread per client" model which should be fine for experimentation, prototyping or other small-scale use. 

Things it has over the usual minimal examples:
- TLS support using native-tls (though tungstenite looks pretty flexible)
- Custom error types using [thiserror](https://docs.rs/thiserror/1.0.20/thiserror/index.html) (maybe overkill for a simple project like this but I wanted to learn that too)
- Handles errors instead of `unwrap`/`expect` everywhere

I'm doing this with on Windows 10, no idea if it works on other platforms.

If you have .NET Core SDK installed there's an easy way to create and install a self-signed certificate for playing with this locally:

```
dotnet dev-certs https -ep ./test.pfx -p test --trust
```

There's other ways to do it (mostly using OpenSSL or PowerShell) but the above way was the easiest I found.