@@
identifier i;
@@

- let i = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;
+ let i = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
