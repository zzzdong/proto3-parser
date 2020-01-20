use std::path::Path;

use pest::iterators::Pair;
use pest::Parser;

use crate::error::*;
use crate::model::*;

#[derive(Parser)]
#[grammar = "proto.pest"]
pub struct ProtoParser;

fn parse_option(pair: Pair<'_, Rule>) -> Result<ProtoOption> {
    let mut new_opt: ProtoOption = Default::default();

    for opt in pair.into_inner() {
        match opt.as_rule() {
            Rule::optionName => {
                new_opt.name = opt.as_str().to_string();
            }
            Rule::constant => {
                new_opt.value = opt.as_str().to_string();
            }
            _ => return Err(unexpect_token(opt)),
        }
    }

    Ok(new_opt)
}

fn parse_import(pair: Pair<'_, Rule>) -> Result<Import> {
    let mut new_import: Import = Default::default();

    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::weak => {
                new_import.import_type = ImportType::Weak;
            }
            Rule::public => {
                new_import.import_type = ImportType::Public;
            }
            Rule::strLit => {
                let s = part
                    .into_inner()
                    .nth(1)
                    .ok_or_else(|| token_not_found("inner_str"))?;
                new_import.proto_file = s.as_str().to_string();
            }
            _ => return Err(unexpect_token(part)),
        }
    }

    Ok(new_import)
}

fn parse_enum(pair: Pair<'_, Rule>) -> Result<Enum> {
    let mut new_enum: Enum = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::enumName => {
                new_enum.name = entry.as_str().to_string();
            }
            Rule::enumBody => {
                for part in entry.into_inner() {
                    match part.as_rule() {
                        Rule::option => {
                            let new_opt = parse_option(part)?;
                            new_enum.options.push(new_opt);
                        }
                        Rule::enumField => {
                            let mut new_field: EnumField = Default::default();
                            for field in part.into_inner() {
                                match field.as_rule() {
                                    Rule::ident => {
                                        new_field.name = field.as_str().to_string();
                                    }
                                    Rule::intLit => {
                                        new_field.value = field.as_str().parse()?;
                                    }
                                    Rule::enumValueOption => {
                                        let new_opt = parse_option(field)?;
                                        new_field.options.push(new_opt);
                                    }
                                    _ => return Err(unexpect_token(field)),
                                }
                            }
                            new_enum.fields.push(new_field);
                        }
                        Rule::emptyStatement => {}
                        _ => return Err(unexpect_token(part)),
                    }
                }
            }
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(new_enum)
}

fn parse_message_normal_field_type(pair: Pair<'_, Rule>) -> Result<NormalFieldType> {
    Ok(match pair.as_rule() {
        Rule::doubleType => NormalFieldType::Double,
        Rule::floatType => NormalFieldType::Float,
        Rule::int32Type => NormalFieldType::Int32,
        Rule::int64Type => NormalFieldType::Int64,
        Rule::uint32Type => NormalFieldType::Uint32,
        Rule::uint64Type => NormalFieldType::Uint64,
        Rule::sint32Type => NormalFieldType::Sint32,
        Rule::sint64Type => NormalFieldType::Sint64,
        Rule::fixed32Type => NormalFieldType::Fixed32,
        Rule::fixed64Type => NormalFieldType::Fixed64,
        Rule::sfixed32Type => NormalFieldType::Sfixed32,
        Rule::sfixed64Type => NormalFieldType::Sfixed64,
        Rule::boolType => NormalFieldType::Bool,
        Rule::stringType => NormalFieldType::Str,
        Rule::bytesType => NormalFieldType::Bytes,
        Rule::messageOrEnum => {
            let s = pair.as_str();
            NormalFieldType::MessageOrEnum(s.to_string())
        }
        // Rule::messageType => {
        //     let s = pair.as_str();
        //     NormalFieldType::Message(s.to_string())
        // }
        // Rule::enumType => {
        //     let s = pair.as_str();
        //     NormalFieldType::Enum(s.to_string())
        // }
        _ => return Err(unexpect_token(pair)),
    })
}

fn parse_message_normal_field(pair: Pair<'_, Rule>) -> Result<NormalField> {
    let mut new_field: NormalField = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::repeated => {
                new_field.repeated = true;
            }
            Rule::normalType => {
                let t = entry
                    .into_inner()
                    .next()
                    .ok_or_else(|| token_not_found("normalType"))?;
                new_field.field_type = parse_message_normal_field_type(t)?;
            }
            Rule::fieldName => {
                new_field.name = entry.as_str().to_string();
            }
            Rule::fieldNumber => new_field.number = entry.as_str().parse()?,
            Rule::fieldOptions => {
                let opt = parse_option(entry)?;
                new_field.options.push(opt);
            }
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(new_field)
}

fn parse_message_oneof_field(pair: Pair<'_, Rule>) -> Result<OneofField> {
    let mut new_field: OneofField = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::normalType => {
                let t = entry
                    .into_inner()
                    .next()
                    .ok_or_else(|| token_not_found("normalType"))?;
                new_field.field_type = parse_message_normal_field_type(t)?;
            }
            Rule::fieldName => {
                new_field.name = entry.as_str().to_string();
            }
            Rule::fieldNumber => {
                new_field.number = entry.as_str().parse()?;
            }
            Rule::fieldOptions => {
                let opt = parse_option(entry)?;
                new_field.options.push(opt);
            }
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(new_field)
}

fn parse_message_oneof_define(pair: Pair<'_, Rule>) -> Result<OneofDefine> {
    let mut new_oneof: OneofDefine = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::oneofName => {
                new_oneof.name = entry.as_str().to_string();
            }
            Rule::oneofField => {
                let field = parse_message_oneof_field(entry)?;
                new_oneof.fields.push(field);
            }
            Rule::emptyStatement => {}
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(new_oneof)
}

fn parse_message_map_field_key_type(pair: Pair<'_, Rule>) -> Result<MapFieldKeyType> {
    Ok(match pair.as_rule() {
        Rule::int32Type => MapFieldKeyType::Int32,
        Rule::int64Type => MapFieldKeyType::Int64,
        Rule::uint32Type => MapFieldKeyType::Uint32,
        Rule::uint64Type => MapFieldKeyType::Uint64,
        Rule::sint32Type => MapFieldKeyType::Sint32,
        Rule::sint64Type => MapFieldKeyType::Sint64,
        Rule::fixed32Type => MapFieldKeyType::Fixed32,
        Rule::fixed64Type => MapFieldKeyType::Fixed64,
        Rule::sfixed32Type => MapFieldKeyType::Sfixed32,
        Rule::sfixed64Type => MapFieldKeyType::Sfixed64,
        Rule::boolType => MapFieldKeyType::Bool,
        Rule::stringType => MapFieldKeyType::Str,
        _ => return Err(unexpect_token(pair)),
    })
}

fn parse_message_map_field(pair: Pair<'_, Rule>) -> Result<MapField> {
    let mut new_field: MapField = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::keyType => {
                let t = entry
                    .into_inner()
                    .next()
                    .ok_or_else(|| token_not_found("keyType"))?;
                new_field.key_type = parse_message_map_field_key_type(t)?;
            }
            Rule::normalType => {
                let t = entry
                    .into_inner()
                    .next()
                    .ok_or_else(|| token_not_found("normalType"))?;
                new_field.value_type = parse_message_normal_field_type(t)?;
            }
            Rule::mapName => {
                new_field.name = entry.as_str().to_string();
            }
            Rule::fieldNumber => {
                new_field.number = entry.as_str().parse()?;
            }
            Rule::fieldOptions => {
                let opt = parse_option(entry)?;
                new_field.options.push(opt);
            }
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(new_field)
}

fn parse_message(pair: Pair<'_, Rule>) -> Result<Message> {
    let mut new_message: Message = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::messageName => {
                new_message.name = entry.as_str().to_string();
            }
            Rule::messageBody => {
                for part in entry.into_inner() {
                    match part.as_rule() {
                        Rule::field => {
                            let field = parse_message_normal_field(part)?;
                            new_message.fields.push(MessageField::Normal(field));
                        }
                        Rule::mapField => {
                            let field = parse_message_map_field(part)?;
                            new_message.fields.push(MessageField::Map(field));
                        }
                        Rule::oneof => {
                            let oneof = parse_message_oneof_define(part)?;
                            new_message.fields.push(MessageField::Oneof(oneof));
                        }
                        Rule::Enum => {
                            let enum_def = parse_enum(part)?;
                            new_message.inner_enums.push(enum_def);
                        }
                        Rule::Message => {
                            let msg_def = parse_message(part)?;
                            new_message.inner_messages.push(msg_def);
                        }
                        Rule::option => {
                            let new_opt = parse_option(part)?;
                            new_message.options.push(new_opt);
                        }
                        Rule::emptyStatement => {}
                        _ => return Err(unexpect_token(part)),
                    }
                }
            }
            Rule::emptyStatement => {}
            _ => {}
        }
    }

    Ok(new_message)
}

fn parse_rpc(pair: Pair<'_, Rule>) -> Result<RPC> {
    let mut new_rpc: RPC = Default::default();

    let mut entry = pair.into_inner();
    let name = entry.next().ok_or_else(|| token_not_found("rpcName"))?;
    new_rpc.name = name.as_str().to_string();
    let req = entry.next().ok_or_else(|| token_not_found("messageType"))?;
    new_rpc.request = req.as_str().to_string();
    let resp = entry.next().ok_or_else(|| token_not_found("messageType"))?;
    new_rpc.response = resp.as_str().to_string();
    for e in entry {
        match e.as_rule() {
            Rule::option => {
                let new_opt = parse_option(e)?;
                new_rpc.options.push(new_opt);
            }
            Rule::emptyStatement => {}
            _ => return Err(unexpect_token(e)),
        }
    }

    Ok(new_rpc)
}

fn parse_service(pair: Pair<'_, Rule>) -> Result<Service> {
    let mut new_service: Service = Default::default();

    for entry in pair.into_inner() {
        match entry.as_rule() {
            Rule::serviceName => {
                new_service.name = entry.as_str().to_string();
            }
            Rule::option => {
                let new_opt = parse_option(entry)?;
                new_service.options.push(new_opt);
            }
            Rule::rpc => {
                let new_rpc = parse_rpc(entry)?;
                new_service.rpcs.push(new_rpc);
            }
            Rule::emptyStatement => {}
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(new_service)
}

pub fn parse_proto_file(filepath: impl AsRef<Path>) -> Result<ProtoFile> {
    let text = std::fs::read_to_string(filepath)?;

    parse_proto_text(text)
}

pub fn parse_proto_text(text: impl AsRef<str>) -> Result<ProtoFile> {
    let mut proto: ProtoFile = Default::default();
    let file = ProtoParser::parse(Rule::file, text.as_ref())?;

    for entry in file {
        match entry.as_rule() {
            Rule::syntax => {}
            Rule::import => {
                let import = parse_import(entry)?;
                proto.import.push(import);
            }
            Rule::package => {
                let package = entry
                    .into_inner()
                    .next()
                    .ok_or_else(|| token_not_found("packageName"))?;
                proto.package = package.as_str().to_string();
            }
            Rule::option => {
                let opt = parse_option(entry)?;
                proto.options.push(opt);
            }
            Rule::emptyStatement => {}
            Rule::topLevelDef => {
                for part in entry.into_inner() {
                    match part.as_rule() {
                        Rule::Message => {
                            let msg = parse_message(part)?;
                            proto.messages.push(msg);
                        }
                        Rule::Enum => {
                            let e = parse_enum(part)?;
                            proto.enums.push(e);
                        }
                        Rule::Service => {
                            let s = parse_service(part)?;
                            proto.services.push(s);
                        }
                        _ => return Err(unexpect_token(part)),
                    }
                }
            }
            Rule::EOI => {}
            _ => return Err(unexpect_token(entry)),
        }
    }

    Ok(proto)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let text = r#"syntax = "proto3";
                    package test;
                    import "other.proto";
                    option java_package = "com.example.foo";
                    enum AppMgrErrCode {
                        Success = 0;
                        DbFail = 32001;
                        MissingProtocolId = 32002;
                        CantChangeIdentifier = 32003;
                    };
                    enum EnumAllowingAlias {
                        option allow_alias = true;
                        UNKNOWN = 0;
                        STARTED = 1;
                        RUNNING = 2 [(custom_option) = "hello world"];
                    };
                    message outer {
                        option (my_option).a = true;
                        message inner {   // Level 2
                            int64 ival = 1;
                        }
                        repeated inner inner_message = 2;
                        EnumAllowingAlias enum_field =3;
                        map<int32, string> my_map = 4;
                        oneof oneof_data {
                            string domain = 5;
                            string ip = 6;
                        }
                    }

                    message HelloReq {
                        string msg = 1;
                    }

                    message HelloResp {
                        string msg = 1;
                    }

                    service hello {
                        rpc hello(HelloReq) returns (HelloResp) {}
                    }
                    "#;

        let proto = parse_proto_text(text).expect("parse proto text failed");

        println!("proto: {:?}", proto);
    }

    #[test]
    fn test_parse_dir_files() {
        let dir = std::fs::read_dir("./protos").expect("read_dir failed");
        for entry in dir {
            let entry = entry.expect("entry failed");
            match parse_proto_file(entry.path()) {
                Ok(proto) => println!("parse {:?} done, got:\n{:?}", entry.path(), proto),
                Err(e) => {
                    println!("parse {:?} failed, err: {:?}", entry.path(), e);
                }
            }
        }
    }
}
