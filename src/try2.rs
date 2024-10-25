use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp;
use std::str::FromStr;
use tree_sitter::*;

// Simplified struct to hold AST elements for demonstration
#[derive(Default, Serialize, Deserialize, Debug)]
struct Thing {
    kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    text: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<Thing>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    relations: Vec<String>,
}

impl Thing {
    fn new(kind: Kind, text: String) -> Self {
        Thing {
            kind,
            name: None,
            text,
            children: Vec::new(),
            relations: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Kind {
    Root,
    Comment,
    Import,
    Struct,
    Enum,
    Derive,
    Function,
    Method,
    Field,
    Variable,
    Type,
    Trait,
    Impl,
    If,
    Else,
    Loop,
    Tuple,
    Array,
    FunctionCall,
    Undefined,
}

impl FromStr for Kind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "source_file" => Kind::Root,
            "line_comment" => Kind::Comment,
            "use_declaration" => Kind::Import,
            "struct_item" => Kind::Struct,
            "enum_item" => Kind::Enum,
            "attribute_item" => Kind::Derive,
            "function_item" => Kind::Function,
            "impl_item" => Kind::Impl,
            "field_declaration" => Kind::Field,
            "let_declaration" => Kind::Variable,
            "type_item" => Kind::Type,
            "trait_item" => Kind::Trait,
            "if_expression" => Kind::If,
            "else_clause" => Kind::Else,
            "loop_expression" => Kind::Loop,
            "tuple_expression" => Kind::Tuple,
            "array_expression" => Kind::Array,
            "call_expression" => Kind::FunctionCall,
            _ => Kind::Undefined,
        })
    }
}
impl Default for Kind {
    fn default() -> Self {
        Kind::Comment
    }
}
impl Kind {
    fn is_undefined(&self) -> bool {
        matches!(self, Kind::Undefined)
    }
}

struct ASTConversionService {
    code: String,
    tree: Tree,
}

impl ASTConversionService {
    fn new(code: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        let tree = parser.parse(&code, None).expect("Failed to parse code");
        ASTConversionService { code, tree }
    }
    fn generate_ast_with_relations(&self) -> String {
        let mut ast_root = Thing::new(Kind::Root, "Root".to_string());
        self.build_ast(self.tree.root_node(), &mut ast_root);
        // Convert AST to JSON
        let json_ast = json!(ast_root);
        serde_json::to_string_pretty(&json_ast).unwrap()
    }
    fn add_parent_name(name: &str, parent: &mut Thing) {
        if parent.name.is_none() {
            parent.name = Some(name.to_string());
        }
    }
    fn parent_namer(node_kind: &str, body: &str, parent: &mut Thing) {
        if node_kind == "type_identifier" || node_kind == "identifier" {
            Self::add_parent_name(&body, parent);
        }
    }
    fn build_ast(&self, node: Node, parent: &mut Thing) {
        let node_kind = node.kind().to_string();
        let body = self.node_text(node);
        Self::parent_namer(&node_kind, &body, parent);
        if let Ok(kind) = Kind::from_str(&node_kind) {
            let mut element = Thing::new(kind, body);
            for child in node.children(&mut node.walk()) {
                self.build_ast(child, &mut element);
            }
            if !element.kind.is_undefined() {
                element
                    .relations
                    .push(format!("{:?} -> {:?}", parent.kind, element.kind));
                parent.children.push(element);
            }
        }
    }
    fn node_text(&self, node: Node) -> String {
        let txt = self.code[node.byte_range()].to_string();
        txt[0..std::cmp::min(24, txt.len())].to_string()
    }
}

fn astring(a: Option<String>) -> String {
    a.unwrap_or("".to_string())
}

fn main() {
    let code =
        std::fs::read_to_string("src/try2.rs").expect("Failed to read the Rust source file.");

    let service = ASTConversionService::new(code);

    let ast_json = service.generate_ast_with_relations();
    println!("{}", ast_json);
}
