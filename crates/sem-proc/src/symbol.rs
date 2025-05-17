use crate::*;

pub struct Arena<T> {
    data: RefCell<Vec<Rc<T>>>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(vec![]),
        }
    }

    pub fn allocate(&self, value: T) -> Weak<T> {
        let obj = Rc::new(value);
        self.data.borrow_mut().push(obj.clone());
        Rc::downgrade(&obj)
    }
}

pub struct LmtFactory {
    arena: Arena<Symbol1>,
}

impl LmtFactory {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    pub fn create_smtype_slot(&self, name: String) -> Symbol {
        Symbol(self.arena.allocate(Symbol1::SmTypeSlot(Rc::new(SmTypeSlot1 {
            name,
            inherits: RefCell::new(None),
            subtypes: shared_array![],
            fields: shared_map![],
            methods: shared_map![],
            method_output: Rc::new(RefCell::new(proc_macro2::TokenStream::new())),
        }))))
    }

    pub fn create_field_slot(&self, is_ref: bool, name: String, field_type: syn::Type, field_init: syn::Expr) -> Symbol {
        Symbol(self.arena.allocate(Symbol1::FieldSlot(Rc::new(FieldSlot1 {
            is_ref,
            name,
            field_type,
            field_init,
        }))))
    }

    pub fn create_method_slot(&self, name: String, defined_in: Symbol, doc_attribute: Vec<syn::Attribute>) -> Symbol {
        Symbol(self.arena.allocate(Symbol1::MethodSlot(Rc::new(MethodSlot1 {
            name,
            defined_in,
            doc_attribute: RefCell::new(doc_attribute),
            override_logic_mapping: SharedMap::new(),
        }))))
    }
}

#[derive(Clone)]
pub struct Symbol(Weak<Symbol1>);

impl Eq for Symbol {}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl Hash for Symbol {
    /// Performs hashing of the symbol by reference.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}

macro_rules! access {
    ($symbol:expr) => { $symbol.0.upgrade().unwrap().as_ref() };
}

impl Symbol {
    pub fn is_smtype_slot(&self) -> bool {
        matches!(access!(self), Symbol1::SmTypeSlot(_))
    }

    pub fn is_field_slot(&self) -> bool {
        matches!(access!(self), Symbol1::FieldSlot(_))
    }

    pub fn is_method_slot(&self) -> bool {
        matches!(access!(self), Symbol1::MethodSlot(_))
    }

    pub fn name(&self) -> String {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => slot.name.clone(),
            Symbol1::FieldSlot(slot) => slot.name.clone(),
            Symbol1::MethodSlot(slot) => slot.name.clone(),
        }
    }

    pub fn inherits(&self) -> Option<Symbol> {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => slot.inherits.borrow().clone(),
            _ => panic!(),
        }
    }

    pub fn set_inherits(&self, value: Option<&Symbol>) {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => {
                slot.inherits.replace(value.map(|v| v.clone()));
            },
            _ => panic!(),
        }
    }

    pub fn asc_smtype_list(&self) -> Vec<Symbol> {
        let mut out = vec![self.clone()];
        let mut m = self.inherits();
        while let Some(m1) = m {
            out.insert(0, m1.clone());
            m = m1.inherits();
        }
        out
    }

    /// Returns `MN(M2(M1(...)))` layers over a root `Weak<#DATA::FirstM>` value.
    /// 
    /// Parameters:
    /// 
    /// * `base`: A `Weak<#DATA::FirstM>` value.
    pub fn create_layers_over_weak_root(base: &str, asc_smtype_list: &[Symbol]) -> String {
        let mut layers = String::new();
        let mut parens = 0usize;
        for m in asc_smtype_list.iter().rev() {
            let m_name = m.name();
            layers.push_str(&format!("{m_name}("));
            parens += 1;
        }
        layers.push_str(base);
        layers.push_str(".clone()");
        layers.push_str(&")".repeat(parens));
        return layers;
    }

    pub fn lookup_method_in_base_smtype(&self, name: &str) -> Option<Symbol> {
        let mut m = self.clone();
        while let Some(m1) = m.inherits() {
            let mt = m1.methods().get(&name.to_owned());
            if let Some(mt) = mt {
                return Some(mt);
            }
            m = m1;
        }
        None
    }

    pub fn subtypes(&self) -> SharedArray<Symbol> {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => slot.subtypes.clone(),
            _ => panic!(),
        }
    }

    pub fn fields(&self) -> SharedMap<String, Symbol> {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => slot.fields.clone(),
            _ => panic!(),
        }
    }

    pub fn methods(&self) -> SharedMap<String, Symbol> {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => slot.methods.clone(),
            _ => panic!(),
        }
    }

    pub fn method_output(&self) -> Rc<RefCell<proc_macro2::TokenStream>> {
        match access!(self) {
            Symbol1::SmTypeSlot(slot) => slot.method_output.clone(),
            _ => panic!(),
        }
    }

    pub fn field_type(&self) -> syn::Type {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.field_type.clone(),
            _ => panic!(),
        }
    }

    pub fn field_init(&self) -> syn::Expr {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.field_init.clone(),
            _ => panic!(),
        }
    }

    pub fn is_ref(&self) -> bool {
        match access!(self) {
            Symbol1::FieldSlot(slot) => slot.is_ref.clone(),
            _ => panic!(),
        }
    }

    pub fn defined_in(&self) -> Symbol {
        match access!(self) {
            Symbol1::MethodSlot(slot) => slot.defined_in.clone(),
            _ => panic!(),
        }
    }

    pub fn doc_attribute(&self) -> Vec<syn::Attribute> {
        match access!(self) {
            Symbol1::MethodSlot(slot) => slot.doc_attribute.borrow().clone(),
            _ => panic!(),
        }
    }

    pub fn set_doc_attribute(&self, attr: Vec<syn::Attribute>) {
        match access!(self) {
            Symbol1::MethodSlot(slot) => { slot.doc_attribute.replace(attr); },
            _ => panic!(),
        }
    }

    pub fn override_logic_mapping(&self) -> SharedMap<Symbol, Rc<OverrideLogicMapping>> {
        match access!(self) {
            Symbol1::MethodSlot(slot) => slot.override_logic_mapping.clone(),
            _ => panic!(),
        }
    }
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        self.name()
    }
}

enum Symbol1 {
    SmTypeSlot(Rc<SmTypeSlot1>),
    FieldSlot(Rc<FieldSlot1>),
    MethodSlot(Rc<MethodSlot1>),
}

struct SmTypeSlot1 {
    name: String,
    inherits: RefCell<Option<Symbol>>,
    subtypes: SharedArray<Symbol>,
    fields: SharedMap<String, Symbol>,
    methods: SharedMap<String, Symbol>,
    method_output: Rc<RefCell<proc_macro2::TokenStream>>,
}

struct FieldSlot1 {
    name: String,
    field_type: syn::Type,
    field_init: syn::Expr,
    is_ref: bool,
}

struct MethodSlot1 {
    name: String,
    defined_in: Symbol,
    doc_attribute: RefCell<Vec<syn::Attribute>>,
    override_logic_mapping: SharedMap<Symbol, Rc<OverrideLogicMapping>>,
}

pub struct OverrideLogicMapping {
    override_code: RefCell<Option<proc_macro2::TokenStream>>,
    override_logic_mapping: SharedMap<Symbol, Rc<OverrideLogicMapping>>,
}

impl OverrideLogicMapping {
    pub fn new() -> Self {
        Self {
            override_code: RefCell::new(None),
            override_logic_mapping: SharedMap::new(),
        }
    }

    /// Override code; generally a `return` statement with a semicolon.
    pub fn override_code(&self) -> Option<proc_macro2::TokenStream> {
        self.override_code.borrow().clone()
    }

    /// Sets override code; generally a `return` statement with a semicolon.
    pub fn set_override_code(&self, code: Option<proc_macro2::TokenStream>) {
        self.override_code.replace(code);
    }

    /// Mapping from subtype slot to override logic.
    pub fn override_logic_mapping(&self) -> SharedMap<Symbol, Rc<OverrideLogicMapping>> {
        self.override_logic_mapping.clone()
    }
}

/// A data type slot.
/// 
/// # Supported methods
/// 
/// * `is_smtype_slot()` — Returns `true`.
/// * `name()`
/// * `inherits()`
/// * `set_inherits()`
/// * `subtypes()`
/// * `fields()`
/// * `methods()`
/// * `method_output()` — The contents of the `impl` block of the data type.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SmTypeSlot(pub Symbol);

impl Deref for SmTypeSlot {
    type Target = Symbol;
    fn deref(&self) -> &Self::Target {
        assert!(self.0.is_smtype_slot());
        &self.0
    }
}

/// A field slot.
/// 
/// # Supported methods
/// 
/// * `is_field_slot()` — Returns `true`.
/// * `is_ref()`
/// * `name()`
/// * `field_type()`
/// * `field_init()`
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FieldSlot(pub Symbol);

impl Deref for FieldSlot {
    type Target = Symbol;
    fn deref(&self) -> &Self::Target {
        assert!(self.0.is_field_slot());
        &self.0
    }
}

/// A method slot.
/// 
/// # Supported methods
/// 
/// * `is_method_slot()` — Returns `true`.
/// * `name()`
/// * `defined_in()`
/// * `doc_attribute()`
/// * `set_doc_attribute()`
/// * `override_logic_mapping()` — Mapping from subtype slot to override logic.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct MethodSlot(pub Symbol);

impl Deref for MethodSlot {
    type Target = Symbol;
    fn deref(&self) -> &Self::Target {
        assert!(self.0.is_method_slot());
        &self.0
    }
}