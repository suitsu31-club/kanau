//! # Layers and Adapters
//!
//! This module provides abstractions for wrapping and adapting [`Processor`]s, enabling
//! middleware-like patterns and type transformations in processing pipelines.
//!
//! ## Overview
//!
//! Layers sit between the caller and the processor, intercepting requests and/or responses.
//! This is useful for implementing cross-cutting concerns like:
//!
//! - **Logging**: Record inputs/outputs for debugging
//! - **Metrics**: Measure processing latency and success rates
//! - **Validation**: Check inputs before forwarding to the processor
//! - **Caching**: Short-circuit processing for cached results
//! - **Retry/Circuit-breaker**: Handle transient failures gracefully
//!
//! ## Core Abstractions
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`Proxy`] | Trait for wrapping a processor with middleware logic |
//! | [`ProxiedProcessor`] | Combines a processor with a proxy into a new processor |
//! | [`Adapter`] | Transforms input/output types using processor-based converters |
//! | [`PureAdapter`] | Lightweight type conversion using pure functions |
//!
//! ## Proxy vs Adapter
//!
//! **Proxy** wraps a processor to add behavior (logging, caching, etc.) while keeping
//! the same input/output types. Think of it as "around advice" in AOP terminology.
//!
//! **Adapter** transforms types at the boundaries. Use it when you need to convert
//! between different representations (e.g., HTTP request → domain model → HTTP response).

use crate::chain::{ProcessorPureFunctionChain, ServiceChain, ServiceChain3};
use crate::processor::{Processor, ProcessorReturn};
use std::fmt::Debug;
use std::marker::PhantomData;

/// A middleware abstraction for intercepting processor calls.
///
/// `Proxy` enables the "decorator" or "middleware" pattern: you can wrap any processor
/// to add behavior before and/or after the core processing logic, without modifying
/// the processor itself.
///
/// # Design Notes
///
/// - The proxy receives both the processor reference and the input
/// - It has full control: can modify input, skip the processor entirely, modify output
/// - The output type must match the wrapped processor's output type
///
/// # Example
///
/// ```
/// use kanau::processor::Processor;
/// use kanau::layer::{Proxy, ProxiedProcessor};
///
/// struct TimingProxy;
///
/// impl<I: Send, P: Processor<I> + Sync> Proxy<I, P> for TimingProxy {
///     async fn wrap(&self, processor: &P, input: I) -> Result<P::Output, P::Error> {
///         let start = std::time::Instant::now();
///         let result = processor.process(input).await;
///         println!("Processing took {:?}", start.elapsed());
///         result
///     }
/// }
///
/// // Usage: wrap any processor with timing
/// # struct MyProcessor;
/// # impl Processor<i32> for MyProcessor {
/// #     type Output = i32;
/// #     type Error = ();
/// #     async fn process(&self, x: i32) -> Result<i32, ()> { Ok(x) }
/// # }
/// let timed = ProxiedProcessor::new(TimingProxy, MyProcessor);
/// ```
pub trait Proxy<I: Send, P: Processor<I>> {
    /// Intercepts a processor call, optionally modifying behavior.
    ///
    /// # Arguments
    ///
    /// - `processor` — The wrapped processor to delegate to
    /// - `input` — The input to process
    ///
    /// # Returns
    ///
    /// The same result type as the wrapped processor.
    fn wrap(&self, processor: &P, input: I) -> impl Future<Output = ProcessorReturn<P, I>> + Send;
}

/// Transforms input and output types around a processor using processor-based converters.
///
/// `Adapter` is the heavyweight sibling of [`PureAdapter`]. While `PureAdapter` uses simple
/// functions for conversion, `Adapter` uses full [`Processor`]s, allowing for:
///
/// - **Async conversions**: Database lookups, API calls during transformation
/// - **Stateful conversions**: Converters that maintain caches or counters
/// - **Complex validation**: Multi-step input validation with early returns
///
/// # Type Flow
///
/// ```text
///                    ┌───────────────────────────────────────┐
///                    │              Adapter                  │
///                    │                                       │
///  Input ──────────► │ in_converter ──► [processor] ──► out_converter ──► Output
///                    │                                       │
///                    └───────────────────────────────────────┘
/// ```
///
/// # Type Parameters
///
/// - `Input` — External input type
/// - `InnerInput` — Type produced by `in_converter`, consumed by the wrapped processor
/// - `InnerOutput` — Type produced by the wrapped processor, consumed by `out_converter`
/// - `Err` — Shared error type across all processors
/// - `P1` — Input converter processor (`Input` → `InnerInput`)
/// - `P2` — Output converter processor (`InnerOutput` → final output)
///
/// # Example
///
/// ```
/// use kanau::layer::Adapter;
/// use kanau::processor::Processor;
///
/// // Converters as simple processors
/// struct ParseInt;
/// impl Processor<String> for ParseInt {
///     type Output = i32;
///     type Error = String;
///     async fn process(&self, s: String) -> Result<i32, String> {
///         s.parse().map_err(|e| format!("parse error: {e}"))
///     }
/// }
///
/// struct FormatResult;
/// impl Processor<i32> for FormatResult {
///     type Output = String;
///     type Error = String;
///     async fn process(&self, n: i32) -> Result<String, String> {
///         Ok(format!("Result: {n}"))
///     }
/// }
///
/// let adapter = Adapter::new(ParseInt, FormatResult);
/// // Now `adapter.wrap(&some_int_processor, string_input)` works
/// ```
#[derive(Debug, Clone)]
pub struct Adapter<
    Input: Send,
    InnerInput,
    InnerOutput: Send,
    Err,
    P1: Processor<Input, Output = InnerInput, Error = Err>,
    P2: Processor<InnerOutput, Error = Err>,
> {
    in_converter: P1,
    out_converter: P2,
    _in_phantom: PhantomData<fn(Input) -> Result<InnerInput, Err>>,
    _out_phantom: PhantomData<fn(InnerOutput) -> Result<P2::Output, Err>>,
}

impl<
    I1: Send,
    I: Send,
    O: Send,
    Err,
    P1: Processor<I1, Output = I, Error = Err>,
    P2: Processor<O, Error = Err>,
> Adapter<I1, I, O, Err, P1, P2>
{
    /// Creates a new adapter with the given input and output converters.
    ///
    /// # Arguments
    ///
    /// - `in_converter` — Processor that transforms external input to inner input
    /// - `out_converter` — Processor that transforms inner output to external output
    pub fn new(in_converter: P1, out_converter: P2) -> Self {
        Self {
            in_converter,
            out_converter,
            _in_phantom: PhantomData,
            _out_phantom: PhantomData,
        }
    }

    /// Applies the adapter to wrap a processor call.
    ///
    /// This method:
    /// 1. Converts the input using `in_converter`
    /// 2. Passes the converted input to the processor
    /// 3. Converts the output using `out_converter`
    ///
    /// Any error at any stage short-circuits and returns immediately.
    ///
    /// # Arguments
    ///
    /// - `processor` — The inner processor to wrap
    /// - `input` — The external input to process
    pub async fn wrap(
        &self,
        processor: &impl Processor<I, Output = O, Error = Err>,
        input: I1,
    ) -> ProcessorReturn<P2, O> {
        let converted = self.in_converter.process(input).await?;
        let result = processor.process(converted).await?;
        self.out_converter.process(result).await
    }

    /// Embeds a processor into this adapter, creating a [`ServiceChain`].
    ///
    /// This consumes the adapter and returns a composed processor chain:
    /// `in_converter → processor → out_converter`
    ///
    /// The resulting chain implements [`Processor`] and can be used anywhere
    /// a processor is expected.
    ///
    /// # Arguments
    ///
    /// - `processor` — The processor to embed between the converters
    ///
    /// # Returns
    ///
    /// A [`ServiceChain3`] that processes `I1` inputs and produces `P2::Output`.
    pub fn embed<PInner: Processor<I, Output = O, Error = Err>>(
        self,
        processor: PInner,
    ) -> ServiceChain3<I1, Err, P1, PInner, P2>
    where
        I: Send,
        PInner: Sync,
        P1: Sync,
    {
        ServiceChain::new(self.in_converter)
            .then(processor)
            .then(self.out_converter)
    }
}

/// A processor wrapped with a [`Proxy`] layer.
///
/// `ProxiedProcessor` combines a processor and a proxy into a new [`Processor`] implementation.
/// When [`process`](Processor::process) is called, it delegates to the proxy's [`wrap`](Proxy::wrap)
/// method, which controls how the inner processor is invoked.
///
/// This is the standard way to apply middleware to a processor and continue using it
/// in processor-based pipelines.
///
/// # Type Parameters
///
/// - `I` — Input type
/// - `P` — The wrapped processor
/// - `L` — The proxy/layer type
///
/// # Example
///
/// ```
/// use kanau::processor::Processor;
/// use kanau::layer::{Proxy, ProxiedProcessor};
///
/// struct LoggingProxy;
///
/// impl<I: Send + std::fmt::Debug, P: Processor<I> + Sync> Proxy<I, P> for LoggingProxy
/// where
///     P::Output: std::fmt::Debug,
/// {
///     async fn wrap(&self, processor: &P, input: I) -> Result<P::Output, P::Error> {
///         println!("Processing: {:?}", input);
///         let result = processor.process(input).await;
///         if let Ok(ref output) = result {
///             println!("Result: {:?}", output);
///         }
///         result
///     }
/// }
///
/// # struct MyProcessor;
/// # impl Processor<i32> for MyProcessor {
/// #     type Output = i32;
/// #     type Error = ();
/// #     async fn process(&self, x: i32) -> Result<i32, ()> { Ok(x) }
/// # }
/// // The result is a Processor with logging middleware
/// let logged = ProxiedProcessor::new(LoggingProxy, MyProcessor);
/// ```
#[derive(Debug, Clone)]
pub struct ProxiedProcessor<I: Send, P: Processor<I>, L: Proxy<I, P>> {
    layer: L,
    processor: P,
    _input_phantom: PhantomData<fn(I) -> ProcessorReturn<P, I>>,
}

impl<I: Send, P: Processor<I>, L: Proxy<I, P>> ProxiedProcessor<I, P, L> {
    /// Creates a new proxied processor.
    ///
    /// # Arguments
    ///
    /// - `layer` — The proxy that will intercept calls
    /// - `processor` — The processor to wrap
    pub fn new(layer: L, processor: P) -> Self {
        Self {
            layer,
            processor,
            _input_phantom: PhantomData,
        }
    }
}

impl<I: Send + Sync, P: Processor<I> + Sync + Send, L: Proxy<I, P> + Sync + Send> Processor<I>
    for ProxiedProcessor<I, P, L>
{
    type Output = P::Output;
    type Error = P::Error;

    async fn process(&self, input: I) -> ProcessorReturn<P, I> {
        self.layer.wrap(&self.processor, input).await
    }
}

/// Lightweight type adapter using pure functions.
///
/// `PureAdapter` is a zero-allocation alternative to [`Adapter`] when your type conversions
/// are simple, synchronous, and stateless. It stores function pointers rather than processor
/// instances, making it extremely cheap to clone and store.
///
/// # When to Use
///
/// | Use `PureAdapter` when... | Use [`Adapter`] when... |
/// |---------------------------|-------------------------|
/// | Conversions are sync | Conversions need async |
/// | No state needed | Converters have state |
/// | Simple transforms | Complex validation logic |
/// | Performance critical | Flexibility matters more |
///
/// # Type Parameters
///
/// - `Err` — Error type returned by conversion functions
/// - `I1` — External input type
/// - `O1` — External output type
/// - `I2` — Inner input type (processor's input)
/// - `O2` — Inner output type (processor's output)
///
/// # Example
///
/// ```
/// use kanau::layer::PureAdapter;
/// use kanau::processor::Processor;
///
/// # struct Calculator;
/// # impl Processor<i32> for Calculator {
/// #     type Output = i32;
/// #     type Error = &'static str;
/// #     async fn process(&self, x: i32) -> Result<i32, &'static str> { Ok(x * 2) }
/// # }
/// // Parse string to int, process, format back to string
/// let adapter = PureAdapter::<&str, String, String, i32, i32>::new_bidirectional(
///     |s| s.parse().map_err(|_| "invalid number"),
///     |n| Ok(format!("{n}")),
/// );
///
/// // Embed creates a processor: String → String
/// let string_calculator = adapter.embed(Calculator);
/// ```
#[derive(Debug, Clone)]
pub struct PureAdapter<Err, I1, O1, I2, O2> {
    in_function: fn(I1) -> Result<I2, Err>,
    out_function: fn(O2) -> Result<O1, Err>,
}

impl<Err, I1, I2, O> PureAdapter<Err, I1, O, I2, O> {
    /// Creates an adapter that only transforms input.
    ///
    /// The output passes through unchanged (identity function).
    ///
    /// # Arguments
    ///
    /// - `in_function` — Function to transform input `I1` → `I2`
    pub fn new_in(in_function: fn(I1) -> Result<I2, Err>) -> Self {
        Self {
            in_function,
            out_function: |x| Ok(x),
        }
    }
}

impl<Err, I, O1, O2> PureAdapter<Err, I, O1, I, O2> {
    /// Creates an adapter that only transforms output.
    ///
    /// The input passes through unchanged (identity function).
    ///
    /// # Arguments
    ///
    /// - `out_function` — Function to transform output `O2` → `O1`
    pub fn new_out(out_function: fn(O2) -> Result<O1, Err>) -> Self {
        Self {
            in_function: |x| Ok(x),
            out_function,
        }
    }
}

impl<Err, I1, O1, I2, O2> PureAdapter<Err, I1, O1, I2, O2> {
    /// Creates an adapter that transforms both input and output.
    ///
    /// # Arguments
    ///
    /// - `in_function` — Function to transform input `I1` → `I2`
    /// - `out_function` — Function to transform output `O2` → `O1`
    pub fn new_bidirectional(
        in_function: fn(I1) -> Result<I2, Err>,
        out_function: fn(O2) -> Result<O1, Err>,
    ) -> Self {
        Self {
            in_function,
            out_function,
        }
    }
}

impl<Err, I1, O1, I2, O2> PureAdapter<Err, I1, O1, I2, O2> {
    /// Embeds a processor into this adapter, creating a [`ProcessorPureFunctionChain`].
    ///
    /// This consumes the adapter and returns a composed processor:
    /// `in_function → processor → out_function`
    ///
    /// Unlike [`Adapter::embed`], this uses pure functions rather than processors
    /// for the conversions, avoiding the overhead of async machinery for simple transforms.
    ///
    /// # Arguments
    ///
    /// - `processor` — The processor to embed
    ///
    /// # Returns
    ///
    /// A [`ProcessorPureFunctionChain`] that processes `I1` inputs and produces `O1` outputs.
    pub fn embed<P: Processor<I2, Output = O2, Error = Err>>(
        self,
        processor: P,
    ) -> ProcessorPureFunctionChain<I1, O1, I2, Err, P>
    where
        I1: Send,
        I2: Send,
        O2: Send,
        P: Processor<I2, Output = O2, Error = Err> + Sync,
    {
        ProcessorPureFunctionChain::new_bidirectional(processor, self.in_function, self.out_function)
    }

    /// Applies this adapter to wrap a processor call.
    ///
    /// This method:
    /// 1. Converts input using `in_function`
    /// 2. Passes converted input to the processor
    /// 3. Converts output using `out_function`
    ///
    /// Note: This consumes `self`. For reusable wrapping, use [`embed`](Self::embed).
    ///
    /// # Arguments
    ///
    /// - `processor` — The processor to wrap
    /// - `input` — The external input
    pub async fn wrap<P: Processor<I1, Output = O1, Error = Err>>(
        self,
        processor: &P,
        input: I1,
    ) -> Result<O1, Err>
    where
        I1: Send,
        I2: Send,
        O2: Send,
        P: Processor<I2, Output = O2, Error = Err>,
    {
        let converted = (self.in_function)(input)?;
        let result = processor.process(converted).await?;
        (self.out_function)(result)
    }
}