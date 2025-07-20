#[derive(Debug, Default, Clone, PartialEq)]
pub enum RegisterType {
    #[default]
    Character,
    Line,
}

#[derive(Debug, Default, Clone)]
pub struct Register {
    pub content: String,
    pub register_type: RegisterType,
}

impl Register {
    pub fn new(content: String, register_type: RegisterType) -> Register {
        match register_type {
            RegisterType::Character => Self::character(content),
            RegisterType::Line => Self::line(content),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn character(content: String) -> Register {
        Self {
            content,
            register_type: RegisterType::Character,
        }
    }

    fn line(content: String) -> Register {
        let content = if content.ends_with('\n') {
            content
        } else {
            format!("{}\n", content)
        };
        Self {
            content,
            register_type: RegisterType::Line,
        }
    }

}

#[derive(Debug, Default)]
pub struct RegisterManager {
    unnamed: Register,
    numbered: [Register; 10],
    small_delete: Register,
}

impl RegisterManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_register(&mut self, name: char, content: Register) {
        match name {
            '"' => self.unnamed = content,
            '0'..'9' => {
                let index = name as u8 - b'0';
                self.numbered[index as usize] = content;
            }
            '_' => self.small_delete = content,
            _ => {}
        }
    }

    pub fn get_register(&self, name: char) -> Option<&Register> {
        match name {
            '"' => Some(&self.unnamed),
            '0'..'9' => {
                let index = name as u8 - b'0';
                self.numbered.get(index as usize)
            }
            '_' => Some(&self.small_delete),
            _ => None,
        }
    }

    pub fn on_yank(&mut self, content: String, register_type: RegisterType) {
        let register = Register::new(content, register_type);
        self.unnamed = register.clone();
        self.numbered[0] = register;
    }

    pub fn on_delete(&mut self, content: String, register_type: RegisterType) {
        let register = Register::new(content, register_type);
        log::info!("RegisterManager::on_delete: {:?}", register);
        self.unnamed = register.clone();

        if register.content.len() < 50 && !register.content.contains('\n') {
            self.small_delete = register;
        } else {
            self.shift_numbered_registers(register);
        }
    }

    fn shift_numbered_registers(&mut self, register: Register) {
        // Shift "1 -> "8 to "2 to "9, new content on "1
        for i in (1..=8).rev() {
            self.numbered[i + 1] = self.numbered[i].clone();
        }
        self.numbered[1] = register;
    }
}
