This module contains models generated from Jaeger's [thrift file](https://github.com/jaegertracing/jaeger-idl/blob/master/thrift/jaeger.thrift).

The following command will reproduce `thrift_models.rs` from scratch:

```bash
thrift --gen rs ./jaeger.thrift
```
