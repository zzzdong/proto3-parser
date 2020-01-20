#[derive(Debug, Clone)]
pub enum ImportType {
    Weak,
    Public,
}

impl Default for ImportType {
    fn default() -> ImportType {
        ImportType::Weak
    }
}

#[derive(Debug, Clone, Default)]
pub struct Import {
    pub import_type: ImportType,
    pub proto_file: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProtoOption {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct EnumField {
    pub name: String,
    pub value: i32,
    pub options: Vec<ProtoOption>,
}

#[derive(Debug, Clone, Default)]
pub struct Enum {
    pub name: String,
    pub options: Vec<ProtoOption>,
    pub fields: Vec<EnumField>,
}

#[derive(Debug, Clone)]
pub enum NormalFieldType {
    Double,
    Float,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    Str,
    Bytes,
    // Message(String),
    // Enum(String),
    MessageOrEnum(String),
    Invalid,
}

impl Default for NormalFieldType {
    fn default() -> NormalFieldType {
        NormalFieldType::Invalid
    }
}

#[derive(Debug, Clone, Default)]
pub struct NormalField {
    pub repeated: bool,
    pub field_type: NormalFieldType,
    pub name: String,
    pub number: u32,
    pub options: Vec<ProtoOption>,
}

#[derive(Debug, Clone, Default)]
pub struct OneofDefine {
    pub name: String,
    pub fields: Vec<OneofField>,
}

#[derive(Debug, Clone, Default)]
pub struct OneofField {
    pub name: String,
    pub field_type: NormalFieldType,
    pub number: u32,
    pub options: Vec<ProtoOption>,
}

#[derive(Debug, Clone)]
pub enum MapFieldKeyType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    Str,
    Invalid,
}

impl Default for MapFieldKeyType {
    fn default() -> MapFieldKeyType {
        MapFieldKeyType::Invalid
    }
}

#[derive(Debug, Clone, Default)]
pub struct MapField {
    pub name: String,
    pub key_type: MapFieldKeyType,
    pub value_type: NormalFieldType,
    pub number: u32,
    pub options: Vec<ProtoOption>,
}

#[derive(Debug, Clone)]
pub enum MessageField {
    Normal(NormalField),
    Oneof(OneofDefine),
    Map(MapField),
    Invalid,
}

impl Default for MessageField {
    fn default() -> MessageField {
        MessageField::Invalid
    }
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub name: String,
    pub inner_messages: Vec<Message>,
    pub inner_enums: Vec<Enum>,
    pub options: Vec<ProtoOption>,
    pub fields: Vec<MessageField>,
}

#[derive(Debug, Clone, Default)]
pub struct RPC {
    pub name: String,
    pub request: String,
    pub response: String,
    pub options: Vec<ProtoOption>,
}

#[derive(Debug, Clone, Default)]
pub struct Service {
    pub name: String,
    pub options: Vec<ProtoOption>,
    pub rpcs: Vec<RPC>,
}

#[derive(Debug, Clone, Default)]
pub struct ProtoFile {
    pub filename: String,
    pub package: String,
    pub import: Vec<Import>,
    pub options: Vec<ProtoOption>,
    pub enums: Vec<Enum>,
    pub messages: Vec<Message>,
    pub services: Vec<Service>,
}
