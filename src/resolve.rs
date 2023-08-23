use std::rc::Rc;

#[derive(Debug)]
pub enum PushError {
    AlreadyUsed,
}
#[derive(Debug)]
pub enum SetError {
    AlreadyResolved,
    AlreadyHooked,
}
#[derive(Debug)]
pub enum GetError {
    NotResolved,
    Empty,
}
#[derive(Debug)]
pub enum ResolveError {
    Waiting,
    Empty,
    AlreadyResolved,
}

pub struct Resolver<T>(Rc<Option<T>>);

impl<T> Resolver<T> {
    pub fn push(&mut self, value: T) -> Result<(), PushError> {
        match Rc::get_mut(&mut self.0) {
            Some(mut_ref) => {
                if mut_ref.is_some() {
                    Err(PushError::AlreadyUsed)
                } else {
                    mut_ref.insert(value);
                    Ok(())
                }
            }
            None => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum ResolveInner<T> {
    Waiting(Rc<Option<T>>),
    Value(T),
    Empty,
}

pub enum ResolveResult<T> {
    Value(T),
    Waiting,
    Empty,
}

#[derive(Debug)]
pub struct Resolvable<T>(ResolveInner<T>);

impl<T> Resolvable<T> {
    pub fn new() -> Self {
        Self(ResolveInner::Empty)
    }
    pub fn new_with(value: T) -> Self {
        Self(ResolveInner::Value(value))
    }
    pub fn hook(&mut self) -> Result<Resolver<T>, SetError> {
        match self.0 {
            ResolveInner::Waiting(..) => Err(SetError::AlreadyHooked),
            ResolveInner::Value(..) => Err(SetError::AlreadyResolved),
            ResolveInner::Empty => {
                let rc = Rc::<Option<T>>::new(None);
                self.0 = ResolveInner::Waiting(rc.clone());
                Ok(Resolver::<T>(rc))
            }
        }
    }
    pub fn set(&mut self, value: T) -> Result<(), SetError> {
        match self.0 {
            ResolveInner::Waiting(..) => Err(SetError::AlreadyHooked),
            ResolveInner::Value(..) => Err(SetError::AlreadyResolved),
            ResolveInner::Empty => {
                self.0 = ResolveInner::Value(value);
                Ok(())
            }
        }
    }
    pub fn resolve(&mut self) -> Result<(), ResolveError> {
        match self.0 {
            ResolveInner::Waiting(ref mut rc) => match Rc::get_mut(rc) {
                Some(rc_inner) => match rc_inner.take() {
                    Some(value) => {
                        self.0 = ResolveInner::Value(value);
                        Ok(())
                    }
                    None => Err(ResolveError::Waiting),
                },
                None => unreachable!(),
            },
            ResolveInner::Value(..) => Err(ResolveError::AlreadyResolved),
            ResolveInner::Empty => Err(ResolveError::Empty),
        }
    }
    pub fn get(&self) -> Result<&T, GetError> {
        match self.0 {
            ResolveInner::Waiting(..) => Err(GetError::NotResolved),
            ResolveInner::Empty => Err(GetError::Empty),
            ResolveInner::Value(ref value) => Ok(value),
        }
    }
}
