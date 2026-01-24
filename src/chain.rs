//! # Processor Chaining
//!
//! This module provides composable abstractions for sequentially chaining [Processor]s.
//!
//! ## Overview
//!
//! When building data processing pipelines, it's common to compose multiple processors
//! where the output of one feeds into the input of the next. This module offers two
//! primary mechanisms for such composition:
//!
//! - [ServiceChain]: Chains two processors together, forming a binary tree structure
//!   that can be nested to compose arbitrary-length pipelines.
//! - [ProcessorPureFunctionChain]: Wraps a processor with pure input/output transformation
//!   functions, enabling type adaptation without additional async overhead.
//!
//! ## Algebraic Structure
//!
//! The chaining mechanism forms a **monoid** under composition:
//! - Identity: [IdentityFunctor] serves as the identity element
//! - Associativity: `(P1 ∘ P2) ∘ P3 ≅ P1 ∘ (P2 ∘ P3)` (isomorphic in behavior)
//!
//! ## Usage
//!
//! The [ProcessorChainExt] trait provides ergonomic methods for chaining:
//!
//! ```
//! use kanau::chain::ProcessorChainExt;
//! use kanau::processor::Processor;
//!
//! struct ProcessorA;
//! struct ProcessorB;
//! struct ProcessorC;
//!
//! impl Processor<i32> for ProcessorA {
//!     type Output = i32;
//!     type Error = std::convert::Infallible;
//!     async fn process(&self, input: i32) -> Result<i32, Self::Error> {
//!         Ok(input * 2)
//!     }
//! }
//!
//! impl Processor<i32> for ProcessorB {
//!     type Output = f64;
//!     type Error = std::convert::Infallible;
//!     async fn process(&self, input: i32) -> Result<f64, Self::Error> {
//!         Ok(input as f64 + 114.514)
//!     }
//! }
//!
//! impl Processor<f64> for ProcessorC {
//!     type Output = String;
//!     type Error = std::convert::Infallible;
//!     async fn process(&self, input: f64) -> Result<String, Self::Error> {
//!         Ok(format!("{input:.2}"))
//!     }
//! }
//!
//! let processor_a = ProcessorA;
//! let processor_b = ProcessorB;
//! let processor_c = ProcessorC;
//! let pipeline = processor_a
//!     .chain(processor_b)
//!     .chain(processor_c);
//! ```

use crate::processor::{IdentityFunctor, Processor};
use std::marker::PhantomData;

/// A binary composition of two processors.
///
/// `ServiceChain` connects two processors `P1` and `P2` in sequence, where `P1`'s output
/// type matches `P2`'s input type. The resulting chain itself implements [`Processor`],
/// enabling recursive composition for arbitrary-length pipelines.
///
/// # Type Parameters
///
/// - `I1`: Input type of the first processor
/// - `Err`: Shared error type for both processors
/// - `P1`: The first processor in the chain
/// - `P2`: The second processor, consuming `P1`'s output
///
/// # Examples
///
/// ```
/// # use kanau::chain::ServiceChain;
/// # use kanau::processor::Processor;
/// # #[derive(Clone, Copy)]
/// # struct ProcessorA;
/// # #[derive(Clone, Copy)]
/// # struct ProcessorB;
/// # #[derive(Clone, Copy)]
/// # struct ProcessorC;
/// # impl Processor<i32> for ProcessorA {
/// #     type Output = i32;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: i32) -> Result<i32, Self::Error> {
/// #         Ok(input * 2)
/// #     }
/// # }
/// # impl Processor<i32> for ProcessorB {
/// #     type Output = f64;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: i32) -> Result<f64, Self::Error> {
/// #         Ok(input as f64 + 114.514)
/// #     }
/// # }
/// # impl Processor<f64> for ProcessorC {
/// #     type Output = String;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: f64) -> Result<String, Self::Error> {
/// #         Ok(format!("{input:.2}"))
/// #     }
/// # }
/// # let first_processor = ProcessorA;
/// # let second_processor = ProcessorB;
/// # let third_processor = ProcessorC;
/// // Direct construction
/// let service_chain = ServiceChain::new(first_processor)
///     .then(second_processor)
///     .then(third_processor);
///
/// // Using the extension trait
/// use kanau::chain::ProcessorChainExt;
/// let service_chain = first_processor
///     .chain(second_processor)
///     .chain(third_processor);
/// ```
#[derive(Debug, Clone)]
pub struct ServiceChain<
    I1: Send,
    Err,
    P1: Processor<I1, Error = Err>,
    P2: Processor<P1::Output, Error = Err>,
> where
    P1::Output: Send,
{
    processor1: P1,
    processor2: P2,
    _input_phantom: PhantomData<fn(I1) -> Result<P1::Output, Err>>,
}

#[allow(missing_docs)]
pub type ServiceChain1<I1, Err, P1> = ServiceChain<I1, Err, IdentityFunctor<I1, Err>, P1>;

#[allow(missing_docs)]
pub type ServiceChain2<I1, Err, P1, P2> = ServiceChain<I1, Err, ServiceChain1<I1, Err, P1>, P2>;

#[allow(missing_docs)]
pub type ServiceChain3<I1, Err, P1, P2, P3> =
    ServiceChain<I1, Err, ServiceChain2<I1, Err, P1, P2>, P3>;

#[allow(missing_docs)]
pub type ServiceChain4<I1, Err, P1, P2, P3, P4> =
    ServiceChain<I1, Err, ServiceChain3<I1, Err, P1, P2, P3>, P4>;

#[allow(missing_docs)]
pub type ServiceChain5<I1, Err, P1, P2, P3, P4, P5> =
    ServiceChain<I1, Err, ServiceChain4<I1, Err, P1, P2, P3, P4>, P5>;

#[allow(missing_docs)]
pub type ServiceChain6<I1, Err, P1, P2, P3, P4, P5, P6> =
    ServiceChain<I1, Err, ServiceChain5<I1, Err, P1, P2, P3, P4, P5>, P6>;

#[allow(missing_docs)]
pub type ServiceChain7<I1, Err, P1, P2, P3, P4, P5, P6, P7> =
    ServiceChain<I1, Err, ServiceChain6<I1, Err, P1, P2, P3, P4, P5, P6>, P7>;

#[allow(missing_docs)]
pub type ServiceChain8<I1, Err, P1, P2, P3, P4, P5, P6, P7, P8> =
    ServiceChain<I1, Err, ServiceChain7<I1, Err, P1, P2, P3, P4, P5, P6, P7>, P8>;

impl<I, Err, P> ServiceChain<I, Err, IdentityFunctor<I, Err>, P>
where
    P: Processor<I, Error = Err>,
    I: Send,
{
    /// Creates a new service chain with a single processor.
    ///
    /// This is the entry point for building a chain. Use [`then`](ServiceChain::then)
    /// to append additional processors.
    pub fn new(starting_processor: P) -> Self {
        Self {
            processor1: IdentityFunctor::new(),
            processor2: starting_processor,
            _input_phantom: PhantomData,
        }
    }
}

impl<I1: Send, Err, P1, P2> ServiceChain<I1, Err, P1, P2>
where
    P1: Processor<I1, Error = Err> + Sync,
    P1::Output: Send,
    P2: Processor<P1::Output, Error = Err> + Sync,
{
    /// Appends a processor to the end of this chain.
    ///
    /// Returns a new `ServiceChain` where the current chain becomes the first
    /// processor and the given processor becomes the second.
    pub fn then<P3: Processor<P2::Output, Error = Err>>(
        self,
        processor3: P3,
    ) -> ServiceChain<I1, Err, Self, P3>
    where
        P2::Output: Send,
    {
        ServiceChain {
            processor1: self,
            processor2: processor3,
            _input_phantom: PhantomData,
        }
    }
}

impl<I1: Send, Err, P1, P2> Processor<I1> for ServiceChain<I1, Err, P1, P2>
where
    P1: Processor<I1, Error = Err> + Sync,
    P2: Processor<P1::Output, Error = Err> + Sync,
    P1::Output: Send,
    P2::Output: Send,
{
    type Output = P2::Output;
    type Error = Err;
    fn process(&self, input: I1) -> impl Future<Output = Result<P2::Output, Err>> + Send {
        async move {
            let output1 = self.processor1.process(input).await?;
            self.processor2.process(output1).await
        }
    }
}

/// A processor wrapped with pure input and output transformation functions.
///
/// This struct adapts a processor's interface by applying synchronous, pure functions
/// to its input and output. This is useful for type conversions, validation, or
/// lightweight transformations without the overhead of a full async processor.
///
/// # Type Parameters
///
/// - `Input`: External input type (what callers provide)
/// - `Output`: External output type (what callers receive)
/// - `InnerInput`: The wrapped processor's actual input type
/// - `Err`: Shared error type
/// - `Proc`: The inner processor being wrapped
///
/// # Construction
///
/// - [`new_bidirectional`](Self::new_bidirectional): Wrap with both input and output functions
/// - [`new_in`](Self::new_in): Wrap with only an input transformation
/// - [`new_out`](Self::new_out): Wrap with only an output transformation
///
/// # Examples
///
/// ```
/// # use kanau::chain::ProcessorPureFunctionChain;
/// # use kanau::processor::Processor;
/// # struct ByteProcessor;
/// # impl Processor<Vec<u8>> for ByteProcessor {
/// #     type Output = Vec<u8>;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: Vec<u8>) -> Result<Vec<u8>, Self::Error> {
/// #         Ok(input.into_iter().map(|b| b.wrapping_add(1)).collect())
/// #     }
/// # }
/// # let inner_processor = ByteProcessor;
/// // Transform string input to bytes, process, then format output
/// let adapted = ProcessorPureFunctionChain::new_bidirectional(
///     inner_processor,
///     |s: String| Ok(s.into_bytes()),
///     |bytes| Ok(String::from_utf8_lossy(&bytes).into_owned()),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ProcessorPureFunctionChain<Input: Send, Output, InnerInput: Send, Err, Proc>
where
    Proc: Processor<InnerInput, Error = Err>,
{
    processor: Proc,
    in_function: fn(Input) -> Result<InnerInput, Err>,
    out_function: fn(Proc::Output) -> Result<Output, Err>,
}

impl<Input, Output, InnerInput, Err, Proc>
    ProcessorPureFunctionChain<Input, Output, InnerInput, Err, Proc>
where
    Input: Send,
    InnerInput: Send,
    Proc: Processor<InnerInput, Error = Err>,
{
    /// Creates a new chain with both input and output transformation functions.
    pub fn new_bidirectional(
        processor: Proc,
        in_function: fn(Input) -> Result<InnerInput, Err>,
        out_function: fn(Proc::Output) -> Result<Output, Err>,
    ) -> Self {
        Self {
            processor,
            in_function,
            out_function,
        }
    }
}

impl<Input, InnerInput, Err, Proc>
    ProcessorPureFunctionChain<Input, Proc::Output, InnerInput, Err, Proc>
where
    Input: Send,
    InnerInput: Send,
    Proc: Processor<InnerInput, Error = Err>,
{
    /// Creates a new chain with only an input transformation function.
    ///
    /// The output type remains unchanged from the inner processor.
    pub fn new_in(processor: Proc, in_function: fn(Input) -> Result<InnerInput, Err>) -> Self {
        Self {
            processor,
            in_function,
            out_function: |x| Ok(x),
        }
    }
}

impl<Output, InnerInput, Err, Proc>
    ProcessorPureFunctionChain<InnerInput, Output, InnerInput, Err, Proc>
where
    InnerInput: Send,
    Proc: Processor<InnerInput, Error = Err>,
{
    /// Creates a new chain with only an output transformation function.
    ///
    /// The input type remains unchanged from the inner processor.
    pub fn new_out(processor: Proc, out_function: fn(Proc::Output) -> Result<Output, Err>) -> Self {
        Self {
            processor,
            in_function: |x| Ok(x),
            out_function,
        }
    }
}

impl<Input, Output, InnerInput, Err, Proc> Processor<Input>
    for ProcessorPureFunctionChain<Input, Output, InnerInput, Err, Proc>
where
    Input: Send,
    InnerInput: Send,
    Proc: Processor<InnerInput, Error = Err> + Sync,
{
    type Output = Output;
    type Error = Err;
    async fn process(&self, input: Input) -> Result<Output, Err> {
        let inner_input = (self.in_function)(input)?;
        let output = self.processor.process(inner_input).await?;
        (self.out_function)(output)
    }
}

/// Extension trait providing fluent chaining methods for [`Processor`] implementations.
///
/// This trait is automatically implemented for all types that implement [`Processor`],
/// enabling ergonomic pipeline construction through method chaining.
///
/// # Methods
///
/// - [`chain`](Self::chain): Append another processor to form a [`ServiceChain`]
/// - [`chain_input_map`](Self::chain_input_map): Prepend an input transformation function
/// - [`chain_output_map`](Self::chain_output_map): Append an output transformation function
///
/// # Examples
///
/// ```
/// # use kanau::chain::ProcessorChainExt;
/// # use kanau::processor::Processor;
/// # struct ParseProcessor;
/// # struct ValidateProcessor;
/// # struct TransformProcessor;
/// # #[derive(Clone)]
/// # struct Parsed(i32);
/// # impl Parsed { fn normalize(&self) -> Self { self.clone() } }
/// # impl Processor<String> for ParseProcessor {
/// #     type Output = Parsed;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: String) -> Result<Parsed, Self::Error> {
/// #         Ok(Parsed(input.len() as i32))
/// #     }
/// # }
/// # impl Processor<Parsed> for ValidateProcessor {
/// #     type Output = Parsed;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: Parsed) -> Result<Parsed, Self::Error> {
/// #         Ok(input)
/// #     }
/// # }
/// # impl Processor<Parsed> for TransformProcessor {
/// #     type Output = String;
/// #     type Error = std::convert::Infallible;
/// #     async fn process(&self, input: Parsed) -> Result<String, Self::Error> {
/// #         Ok(format!("result: {}", input.0))
/// #     }
/// # }
/// # let parse_processor = ParseProcessor;
/// # let validate_processor = ValidateProcessor;
/// # let transform_processor = TransformProcessor;
/// // Build a pipeline with type adaptations
/// let pipeline = parse_processor
///     .chain_output_map(|parsed| Ok(parsed.normalize()))
///     .chain(validate_processor)
///     .chain(transform_processor);
/// ```
pub trait ProcessorChainExt<I: Send>: Processor<I> {
    /// Prepends an input transformation function to this processor.
    ///
    /// The provided function converts the external input type to the processor's
    /// expected input type, allowing interface adaptation.
    fn chain_input_map<Input: Send>(
        self,
        input_cast: fn(Input) -> Result<I, Self::Error>,
    ) -> ProcessorPureFunctionChain<Input, Self::Output, I, Self::Error, Self>
    where
        Self: Sized,
    {
        ProcessorPureFunctionChain::new_in(self, input_cast)
    }
    /// Appends an output transformation function to this processor.
    ///
    /// The provided function converts the processor's output to a different type,
    /// useful for formatting, type conversion, or post-processing.
    fn chain_output_map<Output>(
        self,
        output_cast: fn(Self::Output) -> Result<Output, Self::Error>,
    ) -> ProcessorPureFunctionChain<I, Output, I, Self::Error, Self>
    where
        Self: Sized,
    {
        ProcessorPureFunctionChain::new_out(self, output_cast)
    }

    /// Chains another processor after this one, forming a [`ServiceChain`].
    ///
    /// The output of `self` becomes the input of `processor2`. Both processors
    /// must share the same error type.
    fn chain<P2: Processor<Self::Output, Error = Self::Error>>(
        self,
        processor2: P2,
    ) -> ServiceChain2<I, Self::Error, Self, P2>
    where
        Self: Sized + Sync,
        Self::Output: Send,
        I: Send,
    {
        ServiceChain::new(self).then(processor2)
    }
}

impl<I: Send, P: Processor<I>> ProcessorChainExt<I> for P {}
