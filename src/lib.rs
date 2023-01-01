pub type Value = i32;
pub type ForthResult = Result<(), Error>;

pub struct Forth {
    pub stack: Vec<Value>,
    pub definitions: Vec<Definition>,
}

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub instructions: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

#[derive(Debug)]
pub enum Instruction {
    Number(Value),
    Add,
    Subtract,
    Multiply,
    Divide,
    Dup,
    Drop,
    Over,
    Swap,
    CallDefinition(usize),
}

impl Default for Forth {
    fn default() -> Self {
        Self::new()
    }
}

impl Forth {
    pub fn new() -> Forth {
        Forth {
            stack: Vec::<Value>::new(),
            definitions: Vec::<Definition>::new(),
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack[..]
    }

    pub fn eval(&mut self, input: &str) -> ForthResult {
        let mut words = input.split_whitespace();
        while let Some(word) = words.next() {
            match word {
                ":" => self.add_definition(&mut words)?,
                _ => {
                    let max_index = self.definitions.len().saturating_sub(1);
                    self.eval_instruction(word, max_index)?
                }
            };
        }
        Ok(())
    }

    fn instruction_from_word(&self, word: &str, max_index: usize) -> Result<Instruction, Error> {
        let canonical = word.to_ascii_uppercase();

        for (index, definition) in self.definitions.iter().enumerate().rev() {
            if definition.name == canonical && index <= max_index {
                return Ok(Instruction::CallDefinition(index));
            }
        }

        match &canonical as &str {
            "+" => Ok(Instruction::Add),
            "-" => Ok(Instruction::Subtract),
            "*" => Ok(Instruction::Multiply),
            "/" => Ok(Instruction::Divide),
            "DUP" => Ok(Instruction::Dup),
            "DROP" => Ok(Instruction::Drop),
            "SWAP" => Ok(Instruction::Swap),
            "OVER" => Ok(Instruction::Over),
            _ => match word.parse::<Value>() {
                Ok(int) => Ok(Instruction::Number(int)),
                _ => Err(Error::UnknownWord),
            },
        }
    }

    fn add_definition<'a, I>(&mut self, words: &mut I) -> ForthResult
    where
        I: Iterator<Item = &'a str>,
    {
        let mut definition_instructions = Vec::<String>::new();
        let definition_name = match words.next() {
            Some(word) => {
                if word.parse::<Value>().is_ok() {
                    // cannot redefine numbers !
                    return Err(Error::InvalidWord);
                }
                word
            }
            None => return Err(Error::InvalidWord),
        };
        for word in words {
            if word == ";" {
                self.definitions.push(Definition {
                    name: definition_name.to_ascii_uppercase(),
                    instructions: definition_instructions.clone(),
                });
                return Ok(());
            } else {
                definition_instructions.push(word.to_string());
            };
        }
        Err(Error::InvalidWord)
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> Result<Value, Error> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            _ => Err(Error::StackUnderflow),
        }
    }

    fn eval_instruction(&mut self, word: &str, index: usize) -> ForthResult {
        let instruction = self.instruction_from_word(word, index)?;
        match instruction {
            Instruction::Number(value) => self.push_value_onto_the_stack(value),
            Instruction::Add => self.perform_maths_operation(Instruction::Add),
            Instruction::Subtract => self.perform_maths_operation(Instruction::Subtract),
            Instruction::Multiply => self.perform_maths_operation(Instruction::Multiply),
            Instruction::Divide => self.perform_maths_operation(Instruction::Divide),
            Instruction::Dup => self.dup(),
            Instruction::Drop => self.drop(),
            Instruction::Swap => self.swap(),
            Instruction::Over => self.over(),
            Instruction::CallDefinition(instruction_index) => {
                self.call_user_defined_instruction(instruction_index)
            }
        }
    }

    fn push_value_onto_the_stack(&mut self, value: Value) -> ForthResult {
        self.stack_push(value);
        Ok(())
    }

    fn call_user_defined_instruction(&mut self, instruction_index: usize) -> ForthResult {
        let def = self.definitions.get(instruction_index).unwrap();
        let max_index = if instruction_index > 0 {
            instruction_index - 1
        } else {
            instruction_index
        };
        for word in def.instructions.clone().into_iter() {
            self.eval_instruction(&word, max_index)?;
        }
        Ok(())
    }

    fn perform_maths_operation(&mut self, instruction: Instruction) -> ForthResult {
        if self.stack.len() < 2 {
            return Err(Error::StackUnderflow);
        }
        if let Instruction::Divide = instruction {
            for value in self.stack.iter().skip(1) {
                if *value == 0 {
                    return Err(Error::DivisionByZero);
                }
            }
        }
        let first_value = self.stack[0];
        self.stack =
            vec![self
                .stack
                .iter()
                .skip(1)
                .fold(first_value, |acc, v| match instruction {
                    Instruction::Add => acc + v,
                    Instruction::Subtract => acc - v,
                    Instruction::Multiply => acc * v,
                    _ => acc / v,
                })];
        Ok(())
    }

    fn dup(&mut self) -> ForthResult {
        let last = self.stack_pop()?;
        self.stack_push(last);
        self.stack_push(last);
        Ok(())
    }

    fn drop(&mut self) -> ForthResult {
        self.stack_pop()?;
        Ok(())
    }

    fn swap(&mut self) -> ForthResult {
        let last = self.stack_pop()?;
        let previous = self.stack_pop()?;
        self.stack_push(last);
        self.stack_push(previous);
        Ok(())
    }

    fn over(&mut self) -> ForthResult {
        let last = self.stack_pop()?;
        let previous = self.stack_pop()?;
        self.stack_push(previous);
        self.stack_push(last);
        self.stack_push(previous);
        Ok(())
    }
}
