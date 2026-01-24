use futures::stream::FuturesUnordered;
use std::marker::PhantomData;
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
pub trait Processor<I: Send> {
    /// The output type of the processor.
    type Output;

    /// The error type of the processor.
    type Error;
    #[allow(missing_docs)]
    fn process(&self, input: I) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

/// Type alias for the return type of processor.
pub type ProcessorReturn<P: Processor<I>, I> =
    Result<<P as Processor<I>>::Output, <P as Processor<I>>::Error>;

impl<P, I> Processor<I> for &P
where
    P: Processor<I> + ?Sized,
    I: Send,
{
    type Output = P::Output;
    type Error = P::Error;
    fn process(&self, request: I) -> impl Future<Output = Result<P::Output, P::Error>> + Send {
        (**self).process(request)
    }
}

impl<P, I> Processor<I> for &mut P
where
    P: Processor<I> + ?Sized,
    I: Send,
{
    type Output = P::Output;
    type Error = P::Error;
    fn process(&self, request: I) -> impl Future<Output = Result<P::Output, P::Error>> + Send {
        (**self).process(request)
    }
}

#[derive(Debug, Clone, Copy)]
/// ## IdentityFunctor
///
/// An identity functor. It implements [Processor] and [ArcProcessor] and return
/// the input as is without any changes, wrapping it in `Result<I, E>`.
///
/// This is zero-cost abstraction of identical transformation, also used as identity of [ServiceChain].
pub struct IdentityFunctor<I, E> {
    _input_phantom: PhantomData<fn(I) -> Result<I, E>>,
}

impl<I, E> IdentityFunctor<I, E> {
    /// Create a new identity functor.
    pub fn new() -> Self {
        Self {
            _input_phantom: PhantomData,
        }
    }
}

impl<I: Send, E> Processor<I> for IdentityFunctor<I, E> {
    type Output = I;
    type Error = E;
    fn process(&self, input: I) -> impl Future<Output = Result<I, E>> + Send {
        async move { Ok(input) }
    }
}

impl<I: Send, E> ArcProcessor<I> for IdentityFunctor<I, E> {
    type Output = I;
    type Error = E;
    fn process(_state: Arc<Self>, input: I) -> impl Future<Output = Result<I, E>> + Send {
        async move { Ok(input) }
    }
}

/// ## AsyncFnProcessor
///
/// A processor that is created from an async function.
pub struct AsyncFnProcessor<I, O, E, Fut: Future<Output = Result<O, E>> + Send> {
    f: fn(I) -> Fut,
    _input_phantom: PhantomData<I>,
    _output_phantom: PhantomData<Result<O, E>>,
}

impl<I, O, E, Fut: Future<Output = Result<O, E>> + Send> AsyncFnProcessor<I, O, E, Fut> {
    /// Create a new async fn processor.
    pub fn new(f: fn(I) -> Fut) -> Self {
        Self {
            f,
            _input_phantom: PhantomData,
            _output_phantom: PhantomData,
        }
    }
}

impl<I: Send, O, E, Fut: Future<Output = Result<O, E>> + Send> Processor<I>
    for AsyncFnProcessor<I, O, E, Fut>
{
    type Output = O;
    type Error = E;
    fn process(&self, input: I) -> impl Future<Output = Result<O, E>> + Send {
        (self.f)(input)
    }
}

/// ## ArcProcessor
///
/// A variant of processor to solve lifetime issues.
///
/// The key difference is that [ArcProcessor] takes an `Arc<Self>` as state, not `&Self`,
/// ensuring the processor outlives the `Future` it returns. This is particularly useful in
/// cases where the future needs to live independently of the original context, such as
/// when it's spawned into a new task.
pub trait ArcProcessor<I> {
    /// The output type of the processor.
    type Output;
    /// The error type of the processor.
    type Error;
    #[allow(missing_docs)]
    fn process(
        state: Arc<Self>,
        input: I,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

/// ## Parallel Map
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
pub fn parallel_map<'p, I, P, Iter>(
    iter: Iter,
    ref_processor: &'p P,
) -> impl Stream<Item = ProcessorReturn<P, I>> + Send + 'p
where
    I: Send + Sync + 'p,
    P: Processor<I> + Send + Sync,
    Iter: Iterator<Item = I> + Send + Sync,
{
    let set: FuturesUnordered<_> = iter.map(|input| ref_processor.process(input)).collect();
    set
}
