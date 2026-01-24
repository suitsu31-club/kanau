use crate::chain::{ProcessorPureFunctionChain, ServiceChain, ServiceChain3};
use crate::processor::{Processor, ProcessorReturn};
use std::fmt::Debug;
use std::marker::PhantomData;

/// ## Proxy
///
/// A Layer is a crucial component in kanau that wraps around a Processor,
/// enabling middleware-like functionality. It provides a clean way to
/// handle cross-cutting concerns by intercepting and potentially modifying
/// both the input and output of a Processor.
pub trait Proxy<I: Send, P: Processor<I>> {
    /// Wrap a processor with a layer.
    fn wrap(&self, processor: &P, input: I) -> impl Future<Output = ProcessorReturn<P, I>> + Send;
}

#[derive(Debug, Clone)]
/// ## Adapter
///
/// Convert types from I1 to I2 and from O2 to O1.
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
    /// Create a new adapter.
    pub fn new(in_converter: P1, out_converter: P2) -> Self {
        Self {
            in_converter,
            out_converter,
            _in_phantom: PhantomData,
            _out_phantom: PhantomData,
        }
    }

    /// Wrap a processor with an adapter.
    pub async fn wrap(
        &self,
        processor: &impl Processor<I, Output = O, Error = Err>,
        input: I1,
    ) -> ProcessorReturn<P2, O> {
        let converted = self.in_converter.process(input).await?;
        let result = processor.process(converted).await?;
        self.out_converter.process(result).await
    }

    /// embed the processor into the adapter to form a [ServiceChain].
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

#[derive(Debug, Clone)]
/// ## LayerWrappedProcessor
///
/// A processor that is wrapped with a layer.
pub struct ProxiedProcessor<I: Send, P: Processor<I>, L: Proxy<I, P>> {
    layer: L,
    processor: P,
    _input_phantom: PhantomData<fn(I) -> ProcessorReturn<P, I>>,
}

impl<I: Send, P: Processor<I>, L: Proxy<I, P>> ProxiedProcessor<I, P, L> {
    /// Create a new layer wrapped processor.
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

#[derive(Debug, Clone)]
/// ## PureAdapter
///
/// A pure adapter that does not hold any state.
pub struct PureAdapter<Err, I1, O1, I2, O2> {
    in_function: fn(I1) -> Result<I2, Err>,
    out_function: fn(O2) -> Result<O1, Err>,
}

impl<Err, I1, I2, O> PureAdapter<Err, I1, O, I2, O> {
    pub fn new_in(in_function: fn(I1) -> Result<I2, Err>) -> Self {
        Self {
            in_function,
            out_function: |x| Ok(x),
        }
    }
}

impl<Err, I, O1, O2> PureAdapter<Err, I, O1, I, O2> {
    pub fn new_out(out_function: fn(O2) -> Result<O1, Err>) -> Self {
        Self {
            in_function: |x| Ok(x),
            out_function,
        }
    }
}

impl<Err, I1, O1, I2, O2> PureAdapter<Err, I1, O1, I2, O2> {
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

    pub async fn wrap<P: Processor<I1, Output = O1, Error = Err>>(
        self,
        processor: &P,
        input: I1
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