use crate::processor::Processor;
use std::fmt::Debug;
use std::marker::PhantomData;

/// ## Layer
///
/// A Layer is a crucial component in kanau that wraps around a Processor,
/// enabling middleware-like functionality. It provides a clean way to
/// handle cross-cutting concerns by intercepting and potentially modifying
/// both the input and output of a Processor.
pub trait Layer<I, O, P: Processor<I, O>> {
    /// Wrap a processor with a layer.
    fn wrap(&self, processor: &P, input: I) -> impl Future<Output = O> + Send;
}

/// ## Adapter
///
/// Convert types from I1 to I2 and from O2 to O1.
pub struct Adapter<I1, O1, I2, O2, P1: Processor<I1, I2>, P2: Processor<O2, O1>> {
    in_converter: P1,
    out_converter: P2,
    _in_phantom: PhantomData<(I1, I2)>,
    _out_phantom: PhantomData<(O2, O1)>,
}

impl<I1, O1, I2, O2, P1: Processor<I1, I2>, P2: Processor<O2, O1>> Adapter<I1, O1, I2, O2, P1, P2> {
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
    pub async fn wrap_process(&self, processor: &impl Processor<I2, O2>, input: I1) -> O1 {
        let converted = self.in_converter.process(input).await;
        let result = processor.process(converted).await;
        self.out_converter.process(result).await
    }

    /// Wrap a processor with an adapter.
    pub fn wrap(
        self,
        processor: impl Processor<I2, O2>,
    ) -> AdapterWrappedProcessor<I1, O1, I2, O2, P1, P2, impl Processor<I2, O2>> {
        AdapterWrappedProcessor {
            adapter: self,
            processor,
        }
    }
}

impl<I1, O1, I2, O2, P1: Processor<I1, I2> + Clone, P2: Processor<O2, O1> + Clone> Clone
    for Adapter<I1, O1, I2, O2, P1, P2>
{
    fn clone(&self) -> Self {
        Self {
            in_converter: self.in_converter.clone(),
            out_converter: self.out_converter.clone(),
            _in_phantom: PhantomData,
            _out_phantom: PhantomData,
        }
    }
}

impl<I1, O1, I2, O2, P1: Processor<I1, I2> + Debug, P2: Processor<O2, O1> + Debug> Debug
    for Adapter<I1, O1, I2, O2, P1, P2>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Adapter")
            .field("in_converter", &self.in_converter)
            .field("out_converter", &self.out_converter)
            .finish()
    }
}

/// ## AdapterWrappedProcessor
///
/// A processor that is wrapped with an adapter.
pub struct AdapterWrappedProcessor<
    I1,
    O1,
    I2,
    O2,
    P1: Processor<I1, I2>,
    P2: Processor<O2, O1>,
    P3: Processor<I2, O2>,
> {
    adapter: Adapter<I1, O1, I2, O2, P1, P2>,
    processor: P3,
}

impl<
    I1,
    O1,
    I2,
    O2,
    P1: Processor<I1, I2> + Clone,
    P2: Processor<O2, O1> + Clone,
    P3: Processor<I2, O2> + Clone,
> Clone for AdapterWrappedProcessor<I1, O1, I2, O2, P1, P2, P3>
{
    fn clone(&self) -> Self {
        Self {
            adapter: self.adapter.clone(),
            processor: self.processor.clone(),
        }
    }
}

impl<
    I1,
    O1,
    I2,
    O2,
    P1: Processor<I1, I2> + Debug,
    P2: Processor<O2, O1> + Debug,
    P3: Processor<I2, O2> + Debug,
> Debug for AdapterWrappedProcessor<I1, O1, I2, O2, P1, P2, P3>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdapterWrappedProcessor")
            .field("adapter", &self.adapter)
            .field("processor", &self.processor)
            .finish()
    }
}

impl<I1, O1, I2, O2, P1: Processor<I1, I2>, P2: Processor<O2, O1>, P3: Processor<I2, O2>>
    AdapterWrappedProcessor<I1, O1, I2, O2, P1, P2, P3>
{
    /// Create a new adapter wrapped processor.
    pub fn new(adapter: Adapter<I1, O1, I2, O2, P1, P2>, processor: P3) -> Self {
        Self { adapter, processor }
    }
}

impl<
    I1: Send + Sync,
    O1: Send + Sync,
    I2: Send + Sync,
    O2: Send + Sync,
    P1: Processor<I1, I2> + Sync + Send,
    P2: Processor<O2, O1> + Sync + Send,
    P3: Processor<I2, O2> + Sync + Send,
> Processor<I1, O1> for AdapterWrappedProcessor<I1, O1, I2, O2, P1, P2, P3>
{
    async fn process(&self, input: I1) -> O1 {
        self.adapter.wrap_process(&self.processor, input).await
    }
}

/// ## LayerWrappedProcessor
///
/// A processor that is wrapped with a layer.
pub struct LayerWrappedProcessor<I, O, P: Processor<I, O>, L: Layer<I, O, P>> {
    layer: L,
    processor: P,
    _input_phantom: PhantomData<I>,
    _output_phantom: PhantomData<O>,
}

impl<I, O, P: Processor<I, O>, L: Layer<I, O, P>> LayerWrappedProcessor<I, O, P, L> {
    /// Create a new layer wrapped processor.
    pub fn new(layer: L, processor: P) -> Self {
        Self {
            layer,
            processor,
            _input_phantom: PhantomData,
            _output_phantom: PhantomData,
        }
    }
}

impl<
    I: Send + Sync,
    O: Send + Sync,
    P: Processor<I, O> + Sync + Send,
    L: Layer<I, O, P> + Sync + Send,
> Processor<I, O> for LayerWrappedProcessor<I, O, P, L>
{
    async fn process(&self, input: I) -> O {
        self.layer.wrap(&self.processor, input).await
    }
}

impl<I, O, P: Processor<I, O>, L: Layer<I, O, P>> Clone for LayerWrappedProcessor<I, O, P, L>
where
    P: Clone,
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            layer: self.layer.clone(),
            processor: self.processor.clone(),
            _input_phantom: PhantomData,
            _output_phantom: PhantomData,
        }
    }
}

impl<I, O, P: Processor<I, O>, L: Layer<I, O, P>> Debug for LayerWrappedProcessor<I, O, P, L>
where
    P: Debug,
    L: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayerWrappedProcessor")
            .field("layer", &self.layer)
            .field("processor", &self.processor)
            .finish()
    }
}

/// ## PureAdapter
///
/// A pure adapter that does not hold any state.
pub struct PureAdapter<I1, O1, I2, O2> {
    in_function: fn(I1) -> I2,
    out_function: fn(O2) -> O1,
    _in_phantom: PhantomData<(I1, I2)>,
    _out_phantom: PhantomData<(O2, O1)>,
}

impl<I1, O1, I2, O2> PureAdapter<I1, O1, I2, O2> {
    /// Create a new pure adapter.
    pub fn new(in_function: fn(I1) -> I2, out_function: fn(O2) -> O1) -> Self {
        Self {
            in_function,
            out_function,
            _in_phantom: PhantomData,
            _out_phantom: PhantomData,
        }
    }
}

impl<I1, O1, I2, O2> Clone for PureAdapter<I1, O1, I2, O2> {
    fn clone(&self) -> Self {
        Self {
            in_function: self.in_function,
            out_function: self.out_function,
            _in_phantom: PhantomData,
            _out_phantom: PhantomData,
        }
    }
}

impl<I1, O1, I2, O2> Debug for PureAdapter<I1, O1, I2, O2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PureAdapter")
            .field("in_function", &"fn(I1) -> I2")
            .field("out_function", &"fn(O2) -> O1")
            .finish()
    }
}

/// ## PureAdapterWrappedProcessor
///
/// A processor that is wrapped with a pure adapter.
pub struct PureAdapterWrappedProcessor<I1, O1, I2, O2, P: Processor<I2, O2>> {
    adapter: PureAdapter<I1, O1, I2, O2>,
    processor: P,
}

impl<I1, O1, I2, O2, P: Processor<I2, O2>> PureAdapterWrappedProcessor<I1, O1, I2, O2, P> {
    /// Create a new pure adapter wrapped processor.
    pub fn new(adapter: PureAdapter<I1, O1, I2, O2>, processor: P) -> Self {
        Self { adapter, processor }
    }
}

impl<I1, O1, I2, O2, P: Processor<I2, O2>> Clone for PureAdapterWrappedProcessor<I1, O1, I2, O2, P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            adapter: self.adapter.clone(),
            processor: self.processor.clone(),
        }
    }
}

impl<I1, O1, I2, O2, P: Processor<I2, O2>> Debug for PureAdapterWrappedProcessor<I1, O1, I2, O2, P>
where
    P: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PureAdapterWrappedProcessor")
            .field("adapter", &self.adapter)
            .field("processor", &self.processor)
            .finish()
    }
}

impl<
    I1: Send + Sync,
    O1: Send + Sync,
    I2: Send + Sync,
    O2: Send + Sync,
    P: Processor<I2, O2> + Sync + Send,
> Processor<I1, O1> for PureAdapterWrappedProcessor<I1, O1, I2, O2, P>
{
    async fn process(&self, input: I1) -> O1 {
        let input = (self.adapter.in_function)(input);
        let output = self.processor.process(input).await;
        (self.adapter.out_function)(output)
    }
}
