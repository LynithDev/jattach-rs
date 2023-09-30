use std::error::Error;

pub mod posix;

pub trait VirtualMachine {
    fn execute_command(&mut self, command: &str, args: [&str; 3]) -> Result<String, Box<dyn Error>>;

    fn load_agent(&mut self, agent_path: &str, agent_arguments: &str) -> Result<(), Box<dyn Error>>;
    fn get_properties(&mut self) -> Result<Vec<(String, String)>, Box<dyn Error>>;
}