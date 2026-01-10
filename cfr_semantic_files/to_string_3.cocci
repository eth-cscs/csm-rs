@@
expression f;
@@

- f.unwrap().to_string()
+ f.map(str::to_string).unwrap()
