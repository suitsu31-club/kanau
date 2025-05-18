use crate::processor::Processor;

/// ## EarlyReturn
/// 
/// an enum that shows a value returned from a function should be returned early or not.
/// 
/// Usually used with [early_return!] macro.
pub enum EarlyReturn<Return, Expr = ()> {
    /// Treat the value as an expression.
    Expr(Expr),
    
    /// Treat the value as a return value.
    Return(Return),
}

#[macro_export]
/// ## early_return
/// 
/// A macro that returns early if the value is [EarlyReturn::Return], otherwise returns the value.
macro_rules! early_return {
    ($e:expr) => {
        match $e {
            $crate::flow::EarlyReturn::Return(r) => return r,
            $crate::flow::EarlyReturn::Expr(e) => e,
        }
    }
}

#[macro_export]
/// ## monad_early_return
/// 
/// A macro that returns early if the value is [EarlyReturn::Return], otherwise returns the value.
/// 
/// This macro is useful when you want to return early from a function that returns [EarlyReturn].
macro_rules! monad_early_return {
    ($e:expr) => {
        match $e {
            $crate::flow::EarlyReturn::Return(r) => return $crate::flow::EarlyReturn::Return(r),
            $crate::flow::EarlyReturn::Expr(e) => e,
        }
    };
}

/// ## Continuation Passing Style (CPS)
/// 
/// A function that takes a processor and a next function, 
/// and returns the result of the next function.
/// 
/// The next function is called with the result of the processor.
pub async fn cps_pure<
    I,
    O,
    P: Processor<I, O>,
    Next
>(
    processor: &P,
    input: I,
    next: fn(O) -> Next,
) -> Next {
    next(processor.process(input).await)
}

/// ## Continuation Passing Style (CPS)
/// 
/// A function that takes two processors and an input, 
/// and returns the result of the second processor.
/// 
/// The first processor is called with the input, 
/// and the second processor is called with the result of the first processor.
pub async fn cps<
    I,
    O,
    Return,
    P1: Processor<I, EarlyReturn<Return, O>>,
    Final,
    P2: Processor<O, EarlyReturn<Return, Final>>,
>(
    first: &P1,
    rest: &P2,
    input: I,
) -> EarlyReturn<Return, Final> {
    let step1 = monad_early_return!(first.process(input).await);
    rest.process(step1).await
}
