use crate::core::register::internal::Register;
use crate::core::register::name::RegisterName;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RegisterSystem {
    registers: HashMap<RegisterName, Register>,
}

impl RegisterSystem {
    pub fn new() -> Self {
        let registers = RegisterName::all_names()
            .into_iter()
            .map(|name| (name, Register::default()))
            .collect::<HashMap<_, _>>();
        Self { registers }
    }

    pub fn get(&self, name: &RegisterName) -> Option<&Register> {
        self.registers.get(name)
    }

    pub fn set(&mut self, name: &RegisterName, register: Register) {
        self.registers.insert(name.clone(), register);
    }

    pub fn on_yank(&mut self, register: Register) {
        self.registers
            .insert(RegisterName::Unnamed, register.clone());
        self.registers.insert(RegisterName::LAST_YANK, register);
    }

    pub fn on_delete(&mut self, register: Register) {
        self.registers
            .insert(RegisterName::Unnamed, register.clone());
        if register.content.len() < 50 && !register.content.contains('\n') {
            self.registers.insert(RegisterName::SmallDelete, register);
        } else {
            self.shift_numbered_registers(register);
        }
    }

    pub fn shift_numbered_registers(&mut self, register: Register) {
        for i in 1..9 {
            let value = self
                .registers
                .remove(&RegisterName::Numbered(i))
                .unwrap_or_default();
            self.registers.insert(RegisterName::Numbered(i + 1), value);
        }
        self.registers.insert(RegisterName::Numbered(1), register);
    }
}
