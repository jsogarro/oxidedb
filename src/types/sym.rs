use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

/// Interned symbol: a 4-byte handle into a process-global string table.
/// Index 0 is the null symbol (the empty string).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sym(u32);

#[derive(Default)]
struct Interner {
    ids: HashMap<&'static str, u32>,
    names: Vec<&'static str>,
}

fn interner() -> &'static Mutex<Interner> {
    static INTERNER: OnceLock<Mutex<Interner>> = OnceLock::new();
    INTERNER.get_or_init(|| {
        let mut i = Interner::default();
        i.ids.insert("", 0);
        i.names.push("");
        Mutex::new(i)
    })
}

impl Sym {
    /// The null symbol, `` ` ``.
    pub const NULL: Sym = Sym(0);

    pub fn intern(s: &str) -> Sym {
        let mut i = interner().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(&id) = i.ids.get(s) {
            return Sym(id);
        }
        // ponytail: interned names are leaked and never freed; add a reclaiming table if symbol churn matters.
        let name: &'static str = Box::leak(s.to_owned().into_boxed_str());
        let id = i.names.len() as u32;
        i.names.push(name);
        i.ids.insert(name, id);
        Sym(id)
    }

    pub fn as_str(self) -> &'static str {
        let i = interner().lock().unwrap_or_else(|e| e.into_inner());
        i.names[self.0 as usize]
    }
}

impl fmt::Display for Sym {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}", self.as_str())
    }
}
