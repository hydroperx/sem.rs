use std::{cell::RefCell, rc::{Rc, Weak}};
use std::fmt::Debug;

pub mod util;

pub use hydroperfox_smodel_proc::smodel;

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

    /// Frees dead objects from the arena. Note that a call to `clean()`
    /// may be expensive; therefore it is recommended to call it after a long
    /// processing has been done with the arena.
    pub fn clean(&self) {
        let mut data = self.data.borrow_mut();
        let mut i = data.len();
        while i != 0 {
            i -= 1;
            let obj = data.get(i).unwrap();
            if Rc::weak_count(obj) == 0 && Rc::strong_count(obj) == 1 {
                data.remove(i);
            }
        }
    }
}

#[derive(Debug)]
pub enum SModelError {
    Contravariant,
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use crate::smodel;

        smodel! {
            mod smodel = crate;

            type Arena = Arena;
        
            /// My unified data type.
            struct Thingy {
                pub fn Thingy() {
                    super();
                }
        
                /// Empty, Foo, FooBar or FooQux
                pub fn name(&self) -> String {
                    "".into()
                }

                pub fn base_example(&self) -> String {
                    "from base".into()
                }

                pub fn x(&self) -> f64 {
                    0.0
                }
            }
        
            struct Foo: Thingy {
                pub fn Foo() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "Foo".into()
                }
            }
        
            struct FooBar: Foo {
                pub fn FooBar() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooBar".into()
                }

                pub override fn base_example(&self) -> String {
                    format!("from bar; {}", super.base_example())
                }
            }
            
            struct FooBarBar: FooBar {
                let m_x: f64 = 0.0;
                let ref m_y: String = "".into();

                pub fn FooBarBar(x: f64, y: &str) {
                    super();
                    self.set_m_x(x);
                    self.set_m_y(y.into());
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooBarBar".into()
                }

                pub override fn x(&self) -> f64 {
                    self.m_x()
                }

                pub override fn base_example(&self) -> String {
                    format!("from {}; {}", self.m_y(), super.base_example())
                }
            }
        
            struct FooQux: Foo {
                pub fn FooQux() {
                    super();
                }

                #[inheritdoc]
                pub override fn name(&self) -> String {
                    "FooQux".into()
                }
            }
        }

        let arena = Arena::new();

        let symbol = Foo::new(&arena);
        let base_symbol: Thingy = symbol.into();
        assert_eq!("Foo", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(false, base_symbol.is::<FooBar>());
        assert_eq!(false, base_symbol.is::<FooQux>());
        assert_eq!("from base", base_symbol.base_example());
        assert_eq!(0.0, base_symbol.x());

        let symbol = FooBar::new(&arena);
        let base_symbol: Thingy = symbol.into();
        assert_eq!("FooBar", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(true, base_symbol.is::<FooBar>());
        assert_eq!(false, base_symbol.is::<FooBarBar>());
        assert_eq!(false, base_symbol.is::<FooQux>());
        assert_eq!("from bar; from base", base_symbol.base_example());
        assert_eq!(0.0, base_symbol.x());

        let symbol = FooBarBar::new(&arena, 10.0, "bar bar");
        let base_symbol: Thingy = symbol.into();
        assert_eq!("FooBarBar", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(true, base_symbol.is::<FooBar>());
        assert_eq!(true, base_symbol.is::<FooBarBar>());
        assert_eq!(false, base_symbol.is::<FooQux>());
        assert_eq!("from bar bar; from bar; from base", base_symbol.base_example());
        assert_eq!(10.0, base_symbol.x());

        let symbol = FooQux::new(&arena);
        let base_symbol: Thingy = symbol.into();
        assert_eq!("FooQux", base_symbol.name());
        assert_eq!(true, base_symbol.is::<Foo>());
        assert_eq!(false, base_symbol.is::<FooBar>());
        assert_eq!(true, base_symbol.is::<FooQux>());
        assert_eq!(0.0, base_symbol.x());
    }
}