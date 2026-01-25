//! # Processor
//!
//! This module provides the core abstraction for async request processing in kanau.
//!
//! ## What is a Processor?
//!
//! A [`Processor`] is a **stateful async function** — it encapsulates both dependencies (state)
//! and behavior (the processing logic) in a single, reusable abstraction. Think of it as:
//!
//! ```text
//! Processor = State + async fn(Input) -> Result<Output, Error>
//! ```
//!
//! This design allows you to bundle configuration, database connections, HTTP clients, or any
//! other dependencies into a struct, then implement [`Processor`] to define how inputs are
//! transformed into outputs.
//!
//! ## Comparison with Tower's `Service`
//!
//! If you're familiar with the [`tower`](https://docs.rs/tower) ecosystem, you'll notice
//! similarities between [`Processor`] and `tower::Service`. Here's how they compare:
//!
//! | Aspect | `tower::Service` | `Processor` |
//! |--------|------------------|-------------|
//! | **Backpressure** | Built-in via `poll_ready()` | Not included (use external mechanisms) |
//! | **Future type** | Associated type `Future` | RPITIT (`impl Future`) |
//! | **Mutability** | Requires `&mut self` | Uses `&self` (immutable) |
//! | **Readiness** | Explicit readiness checking | Always ready |
//! | **Complexity** | Higher (GAT, pinning) | Lower (simpler trait bounds) |
//!
//! However, `tower::Service` is designed for network services where backpressure and readiness are
//! critical concerns. However, this comes with complexity for web applications:
//!
//! - The `poll_ready` / `call` dance requires careful handling
//! - The associated `Future` type often requires boxing or complex GAT patterns
//! - `&mut self` makes sharing across tasks more cumbersome while stateless between tasks or across instances is trivial for web applications.
//!
//! [`Processor`] takes a simpler approach: it assumes the processor is always ready and uses
//! `&self`, making it trivially `Clone`-able and shareable. When you need backpressure, you
//! can compose it with external mechanisms (channels, semaphores, `tower::Service`, etc.) rather than baking
//! it into the trait itself.
//!
//! ## Comparison with Async Closures
//!
//! Rust's async closures (`async |x| { ... }`) are still unstable. Even when stabilized,
//! they have limitations:
//!
//! ```ignore
//! // Hypothetical async closure (unstable)
//! let db = get_database();
//! let handler = async |request| {
//!     db.query(request).await  // captures `db` by reference
//! };
//! ```
//!
//! Problems with async closures:
//!
//! 1. **Lifetime entanglement**: The closure's future borrows captured variables, creating
//!    complex lifetime relationships that are hard to express in trait bounds.
//!
//! 2. **No named type**: You can't easily pass async closures across API boundaries or
//!    store them in structs without `Box<dyn ...>`.
//!
//! 3. **Limited composability**: Middleware patterns become awkward without a concrete type.
//!
//! [`Processor`] solves these by giving you a **named, bounded abstraction**:
//!
//! ```
//! # struct DatabasePool;
//! # struct Request;
//! # struct Response;
//! # struct DbError;
//! # impl DatabasePool {
//! #     async fn query(&self, req: Request) -> Result<Response, DbError> {
//! #         Ok(Response)
//! #     }
//! # }
//! use kanau::processor::Processor;
//!
//! struct MyHandler {
//!     db: DatabasePool,  // owned, not borrowed
//! }
//!
//! impl Processor<Request> for MyHandler {
//!     type Output = Response;
//!     type Error = DbError;
//!
//!     async fn process(&self, req: Request) -> Result<Response, DbError> {
//!         self.db.query(req).await
//!     }
//! }
//! ```

use futures::stream::FuturesUnordered;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio_stream::Stream;

/// A stateful async function abstraction.
///
/// `Processor` is the fundamental building block for async request handling in kanau.
/// It represents a reusable async operation that transforms an input `I` into a
/// `Result<Self::Output, Self::Error>`.
///
/// # Design Philosophy
///
/// Unlike `tower::Service`, `Processor` uses `&self` instead of `&mut self`, making it
/// inherently shareable across tasks without additional synchronization. It also uses
/// return-position `impl Trait` (RPITIT) instead of an associated `Future` type,
/// simplifying implementation and avoiding the need for boxing in most cases.
///
/// # Example
///
/// ```
/// use kanau::processor::Processor;
///
/// struct Greeter {
///     prefix: String,
/// }
///
/// impl Processor<String> for Greeter {
///     type Output = String;
///     type Error = std::convert::Infallible;
///
///     async fn process(&self, name: String) -> Result<String, Self::Error> {
///         let greeting = format!("{}, {}!", self.prefix, name);
///         Ok(greeting)
///     }
/// }
/// ```
pub trait Processor<I: Send> {
    /// The output type of the processor.
    type Output;

    /// The error type of the processor.
    type Error;

    /// This requires the returned future to be `Send`. Generally, you can implement this like this without `async` block:
    ///
    /// ```
    /// # struct MyProcessor;
    /// # struct MyError;
    /// # struct Request;
    /// # struct Response;
    /// use kanau::processor::Processor;
    /// impl Processor<Request> for MyProcessor {
    ///     type Output = Response;
    ///     type Error = MyError;
    ///     async fn process(&self, input: Request) -> Result<Response, MyError> {
    ///         Ok(Response)
    ///     }
    /// }
    /// ```
    fn process(&self, input: I) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

/// Type alias for the return type of processor.
pub type ProcessorReturn<P, I> = Result<<P as Processor<I>>::Output, <P as Processor<I>>::Error>;

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

/// The identity processor — returns its input unchanged.
///
/// `IdentityFunctor` is a zero-cost abstraction that implements [`Processor`] by simply
/// wrapping the input in `Ok(input)`. It serves as the identity element for processor
/// composition chains.
///
/// # Type Parameters
///
/// - `E` — The error type (never actually produced)
///
/// # Example
///
/// ```
/// use kanau::processor::{Processor, IdentityFunctor};
/// # async {
/// let identity: IdentityFunctor<()> = IdentityFunctor::new();
/// let result = identity.process(42).await;
/// assert_eq!(result, Ok(42));
/// # };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct IdentityFunctor<E> {
    _error_phantom: PhantomData<fn() -> E>,
}

impl<E> IdentityFunctor<E> {
    /// Create a new identity functor.
    pub fn new() -> Self {
        Self {
            _error_phantom: PhantomData,
        }
    }
}

impl<I: Send, E> Processor<I> for IdentityFunctor<E> {
    type Output = I;
    type Error = E;
    fn process(&self, input: I) -> impl Future<Output = Result<I, E>> + Send {
        async move { Ok(input) }
    }
}

/// Wraps an async function pointer as a [`Processor`].
///
/// This adapter allows you to use a plain `async fn` as a processor without defining
/// a custom struct. It's useful for simple, stateless transformations.
///
/// # Limitations
///
/// - Only works with function *pointers* (`fn(I) -> Fut`), not closures
/// - Cannot capture state — use a custom struct implementing [`Processor`] for that
///
/// # Example
///
/// ```
/// use kanau::processor::{Processor, AsyncFnProcessor};
///
/// async fn double(x: i32) -> Result<i32, ()> {
///     Ok(x * 2)
/// }
///
/// let processor = AsyncFnProcessor::new(double);
/// # async {
/// let result = processor.process(21).await;
/// assert_eq!(result, Ok(42));
/// # };
/// ```
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

/// A processor variant for task spawning scenarios.
///
/// While [`Processor::process`] takes `&self`, the returned future borrows from the processor.
/// This works fine for `.await`-ing inline, but causes lifetime issues when spawning:
///
/// ```compile_fail
/// use kanau::processor::Processor;
/// use tokio::spawn;
/// struct MyProcessor;
///
/// impl Processor<i32> for MyProcessor {
///     type Output = i32;
///     type Error = ();
///     async fn process(&self, input: i32) -> Result<i32, ()> {
///         Ok(input * 2)
///     }
/// }
///
/// let processor = MyProcessor;
/// let input = 42;
/// // This won't compile — future borrows from `processor`
/// tokio::spawn(processor.process(input));
/// ```
///
/// > requirement that the value outlives `'static`
///
/// `ArcProcessor` solves this by taking `Arc<Self>` instead of `&self`. The `Arc` is moved
/// into the future, ensuring the processor lives as long as the future needs it:
///
/// ```
/// # use kanau::processor::{Processor, ArcProcessor};
/// # use std::sync::Arc;
/// # struct MyProcessor;
/// # impl Processor<i32> for MyProcessor {
///     # type Output = i32;
///     # type Error = ();
///     # async fn process(&self, input: i32) -> Result<i32, ()> {
///     #     Ok(input * 2)
///     # }
/// # }
/// // This works — Arc is moved into the spawned task
/// # #[tokio::main]
/// # async fn main() {
/// let processor = Arc::new(MyProcessor);
/// # let input = 42;
/// tokio::spawn(ArcProcessor::process(Arc::clone(&processor), input));
/// # }
/// ```
///
/// # Blanket Implementation
///
/// Any type implementing `Processor<I> + Send + Sync` automatically implements `ArcProcessor<I>`.
/// You rarely need to implement this trait directly.
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

impl<I, P: ?Sized> ArcProcessor<I> for P
where
    P: Processor<I> + Sync + Send,
    I: Send,
{
    type Output = P::Output;
    type Error = P::Error;
    async fn process(state: Arc<Self>, input: I) -> Result<P::Output, P::Error> {
        state.as_ref().process(input).await
    }
}

/// Applies a processor to each item in an iterator, executing all operations concurrently.
///
/// This is the async equivalent of `iter.map(f)`, but all futures run in parallel rather
/// than sequentially. Results are yielded as they complete, so **ordering is not preserved**.
///
/// # Arguments
///
/// - `iter` — An iterator yielding input items
/// - `ref_processor` — A shared reference to the processor
///
/// # Returns
///
/// A [`Stream`] of results. Items may arrive in any order depending on how long each
/// operation takes.
///
/// # Example
///
/// ```
/// use kanau::processor::{Processor, parallel_map};
/// use tokio_stream::StreamExt;
/// # struct MyProcessor;
/// impl Processor<i32> for MyProcessor {
///     type Output = i32;
///     type Error = ();
///     async fn process(&self, input: i32) -> Result<i32, ()> {
///         Ok(input * 2)
///     }
/// }
/// # impl MyProcessor {
/// #     fn new() -> Self { MyProcessor }
/// # }
///
/// # #[tokio::main]
/// # async fn main() {
/// let processor = MyProcessor::new();
/// let inputs = vec![1, 2, 3, 4, 5];
///
/// let mut stream = parallel_map(inputs.into_iter(), &processor);
/// while let Some(result) = stream.next().await {
///     assert!(result.is_ok());
///     assert!(result.unwrap() % 2 == 0);
/// }
/// # }
/// ```
///
/// # Performance Note
///
/// Avoid N+1 queries. When you are trying to operate with database or a large number of items,
/// this function is not encouraged.
///
/// All futures are spawned immediately into a [`FuturesUnordered`], which polls them
/// cooperatively. For CPU-bound work, consider using `tokio::spawn_blocking` or a thread pool instead.
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
