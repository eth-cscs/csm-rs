@@
expression f,g;
@@

- f[g].as_str()
+ f.get(g).and_then(Value::as_str)
