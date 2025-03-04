mod outputs;
pub use outputs::{
    check_outputs_allow_multiple_units, check_outputs_allow_single_unit, process_outputs,
    Error as OutputsError,
};
mod inputs;
pub use inputs::{process_melt_inputs, process_swap_inputs, Error as InputsError};
