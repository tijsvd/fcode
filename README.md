# Fcode

A serializer/deserializer pair for Rust's [Serde](https://serde.rs/) framework. Fcode serializes into a binary format,
allowing for some schema evolution. 

## API 

See [https://docs.rs/fcode].

## Rationale

My use case for writing this was a project with multiple applications communicating via TCP, at high throughput, with
reasonably tight latency requirements. The project went through the following phases:

* The prototype started out with [Bincode](https://docs.rs/bincode/). Bincode is very fast and simple, but it doesn't
  allow for any schema evolution.
  That's fine for a single binary that just needs to serialize things. However I want to be able to add fields and
  perform a rolling update. 

* Then I moved the whole project to protocol buffers, using [stepancheg's
  implementation](https://github.com/stepancheg/rust-protobuf). I disliked this implementation, mostly because every
  struct has some additional fields for bookkeeping; I ended up always creating with `..Default::default()`, making it
  easy to miss a newly added field somewhere. 

* I experimented with Google Flatbuffers. The idea of "zero-copy reading" sounds very nice, but I wanted to use the
  generated objects further in my code, store them in maps, push them through queues, etcetera. That's not possible with
  Flatbuffers (well, not with any of its implementations). So I ended up writing every object that I'd want to use again
  as a normal Rust struct, together with a manual conversion layer. So another serialization/deserialization step after
  all, which kind of defeats the purpose. Besides I didn't like the particular implementation, that would panic if
  something wasn't quite right, or do other insane things like blindly transmute enums. 

* Finally I ended up back at protocol buffers, with the [Prost implementation](https://github.com/danburkert/prost). I
  was reasonably happy with this. The generated structs are nice and clean, and it's easy to extend them further.

I still had some issues with the clean Prost-generated structures.

* Enum fields convert to `i32`, which is correct as enums may be extended with new values that the receiver doesn't know
  about. But it's not very ergonomic, as it requires another decoding step on every access. Also, I had some enums that
  would never be extended (buy/sell), and I'd rather just get a deserialization error. Serde deserializes enums to their
  proper type, and allows for unknown values with the `#[serde(other)]` attribute.

* Oneof fields have similar problems as enums. And the nesting of types is annoying, i.e. a oneof field can't be the
  toplevel message.

* The required/optional field story is annoying. In the proto2 syntax, optional vs required may be declared, and it
  works well. But according to Google, required fields may not be added later, and their implementations will actually
  complain about required fields not being present (Prost does the right thing, i.e. just initialize to 0/false/empty).
  The proto3 syntax doesn't differentiate between required and optional, but sometimes you want to have an explicit
  optional scalar and not an extra boolean; besides, Prost makes all nested structs options, which is also annoying at
  times. 

* In one case I embed blobs in a message, and zero-copy encoding/decoding would actually be beneficial in this case.
  With protobufs, I copy them into the message, and then again into the send buffer during serialization. With Serde
  (most formats) I can choose.

So eventually I just bit the bullet and wrote the serialization format that I wanted. I use Serde since it provides
these nice derive macros, and it's used commonly.

## Wire format

The wire format is partly constrained by Serde's serializer interface, and partly by the desired evolutions. It ends up
to be close to Google's protobuf, with a similar outcome in space.

Every value starts with a single tag byte. The lower 3 bits in the tag byte designate the wire type. Then, if the wire
type indicates that a varint should follow, the higher 5 bits in the tag byte are part of that varint (the least
significant 4 bits of the value, and a stop bit). As a result, booleans and integer values under 16 take a single byte
on the wire.

The possible wire types are:

|Value  |  Name     | Follow-up content                                         |
|------ |  -----    | ---------------------------------------------------       |
|0      |  integer  | remaining bits of varint                                  |
|1      |  fixed32  | 4 bytes little endian                                     |
|2      |  fixed64  | 8 bytes little endian                                     |
|3      |  sequence | varint length, followed by N individually encoded items   |
|4      |  bytes    | varint length, followed by N bytes                        |
|5      |  variant  | varint discriminator, followed by a single item           |
|6      |  reserved |                                                           |
|7      |  reserved |                                                           |

With this scheme, it is always possible to skip an item without knowing the Rust type. This is important for new fields
in structs and unknown enum variants.

All integers are encoded as varints. Signed integers are first encoded into unsigned integers using the zig-zag method
(same as protobufs), so sender and receiver must agree on the signed-ness. Boolean is encoded as integer 0 or 1, and
decoded as zero or non-zero. Unit types are encoded as integer 0, but the decoder just skips the field without checking
the wire type. The decoder also allows fixed32 and fixed64 wire types for 32-bit and 64-bit integers, respectively, for
the case where perhaps one day we can hint to serde that values must be encoded that way.

Except for this 5-bit extra field, varints are encoded the same as in protobufs, with 7 bits of information per byte, a
continuation bit as bit 7, least significant bits first. So e.g. the value 10042 (0b10011100111010) would be encoded as:

```
  11010000        11110011        00000100
  -               -               -
  |-> continue    |-> continue    |-> stop

   ----            -------         -------
    |-> bit 0-3      |-> bit 4-10    |-> bit 11-17

       ---
        |-> wire type 0 = integer

  -> D0 F3 04
```

Floating point types `f32` and `f64` are encoded as fixed32 and fixed64 little-endian values, same as protobufs.

Structs are encoded as sequences: field count followed by fields, in lexical order. The same format is used for tuples,
tuple structs, arrays, and real sequences (`Vec`, `VecDeque`), and hence all these types are interchangeable.

Maps are encoded as sequences of alternating keys and values. The length designates the total number of encoded values
(i.e. map length * 2).

Strings and blobs are encoded as byte count followed by content. The content is not encoded otherwise. Note that
serde-derive will normally serialize `Vec<u8>` and `&[u8]` as a sequence -- see the
[`serde_bytes`](https://docs.serde.rs/serde_bytes/) crate for details.

Enum values are encoded using a discriminator and the content. Content is always present, even in the case of a
unit variant. Note that when using serde-derive, the discriminator is (AFAIK) *not* the "enum value" as optionally set
in the code, but the lexical index of the variant. 

Finally, newtype structs and newtype variants (`Foo(i32)` and `MyEnum::Foo(i32)`) are encoded just as the inner value.
Therefore, single-item named tuples can't be extended, but any type can be upgraded to a newtype struct.

## Performance

Simple performance measurements indicate that fcode is slower than bincode, by a factor of about 2 (depending on types
used). It does seem to be significantly faster than protobufs (Prost implementation), and vastly faster than JSON. Wire
size is very similar to protobufs.

## Future work

Nothing concrete planned.

It would be great if at some point we can tell Serde (through some attribute) about fixed32 and fixed64
integers; varints are great as a general case, but some integers are just always large (ID's, nano posix timestamps) and
varint encoding is not efficient in that case. 

Similarly, it would be nice to pack scalar sequences together, especially on little endian machines where we could then
just reference the read buffer.

I'm pondering whether to write dedicated derive macros to solve this outside Serde. But then, that would open up so many
more possibilities that a whole different format may be more optimal.
