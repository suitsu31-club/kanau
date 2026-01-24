use crate::processor::{ArcProcessor, IdentityFunctor, Processor};
use std::marker::PhantomData;
use std::sync::Arc;

/// ## ServiceChain
///
/// A chain of multiple processors.
///
/// Consider the processors are functors, then this is a transformation semigroup.
/// With [IdentityFunctor] as the identity, it becomes a monoid.
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

pub type ServiceChain1<I1, Err, P1: Processor<I1, Error = Err>> =
    ServiceChain<I1, Err, IdentityFunctor<I1, Err>, P1>;
pub type ServiceChain2<
    I1,
    Err,
    P1: Processor<I1, Error = Err>,
    P2: Processor<P1::Output, Error = Err>,
> = ServiceChain<I1, Err, ServiceChain1<I1, Err, P1>, P2>;
pub type ServiceChain3<
    I1,
    Err,
    P1: Processor<I1, Error = Err>,
    P2: Processor<P1::Output, Error = Err>,
    P3: Processor<P2::Output, Error = Err>,
> = ServiceChain<I1, Err, ServiceChain2<I1, Err, P1, P2>, P3>;
pub type ServiceChain4<
    I1,
    Err,
    P1: Processor<I1, Error = Err>,
    P2: Processor<P1::Output, Error = Err>,
    P3: Processor<P2::Output, Error = Err>,
    P4: Processor<P3::Output, Error = Err>,
> = ServiceChain<I1, Err, ServiceChain3<I1, Err, P1, P2, P3>, P4>;

impl<I, Err, P> ServiceChain<I, Err, IdentityFunctor<I, Err>, P>
where
    P: Processor<I, Error = Err>,
    I: Send,
{
    /// Create a new service chain from a starting processor.
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
    /// Append a processor to the chain.
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

#[derive(Debug, Clone)]
/// ## ProcessorPureFunctionChain
///
/// A chain of a processor and two pure functions.
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

pub trait ProcessorChainExt<I: Send>: Processor<I> {
    fn chain_input_map<Input: Send>(
        self,
        input_cast: fn(Input) -> Result<I, Self::Error>,
    ) -> ProcessorPureFunctionChain<Input, Self::Output, I, Self::Error, Self>
    where
        Self: Sized,
    {
        ProcessorPureFunctionChain::new_in(self, input_cast)
    }
    fn chain_output_map<Output>(
        self,
        output_cast: fn(Self::Output) -> Result<Output, Self::Error>,
    ) -> ProcessorPureFunctionChain<I, Output, I, Self::Error, Self>
    where
        Self: Sized,
    {
        ProcessorPureFunctionChain::new_out(self, output_cast)
    }
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
