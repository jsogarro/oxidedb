use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

/// Interned symbol: a 4-byte handle into a process-global string table.
/// Index 0 is the null symbol (the empty string).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sym(u32);

/// Name -> index, only touched by `intern` (under the mutex).
#[derive(Default)]
struct Interner {
    ids: HashMap<&'static str, u32>,
}

/// Index -> name (from 1; the null symbol is special-cased), append-only and read without a lock. Segment `k` holds the `2^k` names whose
/// `index + 1` has its top bit at `k` (33 segments cover every `u32` handle); a name is written once, before the `Sym` that points at it
/// escapes, so `as_str` is two atomic loads.
type Segment = Box<[OnceLock<&'static str>]>;
static NAMES: [OnceLock<Segment>; 33] = [const { OnceLock::new() }; 33];

/// (segment, offset) of the name at `index`.
fn slot(index: u32) -> (usize, usize) {
    let n = u64::from(index) + 1;
    let k = n.ilog2() as usize;
    (k, (n - (1 << k)) as usize)
}

fn set_name(index: u32, name: &'static str) {
    let (k, off) = slot(index);
    let seg = NAMES[k].get_or_init(|| (0..1usize << k).map(|_| OnceLock::new()).collect());
    seg[off].set(name).expect("symbol slot written twice");
}

fn interner() -> &'static Mutex<Interner> {
    static INTERNER: OnceLock<Mutex<Interner>> = OnceLock::new();
    INTERNER.get_or_init(|| {
        let mut i = Interner::default();
        i.ids.insert("", 0);
        Mutex::new(i)
    })
}

/// The next symbol index. Running out of the 2^32 handles is a hard limit of the 4-byte `Sym`,
/// so it panics with a clear message rather than wrapping onto an existing symbol.
fn next_id(len: usize) -> u32 {
    u32::try_from(len).expect("symbol table full: more than u32::MAX distinct symbols")
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
        let id = next_id(i.ids.len());
        set_name(id, name);
        i.ids.insert(name, id);
        Sym(id)
    }

    pub fn as_str(self) -> &'static str {
        if self == Sym::NULL {
            return ""; // usable before anything is interned
        }
        let (k, off) = slot(self.0);
        // A `Sym` only exists after `intern` stored its name.
        NAMES[k].get().expect("symbol segment")[off]
            .get()
            .expect("symbol name")
    }
}

impl fmt::Display for Sym {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id_is_checked() {
        assert_eq!(next_id(7), 7);
        assert_eq!(next_id(u32::MAX as usize), u32::MAX);
        assert!(std::panic::catch_unwind(|| next_id(u32::MAX as usize + 1)).is_err());
    }

    #[test]
    fn slots_tile_the_index_space() {
        assert_eq!(slot(0), (0, 0));
        assert_eq!(slot(1), (1, 0));
        assert_eq!(slot(2), (1, 1));
        assert_eq!(slot(3), (2, 0));
        assert_eq!(slot(6), (2, 3));
        assert_eq!(slot(7), (3, 0));
        assert_eq!(slot(u32::MAX), (32, 0)); // the last handle; its segment is allocated only if ever reached
    }

    #[test]
    fn concurrent_intern_and_read_agree() {
        let threads: Vec<_> = (0..8)
            .map(|t| {
                std::thread::spawn(move || {
                    // overlapping name sets, enough to cross several segments
                    (0..600)
                        .map(|i| Sym::intern(&format!("conc{}", (i * 7 + t) % 700)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for syms in threads.into_iter().flat_map(|h| h.join().unwrap()) {
            assert_eq!(Sym::intern(syms.as_str()), syms);
            assert!(syms.as_str().starts_with("conc"));
        }
    }
}
