pub use sp_validate_input_proc_macro::ValidateInput;

pub trait ValidateInput {
	fn validate(&self);
}
