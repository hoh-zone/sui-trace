//! Lightweight in-memory representation of a Move module suitable for
//! pattern-matching rules. The full bytecode parse uses `move-binary-format`
//! upstream, but to keep the workspace dependency-light we currently work on
//! the Sui RPC's `disassembled` JSON output (`sui_getNormalizedMoveModule`).
//! That representation already exposes function visibility, parameters,
//! generic constraints and the kind of every called function, which is
//! sufficient for the V1 rule set.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleContext {
    pub package_id: String,
    pub version: u64,
    pub name: String,
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub friends: Vec<String>,
    /// Optional raw bytecode (hex). Rules that need bytecode can decode it.
    #[serde(default)]
    pub bytecode_hex: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub visibility: Visibility,
    pub is_entry: bool,
    pub parameters: Vec<TypeRef>,
    pub return_types: Vec<TypeRef>,
    /// Calls extracted from the bytecode disassembly. Populated by the engine
    /// before rules run.
    #[serde(default)]
    pub callees: Vec<Callee>,
    /// Free-form set of micro-tags the engine attaches while pre-processing
    /// (e.g. `uses_clock`, `has_loop`, `mutates_treasury`). Rules read these
    /// hints to keep their own logic small.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub abilities: Vec<String>,
    pub fields: Vec<(String, TypeRef)>,
    pub is_capability: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Public,
    PublicFriend,
    PublicPackage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeRef {
    pub type_name: String,
    pub generic: bool,
    pub is_mut_ref: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Callee {
    pub package_id: String,
    pub module: String,
    pub function: String,
}

impl FunctionInfo {
    pub fn calls(&self, module: &str, function: &str) -> bool {
        self.callees
            .iter()
            .any(|c| c.module == module && c.function == function)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}
