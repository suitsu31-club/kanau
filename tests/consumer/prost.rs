//! No direct codec dependency. The generated impls are conditional on
//! `Self` implementing the codec traits, which `Msg<T>` never does, so this only
//! compiles if every path the derives emit resolves through `kanau`.

use kanau::{ProstMessageDe, ProstMessageSer};

#[derive(ProstMessageDe, ProstMessageSer)]
pub struct Msg<T>(pub T);

fn main() {}
