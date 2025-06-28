mod message;

#[proc_macro_derive(BincodeMessageDe)]
pub fn derive_bincode_byte_des(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::bincode::derive_bincode_byte_des(input)
}

#[proc_macro_derive(BincodeMessageSer)]
pub fn derive_bincode_byte_ser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::bincode::derive_bincode_byte_ser(input)
}

#[proc_macro_derive(JsonMessageDe)]
pub fn derive_serde_json_byte_des(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::serde_json::derive_serde_json_byte_des(input)
}

#[proc_macro_derive(JsonMessageSer)]
pub fn derive_serde_json_byte_ser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::serde_json::derive_serde_json_byte_ser(input)
}

#[proc_macro_derive(RkyvMessageDe)]
pub fn derive_rkyv_byte_des(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::rkyv::derive_rkyv_byte_des(input)
}

#[proc_macro_derive(RkyvMessageSer)]
pub fn derive_rkyv_byte_ser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::rkyv::derive_rkyv_byte_ser(input)
}

#[proc_macro_derive(ProstMessageDe)]
pub fn derive_prost_byte_des(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::prost::derive_proto_des(input)
}

#[proc_macro_derive(ProstMessageSer)]
pub fn derive_prost_byte_ser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    message::prost::derive_proto_ser(input)
}
