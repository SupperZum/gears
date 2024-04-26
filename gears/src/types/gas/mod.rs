/// Module for basic gas meter
pub mod basic_meter;
/// Module for config of gas meter.
pub mod config;
/// Module for infinite gas meter.
pub mod infinite_meter;
// Different descriptor for gas meter
pub mod descriptor;
// Kinds of gas meters
pub mod kind;

use std::fmt::Debug;
use std::marker::PhantomData;

use self::kind::MeterKind;

#[no_link]
extern crate derive_more;

use derive_more::{Add, Deref, Display, From, Mul};
use tracing::debug;

#[derive(
    Copy,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    From,
    Add,
    Mul,
    Display,
    Deref,
)]
pub struct Gas(u64);

impl Gas {
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    pub const fn into_inner(self) -> u64 {
        self.0
    }

    pub const MAX_GAS: Gas = Gas::new(u64::MAX);
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Error parsing gas: {0}")]
pub struct GasParseError(pub String);

impl TryFrom<i64> for Gas {
    type Error = GasParseError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < -1 {
            Err(GasParseError(
                "Invalid max block gas. Value can't be lower that -1".to_owned(),
            ))
        } else if value == -1 {
            Ok(Gas(0))
        } else {
            Ok(Gas(value as u64))
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum GasErrors {
    #[error("Out of gas: {0}")]
    ErrorOutOfGas(String),
    #[error("Gas overflow: {0}")]
    ErrorGasOverflow(String),
}

#[derive(Debug)]
pub struct ErrorNegativeGasConsumed(pub String);

pub enum GasRemaining {
    NoLimit, // What about returing used gas in this case?
    Some(Gas),
}

pub trait PlainGasMeter: Send + Sync + Debug {
    // Return name of this gas meter. Used mainly for debug and logging purposes
    fn name(&self) -> &'static str;
    /// Returns the amount of gas that was consumed by the gas meter instance.
    fn gas_consumed(&self) -> Gas;
    /// Returns the amount of gas that was consumed by gas meter instance, or the limit if it is reached.
    fn gas_consumed_or_limit(&self) -> Gas;
    /// Returns the gas left in the GasMeter. Returns `None` if gas meter is infinite.
    fn gas_remaining(&self) -> GasRemaining;
    /// Returns the limit of the gas meter instance. `None` if the gas meter is infinite.
    fn limit(&self) -> Option<Gas>;
    /// Consumes the amount of gas provided.
    /// If the gas overflows, it returns error with the descriptor message.
    /// If the gas meter is not infinite, it returns error  if gas consumed goes above the limit.
    fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasErrors>;
    /// Deducts the given amount from the gas consumed.
    /// This functionality enables refunding gas to the transaction
    /// or block gas pools so that EVM-compatible chains can fully support the go-ethereum StateDB interface.
    fn refund_gas(&mut self, amount: Gas, descriptor: &str)
        -> Result<(), ErrorNegativeGasConsumed>;
    /// Returns true if the amount of gas consumed by the gas meter instance is strictly above the limit, false otherwise.
    fn is_past_limit(&self) -> bool;
    /// Returns true if the amount of gas consumed by the gas meter instance is above or equal to the limit, false otherwise.
    fn is_out_of_gas(&self) -> bool;
}

/// Wrapper around any gas meter which prevents usage of gas over limit with type system
#[derive(Debug)]
pub struct GasMeter<DS> {
    meter: Box<dyn PlainGasMeter>,
    _descriptor: PhantomData<DS>,
}

impl<DS> GasMeter<DS> {
    pub fn new(meter: Box<dyn PlainGasMeter>) -> Self {
        Self {
            meter,
            _descriptor: PhantomData,
        }
    }
}

impl<DS: MeterKind> GasMeter<DS> {
    pub fn replace_meter(&mut self, meter: Box<dyn PlainGasMeter>) {
        let _ = std::mem::replace(&mut self.meter, meter);
    }

    pub fn consumed_or_limit(&mut self) -> Gas {
        self.meter.gas_consumed_or_limit()
    }

    pub fn consume_gas(&mut self, amount: Gas, descriptor: &str) -> Result<(), GasErrors> {
        debug!(
            "Consumed {} gas for {} with {}",
            amount,
            self.meter.name(),
            descriptor
        );
        self.meter.consume_gas(amount, descriptor)
    }

    pub fn is_out_of_gas(&self) -> bool {
        self.meter.is_out_of_gas()
    }

    pub fn limit(&self) -> Option<Gas> {
        self.meter.limit()
    }

    pub fn gas_remaining(&self) -> GasRemaining {
        self.meter.gas_remaining()
    }
}
