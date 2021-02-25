#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Foo {
    #[prost(uint64, tag="1")]
    pub id: u64,
    #[prost(sint32, tag="2")]
    pub count: i32,
    #[prost(sint32, tag="3")]
    pub prefix: i32,
    #[prost(uint32, tag="4")]
    pub length: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bar {
    #[prost(message, optional, tag="1")]
    pub parent: ::core::option::Option<Foo>,
    #[prost(sint32, tag="2")]
    pub time: i32,
    #[prost(float, tag="3")]
    pub ratio: f32,
    #[prost(uint32, tag="4")]
    pub size: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FooBar {
    #[prost(message, optional, tag="1")]
    pub sibling: ::core::option::Option<Bar>,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(double, tag="3")]
    pub rating: f64,
    #[prost(uint32, tag="4")]
    pub postfix: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FooBarContainer {
    #[prost(message, repeated, tag="1")]
    pub list: ::prost::alloc::vec::Vec<FooBar>,
    #[prost(bool, tag="2")]
    pub initialized: bool,
    #[prost(enumeration="Enum", tag="3")]
    pub fruit: i32,
    #[prost(string, tag="4")]
    pub location: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Enum {
    Apples = 0,
    Pears = 1,
    Bananas = 2,
}
