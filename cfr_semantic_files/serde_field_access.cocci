@@
expression f,g;
@@

- f[g].as_str().unwrap()
+ f.get(g).and_then(Value::as_str)
