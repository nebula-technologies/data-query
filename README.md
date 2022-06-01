# Data Query

Data Query is a library that allows query any Serializable data.

## Usage
It currently supports only a few structures of querying.
 
 - Map Data eg 
   - `.some-key.some-other-key`
 - Generic Indexing of Array and Maps
   - `.some-array[0]` - getting key 0 in the array
   - `.some-array[0-2,6]` - getting key 0,1,2 and 6 
   - `.some-map[key1, key2]` - Treating the array as a map, and getting key1 and key2
 
> More will be added later, see TODO

### In code usage
To query the data the following can be used:

#### Using precompile_lex
`precompile_lex!` macro will build the lexical from the query string that has already been given.
Prebuilding the lexical operations reduces the amount of processing required

```rust 
let lex = precompile_lex!(.friends[1].name);
let data = User::default();
let query_res = query(data, lex);
println!("{:?}", query_res.unwrap());
```

#### Using compile
If the query is dynamically created, it might be better to just compile the lexical on the fly.

```rust
let lex = compile(".friends[1,2].name").unwrap();
let data = User::default();
let query_res = query(data, lex);
println!("{:?}", query_res.unwrap());
```

## Todo
At the moment there is only 1 todo because it very high on the list. 
 - Rewrite Lexical module to make it more dynamic and better handle tokens;
