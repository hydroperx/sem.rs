use crate::*;

pub struct SModelHost {
    pub factory: LmtFactory,
    pub semantics: TreeSemantics<Symbol>,
    pub smtype_slots: HashMap<String, Symbol>,
    pub output: TokenStream,
    pub data_output: proc_macro2::TokenStream,
}

impl SModelHost {
    pub fn new() -> Self {
        Self {
            factory: LmtFactory::new(),
            semantics: TreeSemantics::new(),
            smtype_slots: HashMap::new(),
            output: TokenStream::new(),
            data_output: proc_macro2::TokenStream::new(),
        }
    }
}