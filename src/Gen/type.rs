use std::collections::HashMap;
use crate::import::*;

pub struct TypeRegistry {
    generated_types: HashMap<String, String>,
    pub ordered_definitions: Vec<String>,
    struct_definitions: HashMap<String, StructDefinition>,
    enum_definitions: HashMap<String, EnumDefinition>,
}

pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<(String, Option<Type>)>,
    pub is_public: bool,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            generated_types: HashMap::new(),
            ordered_definitions: Vec::new(),
            struct_definitions: HashMap::new(),
            enum_definitions: HashMap::new(),
        }
    }



    pub fn generate_hashmap_definition(&mut self, key_ty: &Type, val_ty: &Type, arch: &ArchConfig) -> Option<String> {
        let key_c_type = key_ty.to_c_type(arch, self);
        let val_c_type = val_ty.to_c_type(arch, self);
        
        let hashmap_name = format!("HashMap_{}_{}", 
            TypeRegistry::sanitize_type_name(&key_ty.name()), 
            TypeRegistry::sanitize_type_name(&val_ty.name()));
        
         
        let is_string_key = matches!(key_ty, Type::Str { .. } | Type::ConstStr);
        
        let hash_fn = if is_string_key {
            format!(r#"
static inline unsigned int hash_{HM}({KT} key) {{
    unsigned int hash = 5381;
    for (size_t i = 0; i < key.len && key.ptr[i]; i++) {{
        hash = ((hash << 5) + hash) + (unsigned char)key.ptr[i];
    }}
    return hash % 256;
}}"#, HM = hashmap_name, KT = key_c_type)
        } else {
            format!(r#"
static inline unsigned int hash_{HM}({KT} key) {{
    unsigned int hash = 0;
    unsigned char* ptr = (unsigned char*)&key;
    for (size_t i = 0; i < sizeof(key); i++) {{
        hash = hash * 31 + ptr[i];
    }}
    return hash % 256;
}}"#, HM = hashmap_name, KT = key_c_type)
        };
        
        let key_compare = if is_string_key {
            "(map->entries[index].key.len == key.len && memcmp(map->entries[index].key.ptr, key.ptr, key.len) == 0)".to_string()
        } else {
            "memcmp(&map->entries[index].key, &key, sizeof(key)) == 0".to_string()
        };
        
        Some(format!(
            r#"typedef struct {{
    struct {{
        {KT} key;
        {VT} value;
        bool occupied;
    }} entries[256];
    size_t size;
}} {HM};
{HASH_FN}

static inline void insert_{HM}({HM} *map, {KT} key, {VT} value) {{
    unsigned int index = hash_{HM}(key);
    unsigned int original = index;
    
    while (map->entries[index].occupied) {{
        if ({KEY_CMP}) {{
            map->entries[index].value = value;
            return;
        }}
        index = (index + 1) % 256;
        if (index == original) return;
    }}
    
    map->entries[index].key = key;
    map->entries[index].value = value;
    map->entries[index].occupied = true;
    map->size++;
}}

static inline {VT}* get_{HM}({HM} *map, {KT} key) {{
    unsigned int index = hash_{HM}(key);
    unsigned int original = index;
    
    while (map->entries[index].occupied) {{
        if ({KEY_CMP}) {{
            return &map->entries[index].value;
        }}
        index = (index + 1) % 256;
        if (index == original) break;
    }}
    
    return NULL;
}}
"#,
            KT = key_c_type,
            VT = val_c_type,
            HM = hashmap_name,
            HASH_FN = hash_fn,
            KEY_CMP = key_compare
        ))
    }

    pub fn register_enum(&mut self, name: String, variants: Vec<(String, Option<Type>)>, is_public: bool) {
        self.enum_definitions.insert(name.clone(), EnumDefinition { 
            name, 
            variants,
            is_public 
        });
    }
pub fn generate_enum_definition(&mut self, name: &str, arch: &ArchConfig) -> Option<String> {
    let enum_def = self.enum_definitions.get(name)?;
    let name = enum_def.name.clone();
    let variants = enum_def.variants.clone();
    
    for (_, variant_type) in &variants {
        if let Some(ty) = variant_type {
            match ty {
                Type::Tuple { fields } if fields.len() > 1 => {
                    self.generate_tuple_definition(fields, arch);
                }
                Type::Tuple { fields } if fields.len() == 1 => {
                    self.generate_type_definition(&fields[0], arch);
                }
                _ => {
                    self.generate_type_definition(ty, arch);
                }
            }
        }
    }
    
    let mut def = "typedef enum {\n".to_string();
    
    for (i, (variant_name, _)) in variants.iter().enumerate() {
        def.push_str(&format!("    {}__{} = {},\n", name, variant_name, i));
    }
    
    def.push_str(&format!("}} {}Tag;\n\n", name));
    def.push_str("typedef struct {\n");
    def.push_str(&format!("    {}Tag tag;\n", name));
    def.push_str("    union {\n");
    
    for (variant_name, variant_type) in &variants {
        if let Some(ty) = variant_type {
            let actual_type = match ty {
                Type::Tuple { fields } if fields.len() == 1 => &fields[0],
                _ => ty
            };
            
            let c_type = actual_type.to_c_type(arch, self);
            def.push_str(&format!("        {} {};\n", c_type, variant_name));
        }
    }
    
    def.push_str(&format!("    }} data;\n}} {};\n", name));
    
    Some(def)
}

    pub fn register_struct(&mut self, name: String, fields: Vec<(String, Type)>) {
        self.struct_definitions.insert(name.clone(), StructDefinition { name, fields });
    }
    
    pub fn get_struct_size(&self, name: &str, arch: &ArchConfig) -> Option<usize> {
        self.struct_definitions.get(name).map(|def| {
            def.fields.iter().map(|(_, ty)| ty.size_bits(arch).div_ceil(8)).sum()
        })
    }

    pub fn generate_std_str(&mut self) -> String {
        let type_id = "String".to_string();
        
        if self.generated_types.contains_key(&type_id) {
            return type_id;
        }
        
        let def = "typedef struct {\n    char* ptr;\n    size_t len;\n    size_t capacity;\n} String;\n".to_string();
        
        self.generated_types.insert(type_id.clone(), def.clone());
        self.ordered_definitions.push(def);

        type_id
    }

    pub fn generate_result_definition(&mut self, ok: &Type, err: &Type, arch: &ArchConfig) -> Option<String> {
        let ok_c = if ok.is_void() { "uint8_t".to_string() } else { ok.to_c_type(arch, self) };
        let err_c = if err.is_void() { "uint8_t".to_string() } else { err.to_c_type(arch, self) };
        
        let type_id = self.get_result_type_id(ok, err);
        
        if self.generated_types.contains_key(&type_id) {
            return None;
        }
        
        let def = format!(
            "typedef struct {{\n    uint8_t tag;\n    union {{\n        {} ok;\n        {} err;\n    }} data;\n}} {};\n",
            ok_c, err_c, type_id
        );
        
        self.generated_types.insert(type_id.clone(), def.clone());
        self.ordered_definitions.push(def.clone());
        Some(def)
    }

    pub fn generate_slices(&mut self, inner: &Type, arch: &ArchConfig) -> String {
        let type_id = format!("Slice_{}", TypeRegistry::sanitize_type_name(&inner.name()));
        
        if self.generated_types.contains_key(&type_id) {
            return type_id;
        }
        
        let inner_c = inner.to_c_type(arch, self);
        let def = format!(
            "typedef struct {{\n    {}* ptr;\n    size_t len;\n}} {};\n",
            inner_c, type_id
        );
        
        self.generated_types.insert(type_id.clone(), def.clone());
        self.ordered_definitions.push(def);
        type_id
    }

    pub fn get_slice_typedef(&self, inner: &Type) -> Option<String> {
        let type_id = format!("Slice_{}", TypeRegistry::sanitize_type_name(&inner.name()));
        self.generated_types.get(&type_id).cloned()
    }

    pub fn generate_option_definition(&mut self, inner: &Type, arch: &ArchConfig) -> Option<String> {
        let type_id = self.get_option_type_id(inner);
        
         
        if self.generated_types.contains_key(&type_id) {
            return None;
        }
        
        let inner_c = if inner.is_void() { "uint8_t".to_string() } else { inner.to_c_type(arch, self) };
        
        let def = format!(
            "typedef struct {{\n    uint8_t tag;\n    {} value;\n}} {};\n",
            inner_c, type_id
        );
        
        self.generated_types.insert(type_id.clone(), def.clone());
        self.ordered_definitions.push(def.clone());
        Some(def)
    }

     
    pub fn generate_tuple_definition(&mut self, fields: &[Type], arch: &ArchConfig) -> Option<String> {
        let type_id = self.get_tuple_type_id(fields);
        
         
        if self.generated_types.contains_key(&type_id) {
            return None;
        }
        
        let mut def = "typedef struct {\n".to_string();
        
        for (i, field) in fields.iter().enumerate() {
            let field_c = field.to_c_type(arch, self);
            def.push_str(&format!("    {} field_{};\n", field_c, i));
        }
        
        def.push_str(&format!("}} {};\n", type_id));
        
        self.generated_types.insert(type_id.clone(), def.clone());
        self.ordered_definitions.push(def.clone());
        Some(def)
    }

    pub fn get_union_type_name(&self, variants: &[Type], arch: &ArchConfig) -> String {
        let mut parts = Vec::new();
        
        for variant in variants {
            let part = match variant {
                Type::Int { bits, signed } => {
                    if *signed {
                        format!("int{}", bits)
                    } else {
                        format!("uint{}", bits)
                    }
                }
                Type::Float { bits } => format!("float{}", bits),
                Type::Bool => "bool".to_string(),
                Type::Str { .. } => "str".to_string(),
                Type::ConstStr => "cstr".to_string(),
                Type::Void => "void".to_string(),
                Type::Ptr(_) => "ptr".to_string(),
                Type::Struct { name } => name.clone(),
                _ => "unknown".to_string(),
            };
            parts.push(part);
        }
        
        format!("Union_{}", parts.join("_"))
    }
    
     
    pub fn generate_union_definition(&mut self, variants: &[Type], arch: &ArchConfig) -> String {
        let union_name = self.get_union_type_name(variants, arch);
        if self.generated_types.contains_key(&union_name) {
            return String::new();
        }

        let mut code = String::new();
        
        code.push_str(&format!("typedef struct {} {{\n", union_name));
        
        let tag_bits = arch.tag_bits_for_variants(variants.len());
        let tag_type = match tag_bits {
            8 => "uint8_t",
            16 => "uint16_t",
            32 => "uint32_t",
            _ => "uint64_t", 
        };
        code.push_str(&format!("    {} tag;\n", tag_type));
        
        code.push_str("    union {\n");
        
        for (idx, variant) in variants.iter().enumerate() {
            let c_type = variant.to_c_type(arch, self);
            code.push_str(&format!("        {} variant_{};\n", c_type, idx));
        }
        
        code.push_str("    } data;\n");
        code.push_str(&format!("}} {};\n\n", union_name));
        
        self.generated_types.insert(union_name.clone(), code.clone());
        
        code
    }

    pub fn ensure_union_defined(&mut self, variants: &[Type], arch: &ArchConfig) {
        let union_name = self.get_union_type_name(variants, arch);
        if !self.generated_types.contains_key(&union_name) {
            self.generate_union_definition(variants, arch);
        }
    }


    pub fn generate_type_definition(&mut self, ty: &Type, arch: &ArchConfig) -> Option<String> {
        match ty {
            Type::Option { inner } => self.generate_option_definition(inner, arch),
            Type::Result { ok, err } => self.generate_result_definition(ok, err, arch),
            Type::Tuple { fields } => self.generate_tuple_definition(fields, arch),
            Type::Union { variants } => {
                let type_id = self.get_union_type_id(variants);

                if self.generated_types.contains_key(&type_id) {
                    return None;
                }
                
                let tag_bits = arch.tag_bits_for_variants(variants.len());
                let tag_type = match tag_bits {
                    8 => "uint8_t",
                    16 => "uint16_t",
                    32 => "uint32_t",
                    _ => "uint64_t",
                };

                let mut def = format!("typedef struct {{\n    {} tag;\n    union {{\n", tag_type);

                for (i, variant) in variants.iter().enumerate() {
                    let variant_c = variant.to_c_type(arch, self);
                    def.push_str(&format!("        {} variant_{};\n", variant_c, i));
                }
                
                def.push_str(&format!("    }} data;\n}} {};\n", type_id));
                
                self.generated_types.insert(type_id.clone(), def.clone());
                Some(def)
            }
            
            _ => None,
        }
    }
    
    fn get_option_type_id(&self, inner: &Type) -> String {
        format!("Option_{}", TypeRegistry::sanitize_type_name(&inner.name()))
    }
    
    fn get_result_type_id(&self, ok: &Type, err: &Type) -> String {
        format!(
            "Result_{}_{}",
            TypeRegistry::sanitize_type_name(&ok.name()),
            TypeRegistry::sanitize_type_name(&err.name())
        )
    }
    
    fn get_union_type_id(&self, variants: &[Type]) -> String {
        let names: Vec<String> = variants.iter().map(|v| TypeRegistry::sanitize_type_name(&v.name())).collect();
        format!("Union_{}", names.join("_"))
    }
    
    fn get_tuple_type_id(&self, fields: &[Type]) -> String {
        let names: Vec<String> = fields.iter().map(|f| TypeRegistry::sanitize_type_name(&f.name())).collect();
        format!("Tuple_{}", names.join("_"))
    }
    
    pub fn sanitize_type_name(name: &str) -> String {
        let mut result = String::new();
        let mut last_was_underscore = false;
        for c in name.chars() {
            if c.is_alphanumeric() {
                result.push(c);
                last_was_underscore = false;
            } else if !last_was_underscore {
                result.push('_');
                last_was_underscore = true;
            }
        }
        result.trim_matches('_').to_string()
    }
}

 
impl Type {
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Type::Ptr(_) | Type::RawPtr(_) | Type::Owned(_) | Type::Ref(_) | Type::MutRef(_))
    }

    pub fn i8() -> Self { Self::Int { bits: 8, signed: true } }
    pub fn i16() -> Self { Self::Int { bits: 16, signed: true } }
    pub fn i32() -> Self { Self::Int { bits: 32, signed: true } }
    pub fn i64() -> Self { Self::Int { bits: 64, signed: true } }
    pub fn u8() -> Self { Self::Int { bits: 8, signed: false } }
    pub fn u16() -> Self { Self::Int { bits: 16, signed: false } }
    pub fn u32() -> Self { Self::Int { bits: 32, signed: false } }
    pub fn u64() -> Self { Self::Int { bits: 64, signed: false } }
    pub fn f32() -> Self { Self::Float { bits: 32 } }
    pub fn f64() -> Self { Self::Float { bits: 64 } }
    pub fn option(inner: Type) -> Self {Self::Option { inner: Box::new(inner) }}
    pub fn result(ok: Type, err: Type) -> Self {Self::Result { ok: Box::new(ok), err: Box::new(err) }}
    pub fn c_str(&self, arch: &ArchConfig, registry: &mut TypeRegistry) -> String { self.to_c_type(arch, registry) }
    pub fn char32() -> Self { Self::Char { bits: 32, signed: false } }
    pub fn char8() -> Self { Self::Char { bits: 8, signed: true } }
    pub fn int(bits: usize, signed: bool) -> Self {Self::Int { bits, signed }}
    pub fn float(bits: usize) -> Self {Self::Float { bits }}
    pub fn str_slice(char_type: Type, length_type: Type) -> Self {
        Self::StrSlice {
            char_type: Box::new(char_type),
            length_type: Box::new(length_type), 
        }
    }
     
    pub fn is_sized_array(&self) -> bool {
        matches!(self, Type::Array { size: Some(_), .. })
    }
    
    pub fn array_size(&self) -> Option<usize> {
        match self {
            Type::Array { size, .. } => *size,
            _ => None,
        }
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Type::Const(_))
    }
    
    pub fn unwrap_const(&self) -> &Type {
        match self {
            Type::Const(inner) => inner.as_ref(),
            other => other,
        }
    }
    
    
    pub fn make_const(self) -> Type {
        Type::Const(Box::new(self))
    }

    pub fn to_c_type(&self, arch: &ArchConfig, registry: &mut TypeRegistry) -> String {
        match self {
            Type::Int { bits, signed } => match (bits, signed) {
                (8, true) => "int8_t".to_string(),
                (16, true) => "int16_t".to_string(),
                (32, true) => "int32_t".to_string(),
                (64, true) => "int64_t".to_string(),
                (128, true) => "__int128".to_string(),
                (8, false) => "uint8_t".to_string(),
                (16, false) => "uint16_t".to_string(),
                (32, false) => "uint32_t".to_string(),
                (64, false) => "uint64_t".to_string(),
                (128, false) => "unsigned __int128".to_string(),
                (b, true) => format!("int{}_t", b),
                (b, false) => format!("uint{}_t", b),
            },

            Type::Float { bits } => match bits {
                32 => "float".to_string(),
                64 => "double".to_string(),
                128 => "long double".to_string(),
                _ => format!("_Float{}", bits),
            },
        
            Type::Char { bits, .. } => match bits {
                8 => "char".to_string(),
                32 => "uint32_t".to_string(),
                _ => format!("uint{}_t", bits),
            },
            
            Type::HashMap { key, value } => format!("HashMap_{}_{}", 
                TypeRegistry::sanitize_type_name(&key.name()), 
                TypeRegistry::sanitize_type_name(&value.name())),
            Type::SelfType => "Self".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Void => "void".to_string(),
            Type::Ptr(inner) => format!("{}*", inner.to_c_type(arch, registry)),
            Type::RawPtr(inner) => format!("{}*", inner.to_c_type(arch, registry)),
            Type::ConstStr => registry.generate_slices(&Type::char8(), arch),
            Type::Str { .. } => registry.generate_slices(&Type::char8(), arch),
            Type::StrSlice { char_type, .. } => registry.generate_slices(char_type, arch),
            Type::Struct { name } => name.clone(),
            Type::Array { element, .. } => registry.generate_slices(element, arch),
            Type::Intersection { types } => types.first()
                .map(|t| t.to_c_type(arch, registry))
                .unwrap_or_else(|| "void".to_string()),
            Type::TripleDot => "...".to_string(),
            Type::MultiArray { element, .. } => registry.generate_slices(element, arch),
            Type::Variadic => "...".to_string(),
            Type::Any => "void*".to_string(),
            Type::Trait => "void*".to_string(),
            Type::Owned(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                format!("{}*", inner.to_c_type(arch, registry))
            }
            Type::FnPtr { params, return_type } => {
                let param_types: Vec<String> = params.iter()
                    .map(|p| p.to_c_type(arch, registry))
                    .collect();
                format!("{} (*)({})", 
                    return_type.to_c_type(arch, registry), 
                    param_types.join(", "))
            },
            
            Type::Tuple { fields } => {
                let names: Vec<String> = fields.iter()
                    .map(|f| TypeRegistry::sanitize_type_name(&f.name()))
                    .collect();
                format!("Tuple_{}", names.join("_"))
            }
            
            Type::Union { variants } => {
                let variant_names: Vec<String> = variants.iter().map(|v| v.name().replace(" ", "_")).collect();
                format!("Union_{}", variant_names.join("_"))
            }
            
            Type::Option { inner } => {
                format!("Option_{}", TypeRegistry::sanitize_type_name(&inner.name()))
            }
            Type::Result { ok, err } => {
                format!("Result_{}_{}", 
                    TypeRegistry::sanitize_type_name(&ok.name()), 
                    TypeRegistry::sanitize_type_name(&err.name()))
            }
            Type::Const(innerg) => {
                format!("const {}", innerg.to_c_type(arch, registry))
            }
            
            Type::StdStr => "String".to_string(),
            
            Type::Auto => panic!("Auto type must be resolved before codegen"),
        }
    }

    pub fn size_bits(&self, arch: &ArchConfig) -> usize {
        match self {
            Type::ConstStr => arch.pointer_bits,
            Type::Const(inner) => inner.size_bits(arch),
            Type::Intersection { types } => {types.iter().map(|t| t.size_bits(arch)).max().unwrap_or(0)}
            Type::TripleDot => 0,
            Type::HashMap { key, value } => {
                let entry_size = key.size_bits(arch) + value.size_bits(arch) + 8;
                256 * entry_size + arch.pointer_bits
            }
            Type::Int { bits, .. } | Type::Float { bits } | Type::Char { bits, .. } => *bits,
            Type::Bool => 8,
            Type::Void | Type::Variadic => 0,
            Type::Ptr(_) | Type::RawPtr(_) | Type::FnPtr { .. } => arch.pointer_bits,
            Type::Str { len_type } | Type::StrSlice { length_type: len_type, .. } => {arch.pointer_bits + len_type.size_bits(arch)}
            Type::Struct { .. } => arch.pointer_bits,
            Type::Array { element, size: Some(size) } => element.size_bits(arch) * size,
            Type::Array { .. } => arch.pointer_bits + arch.pointer_bits,
            Type::Tuple { fields } => fields.iter().map(|f| f.size_bits(arch)).sum(),
            Type::Option { inner } => 8 + inner.size_bits(arch),
            Type::SelfType => 10,
            Type::Any | Type::Trait => arch.pointer_bits,
            Type::Owned(_) | Type::Ref(_) | Type::MutRef(_) => arch.pointer_bits,
            Type::StdStr => arch.pointer_bits * 2 + arch.pointer_bits,
            Type::Union { variants } => {
                let tag_bits = arch.tag_bits_for_variants(variants.len());
                let max_data = variants.iter().map(|v| v.size_bits(arch)).max().unwrap_or(0);
                tag_bits + max_data
            }
            Type::MultiArray { element, dimensions } => {
                let total: usize = dimensions.iter().product();
                element.size_bits(arch) * total
            }
            Type::Result { ok, err } => {
                8 + ok.size_bits(arch).max(err.size_bits(arch))
            }
            Type::Auto => 0,
        }
    }
    
    pub fn name(&self) -> String {
        match self {
            Type::StdStr => "String".to_string(),
            Type::Auto => "auto".to_string(),
            Type::HashMap { key, value } => {format!("HashMap_{}", value.name())}
            Type::ConstStr => "const str".to_string(),
            Type::Const(inner) => format!("const {}", inner.name()),
            Type::TripleDot => "...".to_string(),
            Type::Int { bits, signed: true } => format!("int{}", bits),
            Type::Int { bits, signed: false } => format!("uint{}", bits),
            Type::Float { bits } => format!("float{}", bits),
            Type::Bool => "bool".to_string(),
            Type::Char { bits: 8 , ..} => "char".to_string(),
            Type::Char { bits: 32, .. } => "char32".to_string(),
            Type::Char { bits, .. } => format!("char{}", bits),
            Type::Void => "void".to_string(),
            Type::Ptr(inner) => format!("*{}", inner.name()),
            Type::RawPtr(inner) => format!("^{}", inner.name()),
            Type::Str { .. } => "char8".to_string(),
            Type::StrSlice { .. } => "char8".to_string(),
            Type::Struct { name } => name.clone(),
            Type::Array { element, size: Some(s) } => format!("{}[{}]", element.name(), s),
            Type::Array { element, size: None } => format!("{}[]", element.name()),
            Type::SelfType => "Self".to_string(),
            Type::Option { inner } => format!("Option<{}>", inner.name()),
            Type::Result { ok, err } => format!("Result<{}, {}>", ok.name(), err.name()),
            Type::Variadic => "...".to_string(),
            Type::Any => "any".to_string(),
            Type::Trait => "trait".to_string(),
            Type::Owned(inner) => format!("~{}", inner.name()),
            Type::Ref(inner) => format!("&{}", inner.name()),
            Type::MutRef(inner) => format!("&mut {}", inner.name()),
            Type::Intersection { types } => {
                let names: Vec<String> = types.iter().map(|t| t.name()).collect();
                format!("({})", names.join(" & "))
            }
            Type::MultiArray { element, dimensions } => {
                let dims = dimensions.iter().map(|d| format!("[{}]", d)).collect::<String>();
                format!("{}{}", element.name(), dims)
            }
            Type::Tuple { fields } => {
                let names: Vec<String> = fields.iter().map(|f| f.name()).collect();
                format!("({})", names.join(", "))
            }
            Type::Union { variants } => {
                let names: Vec<String> = variants.iter().map(|v| v.name()).collect();
                format!("({})", names.join(" | "))
            }
            Type::FnPtr { params, return_type } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name()).collect();
                format!("fn({}) -> {}", param_names.join(", "), return_type.name())
            }
            Type::Auto => "auto".to_string(),
        }
    }

}