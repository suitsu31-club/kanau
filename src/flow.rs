use crate::processor::Processor;

/// ## EarlyReturn
/// 
/// an enum that shows a value returned from a function should be returned early or not.
/// 
/// Usually used with [early_return!] or [monad_early_return!] macro.
pub enum EarlyReturn<Return, Expr = ()> {
    /// Treat the value as an expression.
    Expr(Expr),
    
    /// Treat the value as a return value.
    Return(Return),
}

impl<R,E> EarlyReturn<R,E> {
    /// Create an [EarlyReturn::Expr]
    pub fn expr(e: E) -> Self {
        EarlyReturn::Expr(e)
    }
    
    /// Create an [EarlyReturn::Return]
    pub fn ret(r: R) -> Self {
        EarlyReturn::Return(r)
    }
    
    /// Swap the return and expression value.
    pub fn swap(self) -> EarlyReturn<E, R> {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Return(e),
            EarlyReturn::Return(r) => EarlyReturn::Expr(r),
        }
    }
    
    /// get the second value if the first value is [EarlyReturn::Expr], otherwise return the first value.
    pub fn or_return<Expr2>(self, res: EarlyReturn<R, Expr2>) -> EarlyReturn<R, Expr2> {
        match self {
            EarlyReturn::Expr(_) => res,
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// get the second value if the first value is [EarlyReturn::Return], otherwise return the first value.
    pub fn or_expr<Return2>(self, res: EarlyReturn<Return2, E>) -> EarlyReturn<Return2, E> {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(e),
            EarlyReturn::Return(_) => res,
        }
    }
    
    /// Map the expression value.
    pub fn map<F: FnOnce(E) -> E2, E2>(self, f: F) -> EarlyReturn<R, E2>
    {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(f(e)),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// Map the expression value with an async function.
    pub async fn process_map<P: Processor<E, E2>, E2>(self, processor: &P) -> EarlyReturn<R, E2> {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(processor.process(e).await),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// Map the return value.
    pub fn map_return<F: FnOnce(R) -> R2, R2>(self, f: F) -> EarlyReturn<R2, E> {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(e),
            EarlyReturn::Return(r) => EarlyReturn::Return(f(r)),
        }
    }
    
    /// Bind function of the monad.
    pub fn flat_map<
        F: FnOnce(E) -> EarlyReturn<R, E2>,
        E2,
    >(self, f: F) -> EarlyReturn<R, E2> {
        match self {
            EarlyReturn::Expr(e) => f(e),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// Bind function of the monad with an async function.
    pub async fn process_flat_map<P: Processor<E, EarlyReturn<R, E2>>, E2>(
        self,
        processor: &P,
    ) -> EarlyReturn<R, E2> {
        match self {
            EarlyReturn::Expr(e) => processor.process(e).await,
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
}

impl<R, E> EarlyReturn<R, EarlyReturn<R, E>> {
    /// Flatten the early return.
    pub fn flatten(self) -> EarlyReturn<R, E> {
        match self {
            EarlyReturn::Expr(EarlyReturn::Expr(e)) => EarlyReturn::Expr(e),
            EarlyReturn::Expr(EarlyReturn::Return(r)) => EarlyReturn::Return(r),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
}

impl<R, E> EarlyReturn<R, &E> {
    /// Copy the expression value.
    pub fn copied_expr(self) -> EarlyReturn<R, E> where E: Copy {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(*e),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// Clone the expression value.
    pub fn cloned_expr(self) -> EarlyReturn<R, E> where E: Clone {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(e.clone()),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// Convert the expression value to an owned value.
    pub fn owned_expr<Owned>(self) -> EarlyReturn<R, Owned> where E: ToOwned<Owned = Owned> {
        match self {
            EarlyReturn::Expr(e) => EarlyReturn::Expr(e.to_owned()),
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
}

impl<Succ, Err, Expr> EarlyReturn<Result<Succ, Err>, Expr> {
    /// Map the expression value with a fallible function. Return the error if the function returns an error.
    pub fn try_map<
        F: FnOnce(Expr) -> Result<Expr2, Err2>,
        Expr2,
        Err2: Into<Err>
    >(self, f: F) -> EarlyReturn<Result<Succ, Err>, Expr2> {
        match self {
            EarlyReturn::Expr(e) => match f(e) {
                Ok(e) => EarlyReturn::Expr(e),
                Err(e) => EarlyReturn::Return(Err(e.into())),
            },
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
    
    /// Map the expression value with an async fallible function. Return the error if the function returns an error.
    pub async fn try_process_map<
        P: Processor<Expr, Result<Expr2, Err2>>, 
        Expr2,
        Err2: Into<Err>,
    >(
        self,
        processor: &P,
    ) -> EarlyReturn<Result<Succ, Err>, Expr2> {
        match self {
            EarlyReturn::Expr(e) => match processor.process(e).await {
                Ok(e) => EarlyReturn::Expr(e),
                Err(e) => EarlyReturn::Return(Err(e.into())),
            },
            EarlyReturn::Return(r) => EarlyReturn::Return(r),
        }
    }
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
