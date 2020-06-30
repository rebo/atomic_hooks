use crate::prelude::*;
use seed::prelude::*;

pub trait LocalUpdateEl2<T> {
    fn update_el(self, el: &mut T);
}

impl<Ms: 'static,T,U,A> LocalUpdateEl2<El<Ms>> for ReactiveStateAccess<T,U,A> where T: UpdateEl<Ms> + 'static + Clone{
    fn update_el(self, el: &mut El<Ms>) {
        self.get().update_el(el);
    }
}



#[derive(Clone,Copy,Debug)]
pub struct Local(topo::Id);

impl std::fmt::Display for Local{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

/// A value unique to the source location where it is created.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Callsite {
    location: usize,
}

impl Callsite {
    /// Constructs a callsite whose value is unique to the source location at
    /// which it is called.
    #[track_caller]
    pub fn here() -> Self {
        Self {
            // the pointer value for a given location is enough to differentiate it from all others
            location: Location::caller() as *const _ as usize,
        }
    }
}

impl Local{
    #[topo::nested]
    pub fn current() -> Local{
        Local(topo::Id::current())
    }
}


