use anyhow::{Result, anyhow};
use std::io::{BufRead, Seek, SeekFrom};
use std::path::Path;

use super::read::*;

#[derive(Debug)]
pub struct TypeTreeNode {
    pub level: u8,
    pub type_name: String,
    pub name: String,
    pub byte_size: i32,
    pub meta_flag: i32,
}

#[derive(Debug)]
pub struct SerializedType {
    pub class_id: i32,
    pub nodes: Vec<TypeTreeNode>,
}

pub struct ObjectData {
    pub unity_version: (u32, u32),
    pub path_id: i64,
    pub byte_start: i64,
    pub byte_size: u32,
    pub class_id: i32,
    pub type_id: usize,
}

pub struct SerializedFile {
    pub name: String,
    pub node_offset: u64,
    pub metadata_size: u32,
    pub file_size: i64,
    pub version: u32,
    pub data_offset: i64,
    pub endianness: u8,
    pub objects: Vec<ObjectData>,
    pub types: Vec<SerializedType>,
    pub externals: Vec<String>,
}

impl SerializedFile {
    pub fn parse(
        name: &str,
        reader: &mut (impl BufRead + Seek),
        offset: u64,
        _size: u64,
    ) -> Result<Self> {
        let name = name.to_string();
        let node_offset = offset;
        let objects = Vec::new();
        let mut types = Vec::new();
        let mut externals = Vec::new();

        reader.seek(SeekFrom::Start(offset))?;
        let metadata_size = u32_be(reader)?;
        let file_size = u32_be(reader)? as i64;
        let version = u32_be(reader)?;
        let data_offset = u32_be(reader)? as i64;
        if version < 9 {
            return Ok(Self {
                name,
                node_offset,
                metadata_size,
                file_size,
                version,
                data_offset,
                endianness: 0,
                objects,
                types,
                externals,
            });
        }

        let endianness = read_endian(reader)?;
        if version < 22 {
            return Ok(Self {
                name,
                node_offset,
                metadata_size,
                file_size,
                version,
                data_offset,
                endianness,
                objects,
                types,
                externals,
            });
        }

        let metadata_size = u32_be(reader)?;
        let file_size = i64_be(reader)?;
        let data_offset = i64_be(reader)?;
        reader.seek(SeekFrom::Current(8))?;

        let objects = Self::parse_objects(reader, &mut types)?;

        let script_types_count = i32_le(reader)?;
        for _ in 0..script_types_count {
            i32_le(reader)?;
            i64_le(reader)?;
        }

        let externals_count = i32_le(reader)?;
        for _ in 0..externals_count {
            let _asset_path = cstring(reader)?;
            let mut guid_buf = vec![0u8; 16];
            reader.read_exact(&mut guid_buf)?;
            let _external_type = i32_le(reader)?;
            let path_name = cstring(reader)?;
            let external_name = Path::new(&path_name)
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or(&path_name);
            externals.push(external_name.to_string());
        }

        Ok(Self {
            name,
            node_offset,
            metadata_size,
            file_size,
            version,
            data_offset,
            endianness,
            objects,
            types,
            externals,
        })
    }

    fn parse_objects(
        reader: &mut (impl BufRead + Seek),
        types: &mut Vec<SerializedType>,
    ) -> Result<Vec<ObjectData>> {
        let mut out = Vec::new();
        let base_pos = reader.stream_position()?;

        let unity_ver = cstring(reader)?;
        let unity_ver: Vec<u32> = unity_ver
            .split('.')
            .take(2)
            .flat_map(|v| v.parse::<u32>())
            .collect();
        let unity_version = (unity_ver[0], unity_ver[1]);
        let _target_platform = i32_le(reader)?;
        let enable_type_tree = byte(reader)? != 0;

        let type_count = i32_le(reader)?;
        for _ in 0..type_count {
            let class_id = i32_le(reader)?;
            let _is_stripped = byte(reader)?;
            let _script_type_index = i16_le(reader)?;
            if class_id == 114 {
                reader.seek(SeekFrom::Current(16))?;
            }
            reader.seek(SeekFrom::Current(16))?;

            let nodes = if enable_type_tree {
                let node_count = u32_le(reader)? as usize;
                let string_buffer_size = u32_le(reader)? as usize;

                let mut raw = Vec::with_capacity(node_count);
                for _ in 0..node_count {
                    let _ver = u16_le(reader)?;
                    let level = byte(reader)?;
                    let _flags = byte(reader)?;
                    let ts = u32_le(reader)?;
                    let ns = u32_le(reader)?;
                    let bsize = i32_le(reader)?;
                    let _idx = i32_le(reader)?;
                    let mflag = i32_le(reader)?;
                    let _hash = u64_le(reader)?;
                    raw.push((level, ts, ns, bsize, mflag));
                }

                let mut string_buf = vec![0u8; string_buffer_size];
                reader.read_exact(&mut string_buf)?;

                let dep_count = i32_le(reader)? as i64;
                reader.seek(SeekFrom::Current(dep_count * 4))?;

                raw.into_iter()
                    .map(|(level, ts, ns, byte_size, meta_flag)| TypeTreeNode {
                        level,
                        type_name: resolve_str(ts, &string_buf),
                        name: resolve_str(ns, &string_buf),
                        byte_size,
                        meta_flag,
                    })
                    .collect()
            } else {
                vec![]
            };

            types.push(SerializedType { class_id, nodes });
        }

        let object_count = i32_le(reader)?;
        for _ in 0..object_count {
            align4(reader, base_pos)?;
            let path_id = i64_le(reader)?;
            let byte_start = i64_le(reader)?;
            let byte_size = u32_le(reader)?;
            let type_id = i32_le(reader)?;
            let class_id = types
                .get(type_id as usize)
                .ok_or_else(|| {
                    anyhow!("type id {} out of bounds (total: {})", type_id, types.len())
                })?
                .to_owned()
                .class_id;

            out.push(ObjectData {
                unity_version,
                path_id,
                byte_start,
                byte_size,
                class_id,
                type_id: type_id as usize,
            })
        }

        Ok(out)
    }

    pub fn type_nodes(&self, type_id: usize) -> &[TypeTreeNode] {
        self.types
            .get(type_id)
            .map(|t| t.nodes.as_slice())
            .unwrap_or(&[])
    }
}

fn read_endian(reader: &mut impl BufRead) -> Result<u8> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn resolve_str(offset: u32, local_buf: &[u8]) -> String {
    if offset & 0x80000000 == 0 {
        let pos = offset as usize;
        let end = local_buf[pos..]
            .iter()
            .position(|&b| b == 0)
            .map(|i| pos + i)
            .unwrap_or(local_buf.len());
        String::from_utf8_lossy(&local_buf[pos..end]).into_owned()
    } else {
        common_string(offset & 0x7FFFFFFF).to_string()
    }
}

fn common_string(offset: u32) -> &'static str {
    match offset {
        0 => "AABB",
        5 => "AnimationClip",
        19 => "AnimationCurve",
        34 => "AnimationState",
        49 => "Array",
        55 => "Base",
        60 => "BitField",
        69 => "bitset",
        76 => "bool",
        81 => "char",
        86 => "ColorRGBA",
        96 => "Component",
        106 => "data",
        111 => "deque",
        117 => "double",
        124 => "dynamic_array",
        138 => "FastPropertyName",
        155 => "first",
        161 => "float",
        167 => "Font",
        172 => "GameObject",
        183 => "Generic Mono",
        196 => "GradientNEW",
        208 => "GUID",
        213 => "GUIStyle",
        222 => "int",
        226 => "list",
        231 => "long long",
        241 => "map",
        245 => "Matrix4x4f",
        256 => "MdFour",
        263 => "MonoBehaviour",
        277 => "MonoScript",
        288 => "m_ByteSize",
        299 => "m_Curve",
        307 => "m_EditorClassIdentifier",
        331 => "m_EditorHideFlags",
        349 => "m_Enabled",
        359 => "m_ExtensionPtr",
        374 => "m_GameObject",
        387 => "m_Index",
        395 => "m_IsArray",
        405 => "m_IsStatic",
        416 => "m_MetaFlag",
        427 => "m_Name",
        434 => "m_ObjectHideFlags",
        452 => "m_PrefabInternal",
        469 => "m_PrefabParentObject",
        490 => "m_Script",
        499 => "m_StaticEditorFlags",
        519 => "m_Type",
        526 => "m_Version",
        536 => "Object",
        543 => "pair",
        548 => "PPtr<Component>",
        564 => "PPtr<GameObject>",
        581 => "PPtr<Material>",
        596 => "PPtr<MonoBehaviour>",
        616 => "PPtr<MonoScript>",
        633 => "PPtr<Object>",
        646 => "PPtr<Prefab>",
        659 => "PPtr<Sprite>",
        672 => "PPtr<TextAsset>",
        688 => "PPtr<Texture>",
        702 => "PPtr<Texture2D>",
        718 => "PPtr<Transform>",
        734 => "Prefab",
        741 => "Quaternionf",
        753 => "Rectf",
        759 => "RectInt",
        767 => "RectOffset",
        778 => "second",
        785 => "set",
        789 => "short",
        795 => "size",
        800 => "SInt16",
        807 => "SInt32",
        814 => "SInt64",
        821 => "SInt8",
        827 => "staticvector",
        840 => "string",
        847 => "TextAsset",
        857 => "TextMesh",
        866 => "Texture",
        874 => "Texture2D",
        884 => "Transform",
        894 => "TypelessData",
        907 => "UInt16",
        914 => "UInt32",
        921 => "UInt64",
        928 => "UInt8",
        934 => "unsigned int",
        947 => "unsigned long long",
        966 => "unsigned short",
        981 => "vector",
        988 => "Vector2f",
        997 => "Vector3f",
        1006 => "Vector4f",
        1015 => "m_ScriptingClassIdentifier",
        1042 => "Gradient",
        1051 => "Type*",
        1057 => "int2_storage",
        1070 => "int3_storage",
        1083 => "BoundsInt",
        1093 => "m_CorrespondingSourceObject",
        1121 => "m_PrefabInstance",
        1138 => "m_PrefabAsset",
        1152 => "FileSize",
        1161 => "Hash128",
        _ => "",
    }
}
