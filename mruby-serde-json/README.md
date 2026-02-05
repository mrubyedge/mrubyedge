# mruby-serde-json
 
mruby-serde-json provides JSON serialization/deserialization for mruby/edge using serde_json.

 ## Features

This gem implements `JSON.load` and `JSON.dump` methods for mruby/edge.

Natural serialization/deserialization is supported for basic classes (Integer, Float, String, Array, Hash, TrueClass, FalseClass, NilClass).  For classes with `to_json` implementation, the gem uses that method for serialization (Otherwise panics).
