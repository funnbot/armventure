use core::hash::BuildHasherDefault;
use lasso::Rodeo;
use rustc_hash::FxHasher;

#[allow(dead_code)]
pub mod key {
    /// 64-bit key
    pub type Key64 = lasso::LargeSpur;
    /// 32-bit key
    pub type Key32 = lasso::Spur;
    /// 16-bit key
    pub type Key16 = lasso::MiniSpur;
    /// 8-bit key
    pub type Key8 = lasso::MicroSpur;
}

/// intern new strings, string to key and key to string resolving
pub type Intern<Key> = Rodeo<Key, BuildHasherDefault<FxHasher>>;
/// readonly, string to key and key to string resolving
pub type InternReader<Key> = lasso::RodeoReader<Key, BuildHasherDefault<FxHasher>>;
/// readonly, only key to string resolving
pub type InternResolver<Key> = lasso::RodeoResolver<Key>;

#[macro_export]
macro_rules! typed_interner {
    { $mod_name:ident; $key_size:ident } => {
        #[allow(dead_code)]
        pub mod $mod_name {
            use $crate::intern;

            type InternalKey = intern::key::$key_size;

            #[derive(Clone, Copy, PartialEq, Eq, Hash)]
            pub struct Key(InternalKey);

            pub struct Intern(intern::Intern<InternalKey>);
            pub struct InternReader(intern::InternReader<InternalKey>);
            pub struct InternResolver(intern::InternResolver<InternalKey>);

            pub struct KeyInternPair<'a>(Key, &'a Intern);
            impl<'a> KeyInternPair<'a> {
                pub fn new(key: Key, intern: &'a Intern) -> Self {
                    Self(key, intern)
                }
            }
            impl ::std::fmt::Display for KeyInternPair<'_> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}", self.1.resolve(self.0))
                }
            }
            impl ::std::fmt::Debug for Key {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}", ::lasso::Key::into_usize(self.0))
                }
            }

            // TODO: use lasso traits
            impl Intern {
                #[inline]
                pub fn new() -> Self {
                    Self(intern::Intern::with_hasher(Default::default()))
                }
                pub fn get_or_intern<Str: AsRef<str>>(&mut self, s: Str) -> Key {
                    Key(self.0.get_or_intern(s))
                }
                #[inline]
                pub fn get_or_intern_static(&mut self, s: &'static str) -> Key {
                    Key(self.0.get_or_intern_static(s))
                }
                #[inline]
                pub fn resolve(&self, key: Key) -> &str {
                    debug_assert!(self.0.contains_key(&key.0));
                    // SAFETY: key is type checked to be from this interner
                    unsafe { self.0.resolve_unchecked(&key.0) }
                }
                pub fn contains<Str: AsRef<str>>(&self, s: Str) -> bool {
                    self.0.contains(s)
                }
                #[inline]
                pub fn len(&self) -> usize {
                    self.0.len()
                }
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }
                #[inline]
                pub fn into_reader(self) -> InternReader {
                    InternReader(self.0.into_reader())
                }
                #[inline]
                pub fn into_resolver(self) -> InternResolver {
                    InternResolver(self.0.into_resolver())
                }
            }

            impl InternReader {
                pub fn get<Str: AsRef<str>>(&mut self, s: Str) -> Option<Key> {
                    self.0.get(s).map(Key)
                }
                #[inline]
                pub fn resolve(&self, key: Key) -> &str {
                    debug_assert!(self.0.contains_key(&key.0));
                    // SAFETY: key is type checked to be from this interner
                    unsafe { self.0.resolve_unchecked(&key.0) }
                }
                pub fn contains<Str: AsRef<str>>(&self, s: Str) -> bool {
                    self.0.contains(s)
                }
                #[inline]
                pub fn into_resolver(self) -> InternResolver {
                    InternResolver(self.0.into_resolver())
                }
                #[inline]
                pub fn len(&self) -> usize {
                    self.0.len()
                }
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }
            }

            impl InternResolver {
                #[inline]
                pub fn resolve(&self, key: Key) -> &str {
                    debug_assert!(self.0.contains_key(&key.0));
                    // SAFETY: key is type checked to be from this interner
                    unsafe { self.0.resolve_unchecked(&key.0) }
                }
                #[inline]
                pub fn len(&self) -> usize {
                    self.0.len()
                }
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }
            }

            impl Key {
                /// # Safety
                /// key must be from the same interner
                #[inline]
                pub unsafe fn new_unchecked(key: InternalKey) -> Self {
                    Self(key)
                }
            }
        }
    };
}
