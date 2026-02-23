//! Pretty printing utilities for the Fig Abstract Syntax Tree
//!
//! This module provides detailed visualization of AST structures for debugging purposes.

use crate::ast::*;
use std::fmt::Write;

/// Pretty printer for AST nodes with configurable indentation and formatting
pub struct PrettyPrinter {
    indent_level: usize,
    indent_size: usize,
    use_unicode: bool,
}

impl Default for PrettyPrinter {
    fn default() -> Self { Self::new() }
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self { indent_level: 0, indent_size: 2, use_unicode: true }
    }

    pub fn ascii() -> Self {
        Self { indent_level: 0, indent_size: 2, use_unicode: false }
    }

    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    // =========================================================================
    // Public entry points
    // =========================================================================

    pub fn print_source_file(&mut self, sf: &SourceFile) -> String {
        let mut out = String::new();
        self.format_source_file(sf, &mut out);
        out
    }

    pub fn print_expression(&mut self, expr: &Expression) -> String {
        let mut out = String::new();
        self.format_expression(expr, &mut out, true);
        out
    }

    pub fn print_type(&mut self, ast_type: &Type) -> String {
        let mut out = String::new();
        self.format_type(ast_type, &mut out, true);
        out
    }

    pub fn print_type_alias(&mut self, type_alias: &TypeAlias) -> String {
        let mut out = String::new();
        self.format_type_alias(type_alias, &mut out);
        out
    }

    pub fn print_generic_parameter(&mut self, param: &GenericParameter) -> String {
        let mut out = String::new();
        self.format_generic_parameter(param, &mut out, true);
        out
    }

    pub fn print_enum(&mut self, enum_decl: &Enum) -> String {
        let mut out = String::new();
        self.format_enum(enum_decl, &mut out);
        out
    }

    pub fn print_union(&mut self, union_decl: &Union) -> String {
        let mut out = String::new();
        self.format_union(union_decl, &mut out);
        out
    }

    pub fn print_struct(&mut self, struct_decl: &Struct) -> String {
        let mut out = String::new();
        self.format_struct(struct_decl, &mut out);
        out
    }

    pub fn print_function(&mut self, func: &Function) -> String {
        let mut out = String::new();
        self.format_function(func, &mut out);
        out
    }

    pub fn print_interface(&mut self, interface: &Interface) -> String {
        let mut out = String::new();
        self.format_interface(interface, &mut out);
        out
    }

    pub fn print_namespace(&mut self, namespace: &Namespace) -> String {
        let mut out = String::new();
        self.format_namespace(namespace, &mut out);
        out
    }

    pub fn print_statement(&mut self, stmt: &Statement) -> String {
        let mut out = String::new();
        self.format_statement(stmt, &mut out, true);
        out
    }

    // =========================================================================
    // Private helpers
    // =========================================================================

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    fn branch(&self) -> &'static str {
        if self.use_unicode { "├─ " } else { "|-- " }
    }

    fn last_branch(&self) -> &'static str {
        if self.use_unicode { "└─ " } else { "`-- " }
    }

    fn prefix(&self, is_last: bool) -> String {
        if self.indent_level == 0 {
            String::new()
        } else {
            format!("{}{}", self.indent(), if is_last { self.last_branch() } else { self.branch() })
        }
    }

    fn format_path_inline(path: &Path) -> String {
        path.segments.join("::")
    }

    fn format_receiver_inline(segs: &[GenericPathSegment]) -> String {
        segs.iter()
            .map(|seg| {
                if seg.generic_args.is_empty() {
                    seg.name.clone()
                } else {
                    let args: Vec<String> = seg.generic_args.iter()
                        .map(Self::format_generic_param_brief)
                        .collect();
                    format!("{}[{}]", seg.name, args.join(", "))
                }
            })
            .collect::<Vec<_>>()
            .join("::")
    }

    fn format_generic_param_brief(gp: &GenericParameter) -> String {
        match gp {
            GenericParameter::Type { name, bounds, .. } if bounds.is_empty() => name.clone(),
            GenericParameter::Type { name, bounds, .. } => {
                let bs: Vec<String> = bounds.iter().map(Self::format_type_brief).collect();
                format!("{}: {}", name, bs.join(" + "))
            },
            GenericParameter::Const { name, ty } =>
                format!("const {}: {}", name, Self::format_type_brief(ty)),
        }
    }

    fn format_type_brief(ty: &Type) -> String {
        match ty {
            Type::U8 => "u8".into(), Type::U16 => "u16".into(), Type::U32 => "u32".into(),
            Type::U64 => "u64".into(), Type::USize => "usize".into(),
            Type::I8 => "i8".into(), Type::I16 => "i16".into(), Type::I32 => "i32".into(),
            Type::I64 => "i64".into(), Type::ISize => "isize".into(),
            Type::F32 => "f32".into(), Type::F64 => "f64".into(),
            Type::Bool => "bool".into(), Type::Null => "null".into(), Type::Ok => "ok".into(),
            Type::SelfType => "Self".into(),
            Type::Path { path, generic_args } if generic_args.is_empty() =>
                Self::format_path_inline(path),
            Type::Path { path, generic_args } => {
                let args: Vec<String> = generic_args.iter().map(Self::format_type_brief).collect();
                format!("{}[{}]", Self::format_path_inline(path), args.join(", "))
            },
            Type::Nullable(inner) => format!("?{}", Self::format_type_brief(inner)),
            Type::Pointer { nullable: _, mutable, element_type } => {
                let m = if *mutable { "*mut " } else { "*" };
                format!("{}{}", m, Self::format_type_brief(element_type))
            },
            Type::Array { element_type, size: None } =>
                format!("[{}]", Self::format_type_brief(element_type)),
            Type::Array { element_type, size: Some(_) } =>
                format!("[{}; ...]", Self::format_type_brief(element_type)),
            Type::ErrorUnion { ok_type, err_type } =>
                format!("{}!{}", Self::format_type_brief(ok_type), Self::format_type_brief(err_type)),
            Type::Function { param_types, return_type } => {
                let params: Vec<String> = param_types.iter().map(Self::format_type_brief).collect();
                match return_type {
                    Some(r) => format!("func({}) -> {}", params.join(", "), Self::format_type_brief(r)),
                    None    => format!("func({})", params.join(", ")),
                }
            },
        }
    }

    fn format_visibility_inline(vis: &Visibility) -> &'static str {
        match vis {
            Visibility::Default => "",
            Visibility::Public  => "public ",
            Visibility::Export  => "export ",
            Visibility::Private => "private ",
        }
    }

    fn format_generic_params_section(&mut self, params: &[GenericParameter], output: &mut String) {
        if params.is_empty() { return; }
        writeln!(output, "{}generic_params:", self.indent()).unwrap();
        self.indent_level += 1;
        for (i, p) in params.iter().enumerate() {
            self.format_generic_parameter(p, output, i == params.len() - 1);
        }
        self.indent_level -= 1;
    }

    fn format_requires_section(&mut self, requires: &[Type], output: &mut String) {
        if requires.is_empty() { return; }
        writeln!(output, "{}requires:", self.indent()).unwrap();
        self.indent_level += 1;
        for (i, t) in requires.iter().enumerate() {
            self.format_type(t, output, i == requires.len() - 1);
        }
        self.indent_level -= 1;
    }

    fn format_annotations_section(&mut self, annotations: &[Annotation], output: &mut String) {
        if annotations.is_empty() { return; }
        writeln!(output, "{}annotations:", self.indent()).unwrap();
        self.indent_level += 1;
        for (i, ann) in annotations.iter().enumerate() {
            let p = self.prefix(i == annotations.len() - 1);
            if ann.args.is_empty() {
                writeln!(output, "{}#{}", p, ann.name).unwrap();
            } else {
                writeln!(output, "{}#{}(...)", p, ann.name).unwrap();
            }
        }
        self.indent_level -= 1;
    }

    // =========================================================================
    // Source file
    // =========================================================================

    fn format_source_file(&mut self, sf: &SourceFile, output: &mut String) {
        writeln!(output, "SourceFile").unwrap();
        self.indent_level += 1;
        for (i, item) in sf.items.iter().enumerate() {
            self.format_namespace_item(item, output, i == sf.items.len() - 1);
        }
        self.indent_level -= 1;
    }

    // =========================================================================
    // Expressions
    // =========================================================================

    fn format_expression(&mut self, expr: &Expression, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        match expr {
            Expression::IntegerLiteral(lit) => {
                writeln!(output, "{}IntegerLiteral: {}", p, lit).unwrap();
            }
            Expression::FloatLiteral(lit) => {
                writeln!(output, "{}FloatLiteral: {}", p, lit).unwrap();
            }
            Expression::BooleanLiteral(v) => {
                writeln!(output, "{}BooleanLiteral: {}", p, v).unwrap();
            }
            Expression::CharLiteral(ch) => {
                writeln!(output, "{}CharLiteral: '{}'", p, ch).unwrap();
            }
            Expression::StringLiteral(s) => {
                writeln!(output, "{}StringLiteral: \"{}\"", p, s).unwrap();
            }
            Expression::OkLiteral => {
                writeln!(output, "{}OkLiteral", p).unwrap();
            }
            Expression::NullLiteral => {
                writeln!(output, "{}NullLiteral", p).unwrap();
            }
            Expression::SelfValue => {
                writeln!(output, "{}SelfValue", p).unwrap();
            }
            Expression::Path(path) => {
                writeln!(output, "{}Path: {}", p, Self::format_path_inline(path)).unwrap();
            }
            Expression::InterpolatedString(parts) => {
                writeln!(output, "{}InterpolatedString ({} part(s))", p, parts.len()).unwrap();
                self.indent_level += 1;
                for (i, part) in parts.iter().enumerate() {
                    let pp = self.prefix(i == parts.len() - 1);
                    match part {
                        InterpolatedPart::Text(t) => {
                            writeln!(output, "{}Text: {:?}", pp, t).unwrap();
                        }
                        InterpolatedPart::Expression(e) => {
                            writeln!(output, "{}Expr:", pp).unwrap();
                            self.indent_level += 1;
                            self.format_expression(e, output, true);
                            self.indent_level -= 1;
                        }
                    }
                }
                self.indent_level -= 1;
            }
            Expression::ArrayLiteral(arr) => {
                writeln!(output, "{}ArrayLiteral", p).unwrap();
                self.indent_level += 1;
                if arr.elements.is_empty() {
                    writeln!(output, "{}(empty)", self.indent()).unwrap();
                } else {
                    writeln!(output, "{}elements: {} item(s)", self.indent(), arr.elements.len()).unwrap();
                    self.indent_level += 1;
                    for (i, e) in arr.elements.iter().enumerate() {
                        self.format_expression(e, output, i == arr.elements.len() - 1);
                    }
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
            }
            Expression::RepeatArrayLiteral(arr) => {
                writeln!(output, "{}RepeatArrayLiteral", p).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}value:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&arr.value, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}count:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&arr.count, output, true);
                self.indent_level -= 1;
                self.indent_level -= 1;
            }
            Expression::BinaryOp(op) => {
                writeln!(output, "{}BinaryOp: {:?}", p, op.op).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}left:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&op.lhs, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}right:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&op.rhs, output, true);
                self.indent_level -= 1;
                self.indent_level -= 1;
            }
            Expression::UnaryOp(op) => {
                writeln!(output, "{}UnaryOp: {:?}", p, op.op).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}operand:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&op.operand, output, true);
                self.indent_level -= 1;
                self.indent_level -= 1;
            }
            Expression::FieldAccess(fa) => {
                let bang = if fa.is_propagating { "!" } else { "" };
                writeln!(output, "{}FieldAccess: .{}{}", p, bang, fa.field).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}object:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&fa.object, output, true);
                self.indent_level -= 2;
            }
            Expression::TypeAccess(ta) => {
                writeln!(output, "{}TypeAccess: ::{}", p, ta.member).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}object:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&ta.object, output, true);
                self.indent_level -= 2;
            }
            Expression::Call(call) => {
                let bang = if call.is_propagating { "!" } else { "" };
                writeln!(output, "{}Call{} ({} arg(s))", p, bang, call.args.len()).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}callee:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&call.callee, output, true);
                self.indent_level -= 1;
                if !call.args.is_empty() {
                    writeln!(output, "{}args:", self.indent()).unwrap();
                    self.indent_level += 1;
                    for (i, a) in call.args.iter().enumerate() {
                        let is_last = i == call.args.len() - 1;
                        if let Some(ref name) = a.name {
                            let pp = self.prefix(is_last);
                            writeln!(output, "{}Arg[{}]:", pp, name).unwrap();
                            self.indent_level += 1;
                            self.format_expression(&a.value, output, true);
                            self.indent_level -= 1;
                        } else {
                            self.format_expression(&a.value, output, is_last);
                        }
                    }
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
            }
            Expression::Index(idx) => {
                writeln!(output, "{}Index", p).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}object:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&idx.object, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}index:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&idx.index, output, true);
                self.indent_level -= 2;
            }
            Expression::Cast(cast) => {
                writeln!(output, "{}Cast", p).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}expr:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&cast.expr, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}as:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_type(&cast.target_type, output, true);
                self.indent_level -= 2;
            }
            Expression::Sizeof(ty) => {
                writeln!(output, "{}Sizeof", p).unwrap();
                self.indent_level += 1;
                self.format_type(ty, output, true);
                self.indent_level -= 1;
            }
            Expression::Alignof(ty) => {
                writeln!(output, "{}Alignof", p).unwrap();
                self.indent_level += 1;
                self.format_type(ty, output, true);
                self.indent_level -= 1;
            }
            Expression::Offsetof(oo) => {
                writeln!(output, "{}Offsetof: .{}", p, oo.field).unwrap();
                self.indent_level += 1;
                self.format_type(&oo.ty, output, true);
                self.indent_level -= 1;
            }
            Expression::Parenthesized(inner) => {
                writeln!(output, "{}Parenthesized", p).unwrap();
                self.indent_level += 1;
                self.format_expression(inner, output, true);
                self.indent_level -= 1;
            }
            Expression::Assign(assign) => {
                writeln!(output, "{}Assign: {:?}", p, assign.op).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}lhs:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&assign.lhs, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}rhs:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&assign.rhs, output, true);
                self.indent_level -= 2;
            }
        }
    }

    // =========================================================================
    // Types
    // =========================================================================

    fn format_type(&mut self, ast_type: &Type, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        match ast_type {
            Type::U8    => { writeln!(output, "{}Type: U8",    p).unwrap(); }
            Type::U16   => { writeln!(output, "{}Type: U16",   p).unwrap(); }
            Type::U32   => { writeln!(output, "{}Type: U32",   p).unwrap(); }
            Type::U64   => { writeln!(output, "{}Type: U64",   p).unwrap(); }
            Type::USize => { writeln!(output, "{}Type: USize", p).unwrap(); }
            Type::I8    => { writeln!(output, "{}Type: I8",    p).unwrap(); }
            Type::I16   => { writeln!(output, "{}Type: I16",   p).unwrap(); }
            Type::I32   => { writeln!(output, "{}Type: I32",   p).unwrap(); }
            Type::I64   => { writeln!(output, "{}Type: I64",   p).unwrap(); }
            Type::ISize => { writeln!(output, "{}Type: ISize", p).unwrap(); }
            Type::F32   => { writeln!(output, "{}Type: F32",   p).unwrap(); }
            Type::F64   => { writeln!(output, "{}Type: F64",   p).unwrap(); }
            Type::Bool  => { writeln!(output, "{}Type: Bool",  p).unwrap(); }
            Type::Ok    => { writeln!(output, "{}Type: Ok",    p).unwrap(); }
            Type::Null  => { writeln!(output, "{}Type: Null",  p).unwrap(); }
            Type::SelfType => { writeln!(output, "{}Type: Self", p).unwrap(); }
            Type::Pointer { nullable, mutable, element_type } => {
                let q = if *nullable { "?" } else { "" };
                let m = if *mutable  { "mut " } else { "" };
                writeln!(output, "{}Type: {}*{}Pointer", p, q, m).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}element_type:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_type(element_type, output, true);
                self.indent_level -= 2;
            }
            Type::Path { path, generic_args } => {
                writeln!(output, "{}Type: Path({})", p, Self::format_path_inline(path)).unwrap();
                if !generic_args.is_empty() {
                    self.indent_level += 1;
                    writeln!(output, "{}generic_args:", self.indent()).unwrap();
                    self.indent_level += 1;
                    for (i, arg) in generic_args.iter().enumerate() {
                        self.format_type(arg, output, i == generic_args.len() - 1);
                    }
                    self.indent_level -= 2;
                }
            }
            Type::Array { element_type, size } => {
                writeln!(output, "{}Type: Array", p).unwrap();
                self.indent_level += 1;
                if let Some(s) = size {
                    writeln!(output, "{}size: {}", self.indent(), s).unwrap();
                } else {
                    writeln!(output, "{}size: dynamic", self.indent()).unwrap();
                }
                writeln!(output, "{}element_type:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_type(element_type, output, true);
                self.indent_level -= 2;
            }
            Type::Nullable(inner) => {
                writeln!(output, "{}Type: Nullable (?)", p).unwrap();
                self.indent_level += 1;
                self.format_type(inner, output, true);
                self.indent_level -= 1;
            }
            Type::ErrorUnion { ok_type, err_type } => {
                writeln!(output, "{}Type: ErrorUnion (!)", p).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}ok_type:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_type(ok_type, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}err_type:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_type(err_type, output, true);
                self.indent_level -= 1;
            }
            Type::Function { param_types, return_type } => {
                writeln!(output, "{}Type: Function ({} param(s))", p, param_types.len()).unwrap();
                self.indent_level += 1;
                if !param_types.is_empty() {
                    writeln!(output, "{}params:", self.indent()).unwrap();
                    self.indent_level += 1;
                    let last = param_types.len() - 1;
                    for (i, pt) in param_types.iter().enumerate() {
                        self.format_type(pt, output, i == last);
                    }
                    self.indent_level -= 1;
                }
                if let Some(ret) = return_type {
                    writeln!(output, "{}return_type:", self.indent()).unwrap();
                    self.indent_level += 1;
                    self.format_type(ret, output, true);
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
            }
        }
    }

    // =========================================================================
    // Generic parameters
    // =========================================================================

    fn format_generic_parameter(&mut self, param: &GenericParameter, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        match param {
            GenericParameter::Type { name, bounds, default_type } => {
                writeln!(output, "{}TypeParam: {}", p, name).unwrap();
                if !bounds.is_empty() || default_type.is_some() {
                    self.indent_level += 1;
                    if !bounds.is_empty() {
                        writeln!(output, "{}bounds:", self.indent()).unwrap();
                        self.indent_level += 1;
                        for (i, b) in bounds.iter().enumerate() {
                            self.format_type(b, output, i == bounds.len() - 1);
                        }
                        self.indent_level -= 1;
                    }
                    if let Some(def) = default_type {
                        writeln!(output, "{}default:", self.indent()).unwrap();
                        self.indent_level += 1;
                        self.format_type(def, output, true);
                        self.indent_level -= 1;
                    }
                    self.indent_level -= 1;
                }
            }
            GenericParameter::Const { name, ty } => {
                writeln!(output, "{}ConstParam: {}", p, name).unwrap();
                self.indent_level += 1;
                self.format_type(ty, output, true);
                self.indent_level -= 1;
            }
        }
    }

    // =========================================================================
    // Type alias
    // =========================================================================

    fn format_type_alias(&mut self, ta: &TypeAlias, output: &mut String) {
        let vis = Self::format_visibility_inline(&ta.visibility);
        writeln!(output, "TypeAlias: {}{}", vis, Self::format_path_inline(&ta.name)).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&ta.annotations, output);
        self.format_generic_params_section(&ta.generic_params, output);
        writeln!(output, "{}aliased_type:", self.indent()).unwrap();
        self.indent_level += 1;
        self.format_type(&ta.aliased_type, output, true);
        self.indent_level -= 2;
    }

    // =========================================================================
    // Enum
    // =========================================================================

    fn format_enum(&mut self, e: &Enum, output: &mut String) {
        let vis = Self::format_visibility_inline(&e.visibility);
        writeln!(output, "Enum: {}{}", vis, Self::format_path_inline(&e.name)).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&e.annotations, output);
        if let Some(repr) = &e.representation {
            writeln!(output, "{}representation:", self.indent()).unwrap();
            self.indent_level += 1;
            self.format_type(repr, output, true);
            self.indent_level -= 1;
        }
        self.format_generic_params_section(&e.generic_params, output);
        self.format_requires_section(&e.requires, output);
        if !e.variants.is_empty() {
            writeln!(output, "{}variants:", self.indent()).unwrap();
            self.indent_level += 1;
            for (i, v) in e.variants.iter().enumerate() {
                self.format_enum_variant(v, output, i == e.variants.len() - 1);
            }
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn format_enum_variant(&mut self, v: &EnumVariant, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        if let Some(val) = &v.value {
            writeln!(output, "{}Variant: {} = {}", p, v.name, val).unwrap();
        } else {
            writeln!(output, "{}Variant: {}", p, v.name).unwrap();
        }
    }

    // =========================================================================
    // Union
    // =========================================================================

    fn format_union(&mut self, u: &Union, output: &mut String) {
        let vis = Self::format_visibility_inline(&u.visibility);
        writeln!(output, "Union: {}{}", vis, Self::format_path_inline(&u.name)).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&u.annotations, output);
        self.format_generic_params_section(&u.generic_params, output);
        self.format_requires_section(&u.requires, output);
        if !u.variants.is_empty() {
            writeln!(output, "{}variants:", self.indent()).unwrap();
            self.indent_level += 1;
            for (i, v) in u.variants.iter().enumerate() {
                self.format_union_variant(v, output, i == u.variants.len() - 1);
            }
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn format_union_variant(&mut self, v: &UnionVariant, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        writeln!(output, "{}Variant: {} :", p, v.name).unwrap();
        self.indent_level += 1;
        self.format_type(&v.ty, output, true);
        self.indent_level -= 1;
    }

    // =========================================================================
    // Struct
    // =========================================================================

    fn format_struct(&mut self, s: &Struct, output: &mut String) {
        let vis = Self::format_visibility_inline(&s.visibility);
        let packed = if s.is_packed { "packed " } else { "" };
        writeln!(output, "Struct: {}{}{}", vis, packed, Self::format_path_inline(&s.name)).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&s.annotations, output);
        self.format_generic_params_section(&s.generic_params, output);
        self.format_requires_section(&s.requires, output);
        if !s.fields.is_empty() {
            writeln!(output, "{}fields:", self.indent()).unwrap();
            self.indent_level += 1;
            for (i, f) in s.fields.iter().enumerate() {
                self.format_struct_field(f, output, i == s.fields.len() - 1);
            }
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn format_struct_field(&mut self, f: &StructField, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        writeln!(output, "{}Field: {} :", p, f.name).unwrap();
        self.indent_level += 1;
        self.format_type(&f.ty, output, true);
        self.indent_level -= 1;
    }

    // =========================================================================
    // Functions
    // =========================================================================

    fn format_function(&mut self, func: &Function, output: &mut String) {
        writeln!(output, "Function: {}", func.signature.name).unwrap();
        self.indent_level += 1;
        writeln!(output, "{}signature:", self.indent()).unwrap();
        self.indent_level += 1;
        self.format_function_signature(&func.signature, output);
        self.indent_level -= 1;
        writeln!(output, "{}body:", self.indent()).unwrap();
        self.indent_level += 1;
        self.format_block(&func.body, output);
        self.indent_level -= 2;
    }

    fn format_function_declaration(&mut self, decl: &FunctionDeclaration, output: &mut String) {
        writeln!(output, "FunctionDeclaration: {}", decl.signature.name).unwrap();
        self.indent_level += 1;
        self.format_function_signature(&decl.signature, output);
        self.indent_level -= 1;
    }

    fn format_function_signature(&mut self, sig: &FunctionSignature, output: &mut String) {
        let vis = Self::format_visibility_inline(&sig.visibility);
        let ext = if sig.is_extern { "extern " } else { "" };
        let eff = if sig.is_effect { "func! " } else { "func " };
        let receiver = sig.receiver.as_ref()
            .map(|segs| format!("{}::", Self::format_receiver_inline(segs)))
            .unwrap_or_default();
        writeln!(output, "FunctionSignature: {}{}{}{}{}", vis, ext, eff, receiver, sig.name).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&sig.annotations, output);
        self.format_generic_params_section(&sig.outer_generic_params, output);
        if !sig.func_generic_params.is_empty() {
            self.format_generic_params_section(&sig.func_generic_params, output);
        }
        if let Some(sp) = &sig.self_param {
            let m = if sp.is_mutable   { "mut " }     else { "" };
            let ptr = if sp.is_pointer { "pointer " } else { "" };
            writeln!(output, "{}self_param: {}{}self", self.indent(), m, ptr).unwrap();
        }
        if !sig.params.is_empty() {
            writeln!(output, "{}params:", self.indent()).unwrap();
            self.indent_level += 1;
            for (i, param) in sig.params.iter().enumerate() {
                self.format_function_parameter(param, output, i == sig.params.len() - 1);
            }
            self.indent_level -= 1;
        }
        if !sig.return_types.is_empty() {
            writeln!(output, "{}return_types:", self.indent()).unwrap();
            self.indent_level += 1;
            let n = sig.return_types.len();
            for (i, rt) in sig.return_types.iter().enumerate() {
                self.format_type(rt, output, i == n - 1);
            }
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    fn format_function_parameter(&mut self, param: &FunctionParameter, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        writeln!(output, "{}Param: {} :", p, param.name).unwrap();
        self.indent_level += 1;
        self.format_type(&param.ty, output, true);
        self.indent_level -= 1;
    }

    // =========================================================================
    // Block
    // =========================================================================

    fn format_block(&mut self, block: &Block, output: &mut String) {
        if block.statements.is_empty() {
            writeln!(output, "{}Block (empty)", self.indent()).unwrap();
        } else {
            writeln!(output, "{}Block ({} statement(s))", self.indent(), block.statements.len()).unwrap();
            self.indent_level += 1;
            for (i, stmt) in block.statements.iter().enumerate() {
                self.format_statement(stmt, output, i == block.statements.len() - 1);
            }
            self.indent_level -= 1;
        }
    }

    // =========================================================================
    // Statements
    // =========================================================================

    fn format_statement(&mut self, stmt: &Statement, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        match stmt {
            Statement::Pass => {
                writeln!(output, "{}Pass", p).unwrap();
            }
            Statement::Break => {
                writeln!(output, "{}Break", p).unwrap();
            }
            Statement::Continue => {
                writeln!(output, "{}Continue", p).unwrap();
            }
            Statement::Expression(e) => {
                writeln!(output, "{}Expression:", p).unwrap();
                self.indent_level += 1;
                self.format_expression(e, output, true);
                self.indent_level -= 1;
            }
            Statement::Let(s) => {
                let ty_str = s.ty.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                writeln!(output, "{}Let: {}{}", p, s.name, ty_str).unwrap();
                self.indent_level += 1;
                self.format_annotations_section(&s.annotations, output);
                writeln!(output, "{}value:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&s.value, output, true);
                self.indent_level -= 2;
            }
            Statement::Mut(s) => {
                let ty_str = s.ty.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                writeln!(output, "{}Mut: {}{}", p, s.name, ty_str).unwrap();
                self.indent_level += 1;
                self.format_annotations_section(&s.annotations, output);
                writeln!(output, "{}value:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&s.value, output, true);
                self.indent_level -= 2;
            }
            Statement::Const(s) => {
                let vis = Self::format_visibility_inline(&s.visibility);
                let gp = if s.generic_params.is_empty() { String::new() } else {
                    let parts: Vec<String> = s.generic_params.iter().map(|p| format!("{}", p)).collect();
                    format!("[{}] ", parts.join(", "))
                };
                let recv = if s.receiver.is_empty() { String::new() } else {
                    let segs: Vec<String> = s.receiver.iter().map(|seg| {
                        if seg.generic_args.is_empty() {
                            seg.name.clone()
                        } else {
                            let args: Vec<String> = seg.generic_args.iter().map(|t| format!("{}", t)).collect();
                            format!("{}[{}]", seg.name, args.join(", "))
                        }
                    }).collect();
                    format!("{}::", segs.join("::"))
                };
                let ty_str = s.ty.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                writeln!(output, "{}Const: {}{}{}{}{}", p, vis, gp, recv, s.name, ty_str).unwrap();
                self.indent_level += 1;
                self.format_annotations_section(&s.annotations, output);
                writeln!(output, "{}value:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&s.value, output, true);
                self.indent_level -= 2;
            }
            Statement::Return(e) => {
                writeln!(output, "{}Return:", p).unwrap();
                self.indent_level += 1;
                self.format_expression(e, output, true);
                self.indent_level -= 1;
            }
            Statement::Block(bs) => {
                let name_str = bs.name.as_deref().unwrap_or("<anon>");
                writeln!(output, "{}Block: {}", p, name_str).unwrap();
                self.indent_level += 1;
                self.format_block(&bs.body, output);
                self.indent_level -= 1;
            }
            Statement::If(s) => {
                writeln!(output, "{}If", p).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}condition:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&s.condition, output, true);
                self.indent_level -= 1;
                writeln!(output, "{}then:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_block(&s.then_body, output);
                self.indent_level -= 1;
                for elif in &s.elif_clauses {
                    writeln!(output, "{}elif:", self.indent()).unwrap();
                    self.indent_level += 1;
                    writeln!(output, "{}condition:", self.indent()).unwrap();
                    self.indent_level += 1;
                    self.format_expression(&elif.condition, output, true);
                    self.indent_level -= 1;
                    self.format_block(&elif.body, output);
                    self.indent_level -= 1;
                }
                if let Some(els) = &s.else_body {
                    writeln!(output, "{}else:", self.indent()).unwrap();
                    self.indent_level += 1;
                    self.format_block(els, output);
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
            }
            Statement::For(s) => {
                writeln!(output, "{}For: {} in ...", p, s.pattern).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}iterable:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&s.iterable, output, true);
                self.indent_level -= 1;
                self.format_block(&s.body, output);
                self.indent_level -= 1;
            }
            Statement::While(s) => {
                writeln!(output, "{}While", p).unwrap();
                self.indent_level += 1;
                writeln!(output, "{}condition:", self.indent()).unwrap();
                self.indent_level += 1;
                self.format_expression(&s.condition, output, true);
                self.indent_level -= 1;
                self.format_block(&s.body, output);
                self.indent_level -= 1;
            }
            Statement::Using(s) => {
                let vis = Self::format_visibility_inline(&s.visibility);
                writeln!(output, "{}Using: {}{}", p, vis, Self::format_path_inline(&s.path)).unwrap();
            }
            Statement::Function(f) => {
                writeln!(output, "{}Function:", p).unwrap();
                self.indent_level += 1;
                self.format_function(f, output);
                self.indent_level -= 1;
            }
            Statement::FunctionDeclaration(d) => {
                writeln!(output, "{}FunctionDeclaration:", p).unwrap();
                self.indent_level += 1;
                self.format_function_declaration(d, output);
                self.indent_level -= 1;
            }
            Statement::TypeAlias(ta) => {
                writeln!(output, "{}TypeAlias:", p).unwrap();
                self.indent_level += 1;
                self.format_type_alias(ta, output);
                self.indent_level -= 1;
            }
            Statement::Struct(s) => {
                writeln!(output, "{}Struct:", p).unwrap();
                self.indent_level += 1;
                self.format_struct(s, output);
                self.indent_level -= 1;
            }
            Statement::Enum(e) => {
                writeln!(output, "{}Enum:", p).unwrap();
                self.indent_level += 1;
                self.format_enum(e, output);
                self.indent_level -= 1;
            }
            Statement::Union(u) => {
                writeln!(output, "{}Union:", p).unwrap();
                self.indent_level += 1;
                self.format_union(u, output);
                self.indent_level -= 1;
            }
            Statement::Interface(i) => {
                writeln!(output, "{}Interface:", p).unwrap();
                self.indent_level += 1;
                self.format_interface(i, output);
                self.indent_level -= 1;
            }
            Statement::Namespace(ns) => {
                writeln!(output, "{}Namespace:", p).unwrap();
                self.indent_level += 1;
                self.format_namespace(ns, output);
                self.indent_level -= 1;
            }
        }
    }

    // =========================================================================
    // Interface
    // =========================================================================

    fn format_interface(&mut self, iface: &Interface, output: &mut String) {
        let vis = Self::format_visibility_inline(&iface.visibility);
        writeln!(output, "Interface: {}{}", vis, Self::format_path_inline(&iface.name)).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&iface.annotations, output);
        self.format_generic_params_section(&iface.generic_params, output);
        if !iface.extends.is_empty() {
            writeln!(output, "{}extends:", self.indent()).unwrap();
            self.indent_level += 1;
            for (i, t) in iface.extends.iter().enumerate() {
                self.format_type(t, output, i == iface.extends.len() - 1);
            }
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    // =========================================================================
    // Namespace
    // =========================================================================

    fn format_namespace(&mut self, ns: &Namespace, output: &mut String) {
        let vis = Self::format_visibility_inline(&ns.visibility);
        writeln!(output, "Namespace: {}{}", vis, Self::format_path_inline(&ns.name)).unwrap();
        self.indent_level += 1;
        self.format_annotations_section(&ns.annotations, output);
        if !ns.items.is_empty() {
            writeln!(output, "{}items:", self.indent()).unwrap();
            self.indent_level += 1;
            for (i, item) in ns.items.iter().enumerate() {
                self.format_statement(item, output, i == ns.items.len() - 1);
            }
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
    }

    // =========================================================================
    // Namespace item (file-level)
    // =========================================================================

    fn format_namespace_item(&mut self, item: &NamespaceItem, output: &mut String, is_last: bool) {
        let p = self.prefix(is_last);
        match item {
            NamespaceItem::Namespace(ns) => {
                writeln!(output, "{}Namespace:", p).unwrap();
                self.indent_level += 1;
                self.format_namespace(ns, output);
                self.indent_level -= 1;
            }
            NamespaceItem::NamespaceDeclaration(nd) => {
                let vis = Self::format_visibility_inline(&nd.visibility);
                writeln!(output, "{}NamespaceDeclaration: {}{}", p, vis, Self::format_path_inline(&nd.name)).unwrap();
            }
            NamespaceItem::Function(f) => {
                writeln!(output, "{}Function:", p).unwrap();
                self.indent_level += 1;
                self.format_function(f, output);
                self.indent_level -= 1;
            }
            NamespaceItem::FunctionDeclaration(d) => {
                writeln!(output, "{}FunctionDeclaration:", p).unwrap();
                self.indent_level += 1;
                self.format_function_declaration(d, output);
                self.indent_level -= 1;
            }
            NamespaceItem::TypeAlias(ta) => {
                writeln!(output, "{}TypeAlias:", p).unwrap();
                self.indent_level += 1;
                self.format_type_alias(ta, output);
                self.indent_level -= 1;
            }
            NamespaceItem::Struct(s) => {
                writeln!(output, "{}Struct:", p).unwrap();
                self.indent_level += 1;
                self.format_struct(s, output);
                self.indent_level -= 1;
            }
            NamespaceItem::Enum(e) => {
                writeln!(output, "{}Enum:", p).unwrap();
                self.indent_level += 1;
                self.format_enum(e, output);
                self.indent_level -= 1;
            }
            NamespaceItem::Union(u) => {
                writeln!(output, "{}Union:", p).unwrap();
                self.indent_level += 1;
                self.format_union(u, output);
                self.indent_level -= 1;
            }
            NamespaceItem::Interface(i) => {
                writeln!(output, "{}Interface:", p).unwrap();
                self.indent_level += 1;
                self.format_interface(i, output);
                self.indent_level -= 1;
            }
            NamespaceItem::Using(s) => {
                let vis = Self::format_visibility_inline(&s.visibility);
                writeln!(output, "{}Using: {}{}", p, vis, Self::format_path_inline(&s.path)).unwrap();
            }
            NamespaceItem::Const(s) => {
                let vis = Self::format_visibility_inline(&s.visibility);
                let gp = if s.generic_params.is_empty() { String::new() } else {
                    let parts: Vec<String> = s.generic_params.iter().map(|p| format!("{}", p)).collect();
                    format!("[{}] ", parts.join(", "))
                };
                let recv = if s.receiver.is_empty() { String::new() } else {
                    let segs: Vec<String> = s.receiver.iter().map(|seg| {
                        if seg.generic_args.is_empty() {
                            seg.name.clone()
                        } else {
                            let args: Vec<String> = seg.generic_args.iter().map(|t| format!("{}", t)).collect();
                            format!("{}[{}]", seg.name, args.join(", "))
                        }
                    }).collect();
                    format!("{}::", segs.join("::"))
                };
                let ty_str = s.ty.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                writeln!(output, "{}Const: {}{}{}{}{}", p, vis, gp, recv, s.name, ty_str).unwrap();
            }
        }
    }
}

// =============================================================================
// Convenience free functions
// =============================================================================

pub fn print_expression(expr: &Expression) -> String {
    PrettyPrinter::new().print_expression(expr)
}

pub fn print_expression_ascii(expr: &Expression) -> String {
    PrettyPrinter::ascii().print_expression(expr)
}

pub fn print_enum(enum_decl: &Enum) -> String {
    PrettyPrinter::new().print_enum(enum_decl)
}

pub fn print_union(union_decl: &Union) -> String {
    PrettyPrinter::new().print_union(union_decl)
}

pub fn print_struct(struct_decl: &Struct) -> String {
    PrettyPrinter::new().print_struct(struct_decl)
}

pub fn print_function(func: &Function) -> String {
    PrettyPrinter::new().print_function(func)
}

pub fn print_interface(interface: &Interface) -> String {
    PrettyPrinter::new().print_interface(interface)
}

pub fn print_namespace(namespace: &Namespace) -> String {
    PrettyPrinter::new().print_namespace(namespace)
}

// =============================================================================
// Display impls
// =============================================================================

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_expression(self))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", PrettyPrinter::new().print_type(self))
    }
}

impl std::fmt::Display for TypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", PrettyPrinter::new().print_type_alias(self))
    }
}

impl std::fmt::Display for GenericParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", PrettyPrinter::new().print_generic_parameter(self))
    }
}

impl std::fmt::Display for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_enum(self))
    }
}

impl std::fmt::Display for Union {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_union(self))
    }
}

impl std::fmt::Display for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_struct(self))
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_function(self))
    }
}

impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_interface(self))
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_namespace(self))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use fig_lexer::IntegerLiteral;

    fn int_lit(digits: &str) -> Expression {
        Expression::IntegerLiteral(
            IntegerLiteral::builder().digits(digits.to_string()).build().unwrap(),
        )
    }

    #[test]
    fn test_print_integer_literal() {
        let output = print_expression(&int_lit("42"));
        assert!(output.contains("IntegerLiteral"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_print_boolean() {
        let output = print_expression(&Expression::BooleanLiteral(true));
        assert!(output.contains("BooleanLiteral: true"));
    }

    #[test]
    fn test_print_ok_literal() {
        assert!(print_expression(&Expression::OkLiteral).contains("OkLiteral"));
    }

    #[test]
    fn test_print_null_literal() {
        assert!(print_expression(&Expression::NullLiteral).contains("NullLiteral"));
    }

    #[test]
    fn test_print_self_value() {
        assert!(print_expression(&Expression::SelfValue).contains("SelfValue"));
    }

    #[test]
    fn test_print_path_simple() {
        let expr = Expression::Path(Path::simple("my_var".to_string()));
        assert!(print_expression(&expr).contains("Path: my_var"));
    }

    #[test]
    fn test_print_path_qualified() {
        let expr = Expression::Path(Path {
            segments: vec!["std".to_string(), "Vec".to_string()],
        });
        assert!(print_expression(&expr).contains("Path: std::Vec"));
    }

    #[test]
    fn test_print_binary_op() {
        let expr = Expression::BinaryOp(BinaryOpExpr {
            lhs: Box::new(int_lit("1")),
            op: BinaryOperator::Add,
            rhs: Box::new(int_lit("2")),
        });
        let out = print_expression(&expr);
        assert!(out.contains("BinaryOp: Add"));
        assert!(out.contains("left:"));
        assert!(out.contains("right:"));
    }

    #[test]
    fn test_print_unary_op() {
        let expr = Expression::UnaryOp(UnaryOpExpr {
            op: UnaryOperator::Negate,
            operand: Box::new(int_lit("5")),
        });
        let out = print_expression(&expr);
        assert!(out.contains("UnaryOp: Negate"));
        assert!(out.contains("operand:"));
    }

    #[test]
    fn test_print_field_access() {
        let expr = Expression::FieldAccess(FieldAccessExpr {
            object: Box::new(Expression::Path(Path::simple("obj".to_string()))),
            field: "field".to_string(),
            is_propagating: false,
        });
        assert!(print_expression(&expr).contains("FieldAccess: .field"));
    }

    #[test]
    fn test_print_field_access_propagating() {
        let expr = Expression::FieldAccess(FieldAccessExpr {
            object: Box::new(Expression::Path(Path::simple("obj".to_string()))),
            field: "field".to_string(),
            is_propagating: true,
        });
        assert!(print_expression(&expr).contains("FieldAccess: .!field"));
    }

    #[test]
    fn test_print_type_access() {
        let expr = Expression::TypeAccess(TypeAccessExpr {
            object: Box::new(Expression::Path(Path::simple("Vec".to_string()))),
            member: "new".to_string(),
        });
        assert!(print_expression(&expr).contains("TypeAccess: ::new"));
    }

    #[test]
    fn test_print_call() {
        let expr = Expression::Call(CallExpr {
            callee: Box::new(Expression::Path(Path::simple("foo".to_string()))),
            args: vec![
                CallArgument { name: None, value: int_lit("1") },
                CallArgument { name: None, value: int_lit("2") },
            ],
            generic_args: Some(vec![Type::I32]),
            is_propagating: false,
        });
        let out = print_expression(&expr);
        assert!(out.contains("Call"));
        assert!(out.contains("callee:"));
        assert!(out.contains("args:"));
    }

    #[test]
    fn test_print_call_propagating() {
        let expr = Expression::Call(CallExpr {
            callee: Box::new(Expression::Path(Path::simple("try_foo".to_string()))),
            args: vec![],
            generic_args: None,
            is_propagating: true,
        });
        assert!(print_expression(&expr).contains("Call!"));
    }

    #[test]
    fn test_print_index() {
        let expr = Expression::Index(IndexExpr {
            object: Box::new(Expression::Path(Path::simple("arr".to_string()))),
            index: Box::new(int_lit("0")),
        });
        let out = print_expression(&expr);
        assert!(out.contains("Index"));
        assert!(out.contains("object:"));
        assert!(out.contains("index:"));
    }

    #[test]
    fn test_print_cast() {
        let expr = Expression::Cast(CastExpr {
            expr: Box::new(int_lit("42")),
            target_type: Box::new(Type::F64),
        });
        let out = print_expression(&expr);
        assert!(out.contains("Cast"));
        assert!(out.contains("as:"));
    }

    #[test]
    fn test_print_sizeof() {
        let out = print_expression(&Expression::Sizeof(Box::new(Type::I32)));
        assert!(out.contains("Sizeof"));
        assert!(out.contains("I32"));
    }

    #[test]
    fn test_print_alignof() {
        let out = print_expression(&Expression::Alignof(Box::new(Type::U64)));
        assert!(out.contains("Alignof"));
    }

    #[test]
    fn test_print_offsetof() {
        let expr = Expression::Offsetof(OffsetofExpr {
            ty: Box::new(Type::Path { path: Path::simple("MyStruct".to_string()), generic_args: vec![] }),
            field: "x".to_string(),
        });
        assert!(print_expression(&expr).contains("Offsetof: .x"));
    }

    #[test]
    fn test_print_array_literal() {
        let expr = Expression::ArrayLiteral(ArrayLiteralExpr {
            elements: vec![int_lit("1"), int_lit("2"), int_lit("3")],
        });
        let out = print_expression(&expr);
        assert!(out.contains("ArrayLiteral"));
        assert!(out.contains("elements: 3 item(s)"));
    }

    #[test]
    fn test_print_empty_array() {
        let expr = Expression::ArrayLiteral(ArrayLiteralExpr { elements: vec![] });
        let out = print_expression(&expr);
        assert!(out.contains("ArrayLiteral"));
        assert!(out.contains("(empty)"));
    }

    #[test]
    fn test_print_interpolated_string() {
        let expr = Expression::InterpolatedString(vec![
            InterpolatedPart::Text("hello ".to_string()),
            InterpolatedPart::Expression(Box::new(Expression::Path(Path::simple("name".to_string())))),
        ]);
        let out = print_expression(&expr);
        assert!(out.contains("InterpolatedString"));
        assert!(out.contains("Text:"));
        assert!(out.contains("Expr:"));
    }

    #[test]
    fn test_print_nested_expression() {
        let add_expr = Expression::BinaryOp(BinaryOpExpr {
            lhs: Box::new(int_lit("1")),
            op: BinaryOperator::Add,
            rhs: Box::new(int_lit("2")),
        });
        let expr = Expression::BinaryOp(BinaryOpExpr {
            lhs: Box::new(Expression::Parenthesized(Box::new(add_expr))),
            op: BinaryOperator::Multiply,
            rhs: Box::new(int_lit("3")),
        });
        let out = print_expression(&expr);
        assert!(out.contains("BinaryOp: Multiply"));
        assert!(out.contains("Parenthesized"));
        assert!(out.contains("BinaryOp: Add"));
    }

    #[test]
    fn test_ascii_output() {
        let out = print_expression_ascii(&Expression::BooleanLiteral(true));
        assert!(!out.contains('\u{251C}'));
        assert!(!out.contains('\u{2514}'));
    }

    #[test]
    fn test_custom_indent_size() {
        let out = PrettyPrinter::new().with_indent_size(4).print_expression(&Expression::BooleanLiteral(false));
        assert!(out.contains("BooleanLiteral: false"));
    }

    #[test]
    fn test_display_trait_expression() {
        let out = format!("{}", Expression::Path(Path::simple("my_var".to_string())));
        assert!(out.contains("Path: my_var"));
    }

    // ── Type tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_print_primitive_types() {
        for (ty, name) in [
            (Type::U8, "U8"), (Type::I32, "I32"), (Type::F64, "F64"),
            (Type::Bool, "Bool"), (Type::Ok, "Ok"), (Type::Null, "Null"),
            (Type::SelfType, "Self"),
        ] {
            let out = PrettyPrinter::new().print_type(&ty);
            assert!(out.contains(name), "expected '{}' in '{}'", name, out);
        }
    }

    #[test]
    fn test_print_pointer_type() {
        let ty = Type::Pointer { nullable: false, mutable: false, element_type: Box::new(Type::Bool) };
        let out = PrettyPrinter::new().print_type(&ty);
        assert!(out.contains("Pointer"));
        assert!(out.contains("element_type:"));
        assert!(out.contains("Bool"));
    }

    #[test]
    fn test_print_nullable_mutable_pointer_type() {
        let ty = Type::Pointer { nullable: true, mutable: true, element_type: Box::new(Type::I32) };
        let out = PrettyPrinter::new().print_type(&ty);
        assert!(out.contains("?*mut"));
    }

    #[test]
    fn test_print_path_type() {
        let ty = Type::Path { path: Path::simple("MyStruct".to_string()), generic_args: vec![] };
        let out = PrettyPrinter::new().print_type(&ty);
        assert!(out.contains("Type: Path(MyStruct)"));
    }

    #[test]
    fn test_print_array_type_fixed_size() {
        let ty = Type::Array {
            element_type: Box::new(Type::U8),
            size: Some(Box::new(int_lit("10"))),
        };
        let out = PrettyPrinter::new().print_type(&ty);
        assert!(out.contains("Type: Array"));
        assert!(out.contains("size:"));
        assert!(out.contains("U8"));
    }

    #[test]
    fn test_print_array_type_dynamic() {
        let ty = Type::Array { element_type: Box::new(Type::Bool), size: None };
        let out = PrettyPrinter::new().print_type(&ty);
        assert!(out.contains("size: dynamic"));
    }

    #[test]
    fn test_display_trait_type() {
        let ty = Type::Pointer { nullable: false, mutable: false, element_type: Box::new(Type::I32) };
        let out = format!("{}", ty);
        assert!(out.contains("Pointer"));
    }

    // ── Generic parameter tests ──────────────────────────────────────────────

    #[test]
    fn test_print_type_param_simple() {
        let p = GenericParameter::Type { name: "T".to_string(), bounds: vec![], default_type: None };
        assert!(PrettyPrinter::new().print_generic_parameter(&p).contains("TypeParam: T"));
    }

    #[test]
    fn test_print_type_param_with_bounds() {
        let p = GenericParameter::Type {
            name: "T".to_string(),
            bounds: vec![Type::Path { path: Path::simple("Display".to_string()), generic_args: vec![] }],
            default_type: None,
        };
        let out = PrettyPrinter::new().print_generic_parameter(&p);
        assert!(out.contains("TypeParam: T"));
        assert!(out.contains("bounds:"));
    }

    #[test]
    fn test_print_type_param_with_default() {
        let p = GenericParameter::Type {
            name: "T".to_string(),
            bounds: vec![],
            default_type: Some(Box::new(Type::I32)),
        };
        let out = PrettyPrinter::new().print_generic_parameter(&p);
        assert!(out.contains("TypeParam: T"));
        assert!(out.contains("default:"));
    }

    #[test]
    fn test_print_const_param() {
        let p = GenericParameter::Const { name: "N".to_string(), ty: Type::USize };
        assert!(PrettyPrinter::new().print_generic_parameter(&p).contains("ConstParam: N"));
    }

    // ── Statement tests ──────────────────────────────────────────────────────

    #[test]
    fn test_print_pass_statement() {
        assert!(PrettyPrinter::new().print_statement(&Statement::Pass).contains("Pass"));
    }

    #[test]
    fn test_print_let_statement() {
        let stmt = Statement::Let(LetStatement {
            annotations: vec![],
            name: "x".to_string(),
            ty: Some(Type::I32),
            value: Box::new(int_lit("42")),
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("Let: x"));
        assert!(out.contains("value:"));
    }

    #[test]
    fn test_print_mut_statement() {
        let stmt = Statement::Mut(MutStatement {
            annotations: vec![],
            name: "y".to_string(),
            ty: None,
            value: Box::new(int_lit("0")),
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("Mut: y"));
    }

    #[test]
    fn test_print_const_statement() {
        let stmt = Statement::Const(ConstStatement {
            visibility: Visibility::Public,
            annotations: vec![],
            generic_params: vec![],
            receiver: vec![],
            name: "MAX".to_string(),
            ty: Some(Type::U64),
            value: Box::new(int_lit("100")),
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("Const: public MAX"));
    }

    #[test]
    fn test_print_return_statement() {
        let out = PrettyPrinter::new().print_statement(&Statement::Return(Box::new(int_lit("0"))));
        assert!(out.contains("Return:"));
    }

    #[test]
    fn test_print_using_statement() {
        let stmt = Statement::Using(UsingStatement {
            visibility: Visibility::Default,
            annotations: vec![],
            path: Path { segments: vec!["std".to_string(), "io".to_string()] },
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("Using: std::io"));
    }

    #[test]
    fn test_print_if_statement() {
        let stmt = Statement::If(IfStatement {
            condition: Box::new(Expression::BooleanLiteral(true)),
            then_body: Block { statements: vec![Statement::Pass] },
            elif_clauses: vec![],
            else_body: None,
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("If"));
        assert!(out.contains("condition:"));
        assert!(out.contains("then:"));
    }

    #[test]
    fn test_print_if_elif_else() {
        let stmt = Statement::If(IfStatement {
            condition: Box::new(Expression::BooleanLiteral(true)),
            then_body: Block { statements: vec![] },
            elif_clauses: vec![ElifClause {
                condition: Box::new(Expression::BooleanLiteral(false)),
                body: Block { statements: vec![] },
            }],
            else_body: Some(Block { statements: vec![Statement::Pass] }),
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("elif:"));
        assert!(out.contains("else:"));
    }

    #[test]
    fn test_print_for_statement() {
        let stmt = Statement::For(ForStatement {
            pattern: "item".to_string(),
            iterable: Box::new(Expression::Path(Path::simple("items".to_string()))),
            body: Block { statements: vec![] },
        });
        let out = PrettyPrinter::new().print_statement(&stmt);
        assert!(out.contains("For: item in"));
    }

    #[test]
    fn test_print_while_statement() {
        let stmt = Statement::While(WhileStatement {
            condition: Box::new(Expression::BooleanLiteral(true)),
            body: Block { statements: vec![] },
        });
        assert!(PrettyPrinter::new().print_statement(&stmt).contains("While"));
    }

    // ── Struct / Enum / Union / Interface / Namespace ───────────────────────

    #[test]
    fn test_print_struct() {
        let s = Struct {
            visibility: Visibility::Public,
            annotations: vec![],
            is_packed: false,
            name: Path::simple("Point".to_string()),
            generic_params: vec![],
            requires: vec![],
            fields: vec![
                StructField { name: "x".to_string(), ty: Type::F32, visibility: Visibility::Default, annotations: vec![] },
                StructField { name: "y".to_string(), ty: Type::F32, visibility: Visibility::Default, annotations: vec![] },
            ],
        };
        let out = print_struct(&s);
        assert!(out.contains("Struct: public Point"));
        assert!(out.contains("Field: x"));
        assert!(out.contains("Field: y"));
    }

    #[test]
    fn test_print_packed_struct() {
        let s = Struct {
            visibility: Visibility::Default,
            annotations: vec![],
            is_packed: true,
            name: Path::simple("Header".to_string()),
            generic_params: vec![],
            requires: vec![],
            fields: vec![],
        };
        assert!(print_struct(&s).contains("packed"));
    }

    #[test]
    fn test_print_enum() {
        let e = Enum {
            visibility: Visibility::Default,
            annotations: vec![],
            name: Path::simple("Color".to_string()),
            representation: None,
            generic_params: vec![],
            requires: vec![],
            variants: vec![
                EnumVariant { name: "Red".to_string(), value: None },
                EnumVariant { name: "Green".to_string(), value: None },
            ],
        };
        let out = print_enum(&e);
        assert!(out.contains("Enum: Color"));
        assert!(out.contains("Variant: Red"));
        assert!(out.contains("Variant: Green"));
    }

    #[test]
    fn test_print_union() {
        let u = Union {
            visibility: Visibility::Default,
            annotations: vec![],
            name: Path::simple("Val".to_string()),
            generic_params: vec![],
            requires: vec![],
            variants: vec![
                UnionVariant { name: "i".to_string(), ty: Type::I32 },
                UnionVariant { name: "f".to_string(), ty: Type::F32 },
            ],
        };
        let out = print_union(&u);
        assert!(out.contains("Union: Val"));
        assert!(out.contains("Variant: i"));
    }

    #[test]
    fn test_print_interface() {
        let iface = Interface {
            visibility: Visibility::Public,
            annotations: vec![],
            name: Path::simple("Display".to_string()),
            generic_params: vec![],
            extends: vec![],
            methods: vec![],
        };
        let out = print_interface(&iface);
        assert!(out.contains("Interface: public Display"));
    }

    #[test]
    fn test_print_namespace() {
        let ns = Namespace {
            visibility: Visibility::Default,
            annotations: vec![],
            name: Path { segments: vec!["mylib".to_string()] },
            items: vec![],
        };
        let out = print_namespace(&ns);
        assert!(out.contains("Namespace: mylib"));
    }

    #[test]
    fn test_print_source_file_empty() {
        let sf = SourceFile { items: vec![] };
        let out = PrettyPrinter::new().print_source_file(&sf);
        assert!(out.contains("SourceFile"));
    }
}