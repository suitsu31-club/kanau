use futures::stream::{FuturesUnordered};
use std::sync::Arc;
use tokio_stream::Stream;

/// # Processor
///
/// A Processor is one of the fundamental building blocks in kanau.
/// In essence, a Processor is a durable async operation handler - think of it as a persistent
/// async function with dependencies bundled in.
///
/// While Rust’s async closures are still unstable, a Processor achieves similar functionality
/// through a trait-based approach. It encapsulates both state (dependencies) and behavior
/// (the processing logic) in a single abstraction.
pub trait Processor<I, O> {
    #[allow(missing_docs)]
    fn process<'a>(&'a self, input: I) -> impl Future<Output = O> + Send + 'a where I: 'a;
}

// an async function is also a processor
impl<I: Send, O: Send, F: Future<Output = O> + Send> Processor<I, O> for fn(I) -> F {
    fn process<'a>(&'a self, input: I) -> impl Future<Output = O> + Send + 'a where I: 'a {
        (self)(input)
    }
}

/// ## FinalProcessor
///
/// A variant of processor to solve lifetime issues.
///
/// The key difference is that [FinalProcessor] takes an `Arc<Self>` as state, not `&Self`,
/// ensuring the processor outlives the `Future` it returns. This is particularly useful in
/// cases where the future needs to live independently of the original context, such as
/// when it's spawned into a new task.
pub trait FinalProcessor<I, O> {
    #[allow(missing_docs)]
    fn process(state: Arc<Self>, input: I) -> impl Future<Output = O> + Send;
}

/// ## RefProcessor
///
/// A variant of processor that receive a borrowed reference as an argument.
///
/// This is useful when the input is a reference that needs to be borrowed for the duration
/// of the processing.
pub trait RefProcessor<Borrowed, O, Owned = ()> {
    #[allow(missing_docs)]
    fn process<'a, 'b>(
        &'a self,
        deps: &'b Borrowed,
        input: Owned,
    ) -> impl Future<Output = O> + Send + 'a + 'b
    where
        Owned: 'a + 'b;
}

/// ## Parallel Map (borrowed version)
/// 
/// `map` function, but for async functions.
/// 
/// These async functions are executed in parallel.
/// 
/// ## Arguments
/// 
/// - `iter` - An iterator that yields references to the input items.
/// - `ref_processor` - A reference of [RefProcessor] that will be used to process the input items.
/// 
/// ## Returns
/// 
/// A stream of output items. The order of the output items is *not guaranteed to be the same* as the input items.
pub fn parallel_map_borrowed<'input, I, O, RP, Iter>(
    iter: Iter,
    ref_processor: &RP,
) -> impl Stream<Item = O> + Send
where
    I: Send + Sync + 'input,
    O: Send + Sync,
    RP: RefProcessor<I, O> + Send + Sync,
    Iter: Iterator<Item = &'input I> + Send + Sync ,
{
    let set: FuturesUnordered<_> = iter
        .map(|input| ref_processor.process(input, ()))
        .collect();
    set
}

/// ## Parallel Map (owned version)
/// 
/// `map` function, but for async functions.
/// 
/// These async functions are executed in parallel.
/// 
/// ## Arguments
/// 
/// - `iter` - An iterator that yields the input items.
/// - `ref_processor` - A reference of [Processor] that will be used to process the input items.
/// 
/// ## Returns
/// 
/// A stream of output items. The order of the output items is *not guaranteed to be the same* as the input items.
pub fn parallel_map<'p,I, O, P, Iter>(
    iter: Iter,
    ref_processor: &'p P,
) -> impl Stream<Item = O> + Send + 'p
where
    I: Send + Sync + 'p,
    O: Send + Sync,
    P: Processor<I, O> + Send + Sync,
    Iter: Iterator<Item = I> + Send + Sync,
{
    let set: FuturesUnordered<_> = iter
        .map(|input| ref_processor.process(input))
        .collect();
    set
}