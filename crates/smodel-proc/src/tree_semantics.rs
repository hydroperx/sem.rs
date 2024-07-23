use crate::*;

pub struct TreeSemantics<T> {
    data_types: RefCell<HashMap<ByAddress<Rc<SmType>>, Option<T>>>,
    methods: RefCell<HashMap<ByAddress<Rc<SmTypeMethod>>, Option<T>>>,
    fields: RefCell<HashMap<ByAddress<Rc<SmTypeField>>, Option<T>>>,
}

impl<T> TreeSemantics<T> {
    pub fn new() -> Self {
        Self {
            data_types: RefCell::new(HashMap::new()),
            methods: RefCell::new(HashMap::new()),
            fields: RefCell::new(HashMap::new()),
        }
    }
}

pub trait TreeSemanticsAccessor<T, S: Clone> {
    fn get(&self, node: &Rc<T>) -> Option<S>;
    fn set(&self, node: &Rc<T>, symbol: Option<S>);
    fn _delete(&self, node: &Rc<T>) -> bool;
    fn _has(&self, node: &Rc<T>) -> bool;
}

impl<S: Clone> TreeSemanticsAccessor<SmType, S> for TreeSemantics<S> {
    fn get(&self, node: &Rc<SmType>) -> Option<S> {
        self.data_types.borrow().get(&ByAddress(node.clone())).and_then(|v| v.clone())
    }
    fn set(&self, node: &Rc<SmType>, symbol: Option<S>) {
        self.data_types.borrow_mut().insert(ByAddress(node.clone()), symbol);
    }
    fn _delete(&self, node: &Rc<SmType>) -> bool {
        self.data_types.borrow_mut().remove(&ByAddress(node.clone())).is_some()
    }
    fn _has(&self, node: &Rc<SmType>) -> bool {
        self.data_types.borrow().contains_key(&ByAddress(node.clone()))
    }
}

impl<S: Clone> TreeSemanticsAccessor<SmTypeField, S> for TreeSemantics<S> {
    fn get(&self, node: &Rc<SmTypeField>) -> Option<S> {
        self.fields.borrow().get(&ByAddress(node.clone())).and_then(|v| v.clone())
    }
    fn set(&self, node: &Rc<SmTypeField>, symbol: Option<S>) {
        self.fields.borrow_mut().insert(ByAddress(node.clone()), symbol);
    }
    fn _delete(&self, node: &Rc<SmTypeField>) -> bool {
        self.fields.borrow_mut().remove(&ByAddress(node.clone())).is_some()
    }
    fn _has(&self, node: &Rc<SmTypeField>) -> bool {
        self.fields.borrow().contains_key(&ByAddress(node.clone()))
    }
}

impl<S: Clone> TreeSemanticsAccessor<SmTypeMethod, S> for TreeSemantics<S> {
    fn get(&self, node: &Rc<SmTypeMethod>) -> Option<S> {
        self.methods.borrow().get(&ByAddress(node.clone())).and_then(|v| v.clone())
    }
    fn set(&self, node: &Rc<SmTypeMethod>, symbol: Option<S>) {
        self.methods.borrow_mut().insert(ByAddress(node.clone()), symbol);
    }
    fn _delete(&self, node: &Rc<SmTypeMethod>) -> bool {
        self.methods.borrow_mut().remove(&ByAddress(node.clone())).is_some()
    }
    fn _has(&self, node: &Rc<SmTypeMethod>) -> bool {
        self.methods.borrow().contains_key(&ByAddress(node.clone()))
    }
}