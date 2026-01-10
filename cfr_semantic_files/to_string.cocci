@@
identifier i,j,k;
expression f,g;
@@

- f.map(|i| i.to_string()).j()
+ f.map(str::to_string).j()
