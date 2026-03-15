use anyhow::{Result, anyhow};
use serde_json::{Map, Value};
use std::io::{BufRead, Seek};

use crate::bundle::read::*;
use crate::bundle::serialized::TypeTreeNode;

pub fn decode(nodes: &[TypeTreeNode], reader: &mut (impl BufRead + Seek)) -> Result<Value> {
    if nodes.is_empty() {
        return Ok(Value::Null);
    }

    decode_node(nodes, 0, reader)
}

fn direct_children(nodes: &[TypeTreeNode], parent: usize) -> Vec<usize> {
    let child_level = nodes[parent].level + 1;
    let mut out = Vec::new();
    let mut i = parent + 1;
    while i < nodes.len() && nodes[i].level > nodes[parent].level {
        if nodes[i].level == child_level {
            out.push(i);
        }

        i += 1;
    }

    out
}

pub fn decode_node(
    nodes: &[TypeTreeNode],
    idx: usize,
    reader: &mut (impl BufRead + Seek),
) -> Result<Value> {
    let node = &nodes[idx];
    let children = direct_children(nodes, idx);

    let value = match node.type_name.as_str() {
        "bool" | "char" | "SInt8" | "UInt8" => Value::Number(byte(reader)?.into()),
        "SInt16" | "UInt16" | "short" | "unsigned short" => Value::Number(i16_le(reader)?.into()),
        "int" | "SInt32" | "UInt32" | "unsigned int" | "Type*" => {
            Value::Number(i32_le(reader)?.into())
        }
        "SInt64" | "long long" | "UInt64" | "unsigned long long" | "FileSize" => {
            Value::Number(i64_le(reader)?.into())
        }
        "float" => {
            let val = f32_le(reader)? as f64;
            Value::Number(serde_json::Number::from_f64(val).unwrap_or(0.into()))
        }
        "double" => {
            let val = f64_le(reader)?;
            Value::Number(serde_json::Number::from_f64(val).unwrap_or(0.into()))
        }
        "string" => {
            let bytes = aligned_bytes(reader)?;
            Value::String(String::from_utf8_lossy(&bytes).into_owned())
        }
        "Array" => {
            let size_val = decode_node(nodes, children[0], reader)?;
            let count = size_val
                .as_i64()
                .ok_or_else(|| anyhow!("Array size is not an integer"))?
                as usize;
            let elem_idx = children[1];
            let mut arr = Vec::with_capacity(count);
            for _i in 0..count {
                let arr_val = decode_node(nodes, elem_idx, reader)?;
                arr.push(arr_val);
            }
            Value::Array(arr)
        }
        "vector" | "map" | "staticvector" => decode_node(nodes, children[0], reader)?,
        "TypelessData" => {
            let size = i32_le(reader)? as usize;
            let mut buf = vec![0u8; size];
            reader.read_exact(&mut buf)?;
            align4(reader, 0)?;
            Value::Array(buf.into_iter().map(|b| Value::Number(b.into())).collect())
        }
        _ => {
            let mut obj = Map::new();
            let mut total_size = 0;
            for child_idx in children {
                let field_name = nodes[child_idx].name.clone();
                total_size += nodes[child_idx].byte_size;
                let val = decode_node(nodes, child_idx, reader)?;
                obj.insert(field_name, val);
            }
            if total_size < node.byte_size && node.byte_size > 0 {
                return Err(anyhow!("Unsupported type: {:?}", node));
            }
            Value::Object(obj)
        }
    };

    if node.meta_flag & 0x4000 != 0 {
        align4(reader, 0)?;
    }

    Ok(value)
}
