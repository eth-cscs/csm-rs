@@
expression f,g;
@@

- f[g].as_sequence()
+ f.get(g).and_then(Value::as_sequence)
